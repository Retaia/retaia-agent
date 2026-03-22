#[cfg(not(feature = "desktop-shell"))]
fn main() {
    let lang = retaia_agent::detect_language();
    eprintln!("{}", retaia_agent::t(lang, "desktop.feature_required"));
    std::process::exit(1);
}

#[cfg(feature = "desktop-shell")]
mod desktop_shell {
    use std::path::PathBuf;
    use std::process::exit;
    use std::str::FromStr;
    use std::time::{Duration, Instant};

    use clap::Parser;
    use eframe::egui;
    use retaia_agent::{
        AgentRuntimeConfig, AuthMode, ConfigRepository, DAEMON_STATS_FILE_NAME, DaemonLabelRequest,
        DaemonLevel, DaemonManager, DaemonManagerError, DaemonRuntimeStats, DaemonStatus,
        DiagnosticsLimits, FileConfigRepository, Language, LogLevel, NotificationSinkProfile,
        RuntimeStatsStoreError, SystemConfigRepository, SystemNotification,
        build_bug_report_markdown, collect_daemon_diagnostics, compact_validation_reason,
        copy_to_clipboard, detect_language, dispatch_notifications, load_runtime_stats,
        normalize_core_api_url, normalize_storage_mount_path, now_unix_ms,
        redacted_runtime_config_from, render_daemon_inspect, render_daemon_inspect_json,
        runtime_history_db_path, select_notification_sink, t, validate_config,
    };
    use service_manager::{
        ServiceLabel, ServiceLevel, ServiceStartCtx, ServiceStatusCtx, ServiceStopCtx,
    };
    use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
    use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

    const DAEMON_LABEL: &str = "io.retaia.agent";

    #[derive(Debug, Parser)]
    #[command(
        name = "agent-desktop-shell",
        about = "Retaia desktop shell (daemon control center)"
    )]
    struct Cli {
        #[arg(long = "config")]
        config: Option<PathBuf>,
    }

    #[derive(Debug, Default, Clone, Copy)]
    struct NativeDaemonManager;

    impl NativeDaemonManager {
        fn with_manager<T>(
            level: DaemonLevel,
            f: impl FnOnce(&dyn service_manager::ServiceManager) -> Result<T, DaemonManagerError>,
        ) -> Result<T, DaemonManagerError> {
            let mut manager = <dyn service_manager::ServiceManager>::native()
                .map_err(|error| DaemonManagerError::Unavailable(error.to_string()))?;
            manager
                .set_level(match level {
                    DaemonLevel::User => ServiceLevel::User,
                    DaemonLevel::System => ServiceLevel::System,
                })
                .map_err(|error| DaemonManagerError::OperationFailed(error.to_string()))?;
            f(manager.as_ref())
        }
    }

    impl DaemonManager for NativeDaemonManager {
        fn install(
            &self,
            _request: retaia_agent::DaemonInstallRequest,
        ) -> Result<(), DaemonManagerError> {
            Err(DaemonManagerError::OperationFailed(
                "install is not exposed by desktop shell".to_string(),
            ))
        }

        fn uninstall(&self, _request: DaemonLabelRequest) -> Result<(), DaemonManagerError> {
            Err(DaemonManagerError::OperationFailed(
                "uninstall is not exposed by desktop shell".to_string(),
            ))
        }

        fn start(&self, request: DaemonLabelRequest) -> Result<(), DaemonManagerError> {
            let label = ServiceLabel::from_str(&request.label)
                .map_err(|error| DaemonManagerError::InvalidLabel(error.to_string()))?;
            Self::with_manager(request.level, |manager| {
                manager
                    .start(ServiceStartCtx { label })
                    .map_err(|error| DaemonManagerError::OperationFailed(error.to_string()))
            })
        }

        fn stop(&self, request: DaemonLabelRequest) -> Result<(), DaemonManagerError> {
            let label = ServiceLabel::from_str(&request.label)
                .map_err(|error| DaemonManagerError::InvalidLabel(error.to_string()))?;
            Self::with_manager(request.level, |manager| {
                manager
                    .stop(ServiceStopCtx { label })
                    .map_err(|error| DaemonManagerError::OperationFailed(error.to_string()))
            })
        }

        fn status(&self, request: DaemonLabelRequest) -> Result<DaemonStatus, DaemonManagerError> {
            let label = ServiceLabel::from_str(&request.label)
                .map_err(|error| DaemonManagerError::InvalidLabel(error.to_string()))?;
            Self::with_manager(request.level, |manager| {
                manager
                    .status(ServiceStatusCtx { label })
                    .map(|status| match status {
                        service_manager::ServiceStatus::NotInstalled => DaemonStatus::NotInstalled,
                        service_manager::ServiceStatus::Running => DaemonStatus::Running,
                        service_manager::ServiceStatus::Stopped(reason) => {
                            DaemonStatus::Stopped(reason)
                        }
                    })
                    .map_err(|error| DaemonManagerError::OperationFailed(error.to_string()))
            })
        }
    }

    struct TrayHandle {
        _tray: TrayIcon,
        open_window_id: MenuId,
        open_status_id: MenuId,
        open_settings_id: MenuId,
        daemon_toggle_id: MenuId,
        refresh_daemon_id: MenuId,
        quit_id: MenuId,
    }

    impl TrayHandle {
        fn new(lang: Language) -> Result<Self, String> {
            let menu = Menu::new();

            let open_window = MenuItem::new(t(lang, "gui.tray.open_window"), true, None);
            let open_status = MenuItem::new(t(lang, "gui.tray.open_status"), true, None);
            let open_settings = MenuItem::new(t(lang, "gui.tray.open_preferences"), true, None);
            let separator_top = PredefinedMenuItem::separator();
            let daemon_toggle = MenuItem::new(t(lang, "gui.tray.start_stop_daemon"), true, None);
            let refresh_daemon = MenuItem::new(t(lang, "gui.tray.refresh_daemon"), true, None);
            let separator_bottom = PredefinedMenuItem::separator();
            let quit = MenuItem::new(t(lang, "gui.tray.quit"), true, None);

            menu.append(&open_window)
                .map_err(|error| format!("unable to append tray menu item: {error}"))?;
            menu.append(&open_status)
                .map_err(|error| format!("unable to append tray menu item: {error}"))?;
            menu.append(&open_settings)
                .map_err(|error| format!("unable to append tray menu item: {error}"))?;
            menu.append(&separator_top)
                .map_err(|error| format!("unable to append tray menu separator: {error}"))?;
            menu.append(&daemon_toggle)
                .map_err(|error| format!("unable to append tray menu item: {error}"))?;
            menu.append(&refresh_daemon)
                .map_err(|error| format!("unable to append tray menu item: {error}"))?;
            menu.append(&separator_bottom)
                .map_err(|error| format!("unable to append tray menu separator: {error}"))?;
            menu.append(&quit)
                .map_err(|error| format!("unable to append tray menu item: {error}"))?;

            let icon = default_tray_icon()?;
            let tray = TrayIconBuilder::new()
                .with_tooltip("Retaia Agent")
                .with_menu(Box::new(menu))
                .with_icon(icon)
                .with_icon_as_template(cfg!(target_os = "macos"))
                .build()
                .map_err(|error| format!("unable to create tray icon: {error}"))?;

            Ok(Self {
                _tray: tray,
                open_window_id: open_window.id().clone(),
                open_status_id: open_status.id().clone(),
                open_settings_id: open_settings.id().clone(),
                daemon_toggle_id: daemon_toggle.id().clone(),
                refresh_daemon_id: refresh_daemon.id().clone(),
                quit_id: quit.id().clone(),
            })
        }

        fn refresh_tooltip(
            &self,
            daemon_status: Option<&DaemonStatus>,
            stats: Option<&DaemonRuntimeStats>,
        ) {
            let run_state = stats.map(|s| s.run_state.as_str()).unwrap_or("unknown");
            let tooltip = format!(
                "Retaia Agent | run_state={} | daemon={}",
                run_state,
                daemon_status_label(daemon_status)
            );
            let _ = self._tray.set_tooltip(Some(&tooltip));
        }
    }

    enum TrayCommand {
        OpenWindow,
        OpenStatus,
        OpenPreferences,
        ToggleDaemon,
        RefreshDaemon,
        Quit,
    }

    struct ControlCenterApp {
        lang: Language,
        manager: NativeDaemonManager,
        tray: TrayHandle,
        config_path: Option<PathBuf>,
        config: AgentRuntimeConfig,
        daemon_status: Option<DaemonStatus>,
        stats: Option<DaemonRuntimeStats>,
        started_at: Instant,
        status_modal: Option<String>,
        settings_modal: Option<SettingsEditorState>,
        pending_settings_restart: bool,
        info_modal: Option<String>,
        last_error: Option<String>,
        quit_requested: bool,
        notification_sink: retaia_agent::RuntimeNotificationSink,
    }

    impl ControlCenterApp {
        fn new(
            config: AgentRuntimeConfig,
            config_path: Option<PathBuf>,
            lang: Language,
        ) -> Result<Self, String> {
            let tray = TrayHandle::new(lang)?;
            let manager = NativeDaemonManager;
            let mut app = Self {
                lang,
                manager,
                tray,
                config_path,
                config,
                daemon_status: None,
                stats: None,
                started_at: Instant::now(),
                status_modal: None,
                settings_modal: None,
                pending_settings_restart: false,
                info_modal: None,
                last_error: None,
                quit_requested: false,
                notification_sink: select_notification_sink(NotificationSinkProfile::DesktopSystem),
            };
            app.refresh_daemon_status();
            app.refresh_stats();
            app.refresh_tray();
            Ok(app)
        }

        fn daemon_request() -> DaemonLabelRequest {
            DaemonLabelRequest {
                label: DAEMON_LABEL.to_string(),
                level: DaemonLevel::User,
            }
        }

        fn refresh_daemon_status(&mut self) {
            match self.manager.status(Self::daemon_request()) {
                Ok(status) => self.daemon_status = Some(status),
                Err(error) => self.last_error = Some(error.to_string()),
            }
        }

        fn notify(&mut self, notification: SystemNotification) {
            let report = dispatch_notifications(&self.notification_sink, &[notification]);
            if !report.failed.is_empty() {
                self.last_error = Some("desktop notification dispatch failed".to_string());
            }
        }

        fn refresh_stats(&mut self) {
            match load_runtime_stats() {
                Ok(stats) => self.stats = Some(stats),
                Err(RuntimeStatsStoreError::NotFound) => self.stats = None,
                Err(error) => self.last_error = Some(error.to_string()),
            }
        }

        fn refresh_tray(&self) {
            self.tray
                .refresh_tooltip(self.daemon_status.as_ref(), self.stats.as_ref());
        }

        fn daemon_toggle(&mut self) {
            let should_stop = matches!(self.daemon_status, Some(DaemonStatus::Running));
            let command = match self.daemon_status {
                Some(DaemonStatus::Running) => self.manager.stop(Self::daemon_request()),
                _ => self.manager.start(Self::daemon_request()),
            };
            if let Err(error) = command {
                self.last_error = Some(error.to_string());
            } else if should_stop {
                self.notify(SystemNotification::DaemonStopped);
            } else {
                self.notify(SystemNotification::DaemonStarted);
            }
            self.refresh_daemon_status();
            self.refresh_stats();
            self.refresh_tray();
        }

        fn open_status(&mut self) {
            let diagnostics = collect_daemon_diagnostics(
                &self.manager,
                DiagnosticsLimits {
                    history_limit: 50,
                    cycles_limit: 120,
                },
            );
            let history_db = runtime_history_db_path()
                .ok()
                .map(|path| path.display().to_string());
            self.status_modal = Some(render_daemon_inspect(&diagnostics, history_db.as_deref()));
        }

        fn open_preferences(&mut self) {
            self.settings_modal = Some(SettingsEditorState::from_config(&self.config));
        }

        fn save_preferences(&mut self) {
            let Some(form) = self.settings_modal.clone() else {
                return;
            };
            let next = match form.to_config(&self.config) {
                Ok(config) => config,
                Err(error) => {
                    self.last_error = Some(error);
                    self.notify(SystemNotification::SettingsInvalid {
                        reason: "invalid settings".to_string(),
                    });
                    return;
                }
            };

            let save_result = match self.config_path.as_ref() {
                Some(path) => FileConfigRepository::new(path.clone()).save(&next),
                None => SystemConfigRepository.save(&next),
            };
            if let Err(error) = save_result {
                self.last_error = Some(format!("unable to save config: {error}"));
                return;
            }

            self.config = next.clone();
            self.settings_modal = Some(SettingsEditorState::from_config(&next));
            self.notify(SystemNotification::SettingsSaved);
            self.request_daemon_restart_after_settings_change();
        }

        fn request_daemon_restart_after_settings_change(&mut self) {
            let request = Self::daemon_request();
            match self.manager.status(request.clone()) {
                Ok(DaemonStatus::Running) => match self.manager.stop(request) {
                    Ok(()) => {
                        self.pending_settings_restart = true;
                        self.info_modal = Some(
                            "Settings saved. Graceful daemon restart requested (waiting current job)."
                                .to_string(),
                        );
                    }
                    Err(error) => {
                        self.last_error =
                            Some(format!("settings saved but daemon stop failed: {error}"));
                    }
                },
                Ok(_) => {
                    self.info_modal = Some("Settings saved".to_string());
                }
                Err(error) => {
                    self.last_error = Some(format!(
                        "settings saved but daemon status check failed: {error}"
                    ));
                }
            }
            self.refresh_daemon_status();
            self.refresh_stats();
            self.refresh_tray();
        }

        fn tick_pending_restart(&mut self) {
            if !self.pending_settings_restart {
                return;
            }
            match self.manager.status(Self::daemon_request()) {
                Ok(DaemonStatus::Running) => {}
                Ok(_) => {
                    if let Err(error) = self.manager.start(Self::daemon_request()) {
                        self.last_error = Some(format!(
                            "settings restart failed while starting daemon: {error}"
                        ));
                    } else {
                        self.info_modal =
                            Some("Daemon restarted with updated settings".to_string());
                        self.notify(SystemNotification::DaemonStarted);
                    }
                    self.pending_settings_restart = false;
                    self.refresh_daemon_status();
                    self.refresh_stats();
                    self.refresh_tray();
                }
                Err(error) => {
                    self.last_error =
                        Some(format!("settings restart status check failed: {error}"));
                    self.pending_settings_restart = false;
                }
            }
        }

        fn process_tray_events(&mut self, ctx: &egui::Context) {
            while let Ok(event) = MenuEvent::receiver().try_recv() {
                if let Some(command) = self.map_tray_event(event.id) {
                    match command {
                        TrayCommand::OpenWindow => {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                        }
                        TrayCommand::OpenStatus => self.open_status(),
                        TrayCommand::OpenPreferences => self.open_preferences(),
                        TrayCommand::ToggleDaemon => self.daemon_toggle(),
                        TrayCommand::RefreshDaemon => {
                            self.refresh_daemon_status();
                            self.refresh_stats();
                            self.refresh_tray();
                            self.notify(SystemNotification::DaemonStatusRefreshed {
                                status: daemon_status_label(self.daemon_status.as_ref())
                                    .to_string(),
                            });
                        }
                        TrayCommand::Quit => self.quit_requested = true,
                    }
                }
            }
        }

        fn map_tray_event(&self, id: MenuId) -> Option<TrayCommand> {
            if id == self.tray.open_window_id {
                return Some(TrayCommand::OpenWindow);
            }
            if id == self.tray.open_status_id {
                return Some(TrayCommand::OpenStatus);
            }
            if id == self.tray.open_settings_id {
                return Some(TrayCommand::OpenPreferences);
            }
            if id == self.tray.daemon_toggle_id {
                return Some(TrayCommand::ToggleDaemon);
            }
            if id == self.tray.refresh_daemon_id {
                return Some(TrayCommand::RefreshDaemon);
            }
            if id == self.tray.quit_id {
                return Some(TrayCommand::Quit);
            }
            None
        }

        fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
            if ctx.input(|i| i.key_pressed(egui::Key::S)) {
                self.open_status();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::C)) {
                self.open_preferences();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::D)) {
                self.daemon_toggle();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::R)) {
                self.refresh_daemon_status();
                self.refresh_stats();
                self.refresh_tray();
                self.notify(SystemNotification::DaemonStatusRefreshed {
                    status: daemon_status_label(self.daemon_status.as_ref()).to_string(),
                });
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Q)) {
                self.quit_requested = true;
            }
            if ctx.input(|i| i.key_pressed(egui::Key::B)) {
                self.copy_bug_report();
            }
            if ctx.input(|i| i.key_pressed(egui::Key::J)) {
                self.copy_diagnostics_json();
            }
        }

        fn daemon_toggle_label(&self) -> &'static str {
            match self.daemon_status {
                Some(DaemonStatus::Running) => t(self.lang, "gui.button.stop_daemon"),
                _ => t(self.lang, "gui.button.start_daemon"),
            }
        }

        fn copy_bug_report(&mut self) {
            let diagnostics = collect_daemon_diagnostics(
                &self.manager,
                DiagnosticsLimits {
                    history_limit: 50,
                    cycles_limit: 120,
                },
            );
            let history_db = runtime_history_db_path()
                .ok()
                .map(|path| path.display().to_string());
            let markdown = build_bug_report_markdown(
                &diagnostics,
                None,
                DAEMON_STATS_FILE_NAME,
                history_db.as_deref(),
            );
            let payload = format!("title={}\n\n{}", markdown.title, markdown.body);
            match copy_to_clipboard(&payload) {
                Ok(()) => {
                    self.info_modal = Some(t(self.lang, "gui.info.report_copied").to_string());
                }
                Err(error) => self.last_error = Some(error.to_string()),
            }
        }

        fn copy_diagnostics_json(&mut self) {
            let diagnostics = collect_daemon_diagnostics(
                &self.manager,
                DiagnosticsLimits {
                    history_limit: 50,
                    cycles_limit: 120,
                },
            );
            let history_db = runtime_history_db_path()
                .ok()
                .map(|path| path.display().to_string());
            let redacted_config = redacted_runtime_config_from(&self.config);
            let payload = render_daemon_inspect_json(
                &diagnostics,
                history_db.as_deref(),
                Some(&redacted_config),
            );
            match copy_to_clipboard(&payload) {
                Ok(()) => {
                    self.info_modal = Some(t(self.lang, "gui.info.diagnostics_copied").to_string());
                }
                Err(error) => self.last_error = Some(error.to_string()),
            }
        }
    }

    impl eframe::App for ControlCenterApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            self.process_tray_events(ctx);
            self.handle_keyboard_shortcuts(ctx);
            self.tick_pending_restart();

            if ctx.input(|i| i.viewport().close_requested()) {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            }

            egui::TopBottomPanel::top("top").show(ctx, |ui| {
                ui.heading(t(self.lang, "gui.title"));
                ui.horizontal(|ui| {
                    let run_state = self
                        .stats
                        .as_ref()
                        .map(|stats| stats.run_state.as_str())
                        .unwrap_or("unknown");
                    ui.label(format!("{}: {run_state}", t(self.lang, "gui.run_state")));
                    ui.separator();
                    ui.label(format!(
                        "{}: {}",
                        t(self.lang, "gui.daemon"),
                        daemon_status_label(self.daemon_status.as_ref())
                    ));
                    ui.separator();
                    ui.label(format!(
                        "{}: {}",
                        t(self.lang, "gui.ui_uptime"),
                        format_duration(self.started_at.elapsed())
                    ));
                });
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.columns(2, |columns| {
                    columns[0].group(|ui| {
                        ui.heading(t(self.lang, "gui.controls"));
                        if ui.button(self.daemon_toggle_label()).clicked() {
                            self.daemon_toggle();
                        }
                        if ui
                            .button(t(self.lang, "gui.button.refresh_daemon"))
                            .clicked()
                        {
                            self.refresh_daemon_status();
                            self.refresh_stats();
                            self.refresh_tray();
                            self.notify(SystemNotification::DaemonStatusRefreshed {
                                status: daemon_status_label(self.daemon_status.as_ref())
                                    .to_string(),
                            });
                        }
                        if ui.button(t(self.lang, "gui.button.open_status")).clicked() {
                            self.open_status();
                        }
                        if ui
                            .button(t(self.lang, "gui.button.copy_bug_report"))
                            .clicked()
                        {
                            self.copy_bug_report();
                        }
                        if ui
                            .button(t(self.lang, "gui.button.copy_diagnostics_json"))
                            .clicked()
                        {
                            self.copy_diagnostics_json();
                        }
                        if ui
                            .button(t(self.lang, "gui.button.open_preferences"))
                            .clicked()
                        {
                            self.open_preferences();
                        }
                        if ui.button(t(self.lang, "gui.button.hide_to_tray")).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                        }
                        if ui.button(t(self.lang, "gui.button.quit")).clicked() {
                            self.quit_requested = true;
                        }
                    });

                    columns[1].group(|ui| {
                        ui.heading(t(self.lang, "gui.stats"));
                        if let Some(stats) = self.stats.as_ref() {
                            ui.label(format!(
                                "{}: {}",
                                t(self.lang, "gui.updated"),
                                stats.updated_at_unix_ms
                            ));
                            ui.label(format!("{}: {}", t(self.lang, "gui.tick"), stats.tick));
                            match stats.current_job.as_ref() {
                                Some(job) => {
                                    ui.label(format!(
                                        "{}: {}",
                                        t(self.lang, "gui.current_job"),
                                        job.job_id
                                    ));
                                    ui.label(format!(
                                        "{}: {}",
                                        t(self.lang, "gui.asset"),
                                        job.asset_uuid
                                    ));
                                    ui.label(format!(
                                        "{}: {}%",
                                        t(self.lang, "gui.progress"),
                                        job.progress_percent
                                    ));
                                    ui.label(format!(
                                        "{}: {}",
                                        t(self.lang, "gui.stage"),
                                        job.stage
                                    ));
                                    ui.label(format!(
                                        "{}: {}",
                                        t(self.lang, "gui.status"),
                                        job.status
                                    ));
                                    let elapsed_ms =
                                        now_ms().saturating_sub(job.started_at_unix_ms);
                                    ui.label(format!(
                                        "{}: {}",
                                        t(self.lang, "gui.current_job_duration"),
                                        format_duration(Duration::from_millis(elapsed_ms))
                                    ));
                                }
                                None => {
                                    ui.label(format!(
                                        "{}: {}",
                                        t(self.lang, "gui.current_job"),
                                        t(self.lang, "gui.idle")
                                    ));
                                    ui.label(format!("{}: -", t(self.lang, "gui.asset")));
                                    ui.label(format!("{}: -", t(self.lang, "gui.progress")));
                                    ui.label(format!("{}: -", t(self.lang, "gui.stage")));
                                    ui.label(format!(
                                        "{}: {}",
                                        t(self.lang, "gui.status"),
                                        t(self.lang, "gui.idle")
                                    ));
                                }
                            }
                            ui.separator();
                            ui.heading(t(self.lang, "gui.last_job"));
                            if let Some(last) = stats.last_job.as_ref() {
                                ui.label(format!(
                                    "{}: {}",
                                    t(self.lang, "gui.last_job_id"),
                                    last.job_id
                                ));
                                ui.label(format!(
                                    "{}: {}",
                                    t(self.lang, "gui.duration"),
                                    format_duration(Duration::from_millis(last.duration_ms))
                                ));
                                ui.label(format!(
                                    "{}: {}",
                                    t(self.lang, "gui.completed_at"),
                                    last.completed_at_unix_ms
                                ));
                            } else {
                                ui.label(format!("{}: -", t(self.lang, "gui.last_job_id")));
                                ui.label(format!("{}: -", t(self.lang, "gui.duration")));
                                ui.label(format!("{}: -", t(self.lang, "gui.completed_at")));
                            }
                        } else {
                            ui.label(t(self.lang, "gui.no_stats"));
                            ui.label(t(self.lang, "gui.start_daemon_hint"));
                        }
                    });
                });

                ui.separator();
                ui.label(t(self.lang, "gui.shortcuts"));
            });

            if let Some(content) = self.status_modal.as_mut() {
                let mut open = true;
                egui::Window::new(t(self.lang, "gui.modal.status"))
                    .open(&mut open)
                    .resizable(true)
                    .show(ctx, |ui| {
                        ui.code(content.as_str());
                    });
                if !open {
                    self.status_modal = None;
                }
            }

            if let Some(form) = self.settings_modal.as_mut() {
                let mut open = true;
                let mut should_save = false;
                let mut should_reload = false;
                egui::Window::new(t(self.lang, "gui.modal.preferences"))
                    .open(&mut open)
                    .resizable(true)
                    .show(ctx, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("core_api_url");
                            ui.text_edit_singleline(&mut form.core_api_url);
                        });
                        ui.horizontal(|ui| {
                            ui.label("ollama_url");
                            ui.text_edit_singleline(&mut form.ollama_url);
                        });
                        ui.horizontal(|ui| {
                            ui.label("auth_mode");
                            egui::ComboBox::from_id_salt("auth_mode_combo")
                                .selected_text(match form.auth_mode {
                                    AuthMode::Interactive => "interactive",
                                    AuthMode::Technical => "technical",
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut form.auth_mode,
                                        AuthMode::Interactive,
                                        "interactive",
                                    );
                                    ui.selectable_value(
                                        &mut form.auth_mode,
                                        AuthMode::Technical,
                                        "technical",
                                    );
                                });
                        });
                        if matches!(form.auth_mode, AuthMode::Technical) {
                            ui.horizontal(|ui| {
                                ui.label("client_id");
                                ui.text_edit_singleline(&mut form.technical_client_id);
                            });
                            ui.horizontal(|ui| {
                                ui.label("secret_key");
                                ui.add(
                                    egui::TextEdit::singleline(&mut form.technical_secret_key)
                                        .password(true),
                                );
                            });
                            ui.small("Leave secret_key empty to keep current value.");
                        }
                        ui.horizontal(|ui| {
                            ui.label("max_parallel_jobs");
                            ui.text_edit_singleline(&mut form.max_parallel_jobs);
                        });
                        ui.horizontal(|ui| {
                            ui.label("log_level");
                            egui::ComboBox::from_id_salt("log_level_combo")
                                .selected_text(match form.log_level {
                                    LogLevel::Error => "error",
                                    LogLevel::Warn => "warn",
                                    LogLevel::Info => "info",
                                    LogLevel::Debug => "debug",
                                    LogLevel::Trace => "trace",
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut form.log_level,
                                        LogLevel::Error,
                                        "error",
                                    );
                                    ui.selectable_value(
                                        &mut form.log_level,
                                        LogLevel::Warn,
                                        "warn",
                                    );
                                    ui.selectable_value(
                                        &mut form.log_level,
                                        LogLevel::Info,
                                        "info",
                                    );
                                    ui.selectable_value(
                                        &mut form.log_level,
                                        LogLevel::Debug,
                                        "debug",
                                    );
                                    ui.selectable_value(
                                        &mut form.log_level,
                                        LogLevel::Trace,
                                        "trace",
                                    );
                                });
                        });
                        ui.separator();
                        ui.label("storage_mounts (one per line: storage_id=/absolute/path)");
                        ui.add_sized(
                            [560.0, 140.0],
                            egui::TextEdit::multiline(&mut form.storage_mounts),
                        );
                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button("Save").clicked() {
                                should_save = true;
                            }
                            if ui.button("Reload").clicked() {
                                should_reload = true;
                            }
                        });
                    });
                if !open {
                    self.settings_modal = None;
                }
                if should_reload {
                    self.settings_modal = Some(SettingsEditorState::from_config(&self.config));
                }
                if should_save {
                    self.save_preferences();
                }
            }

            if let Some(info) = self.info_modal.as_mut() {
                let mut open = true;
                egui::Window::new(t(self.lang, "gui.modal.info"))
                    .open(&mut open)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(info.as_str());
                    });
                if !open {
                    self.info_modal = None;
                }
            }

            if let Some(error) = self.last_error.as_mut() {
                let mut open = true;
                egui::Window::new(t(self.lang, "gui.modal.error"))
                    .open(&mut open)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(error.as_str());
                    });
                if !open {
                    self.last_error = None;
                }
            }

            if self.quit_requested {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            ctx.request_repaint_after(Duration::from_millis(500));
        }
    }

    fn load_settings(config_path: Option<PathBuf>) -> Result<AgentRuntimeConfig, String> {
        match config_path {
            Some(path) => FileConfigRepository::new(path)
                .load()
                .map_err(|error| format!("unable to load config: {error}")),
            None => SystemConfigRepository
                .load()
                .map_err(|error| format!("unable to load config: {error}")),
        }
    }

    #[derive(Debug, Clone)]
    struct SettingsEditorState {
        core_api_url: String,
        ollama_url: String,
        auth_mode: AuthMode,
        technical_client_id: String,
        technical_secret_key: String,
        storage_mounts: String,
        max_parallel_jobs: String,
        log_level: LogLevel,
    }

    impl SettingsEditorState {
        fn from_config(config: &AgentRuntimeConfig) -> Self {
            let storage_mounts = config
                .storage_mounts
                .iter()
                .map(|(storage_id, path)| format!("{storage_id}={path}"))
                .collect::<Vec<_>>()
                .join("\n");
            Self {
                core_api_url: config.core_api_url.clone(),
                ollama_url: config.ollama_url.clone(),
                auth_mode: config.auth_mode,
                technical_client_id: config
                    .technical_auth
                    .as_ref()
                    .map(|value| value.client_id.clone())
                    .unwrap_or_default(),
                technical_secret_key: String::new(),
                storage_mounts,
                max_parallel_jobs: config.max_parallel_jobs.to_string(),
                log_level: config.log_level,
            }
        }

        fn to_config(&self, current: &AgentRuntimeConfig) -> Result<AgentRuntimeConfig, String> {
            let max_parallel_jobs = self
                .max_parallel_jobs
                .trim()
                .parse::<u16>()
                .map_err(|_| "max_parallel_jobs must be a positive integer".to_string())?;
            let storage_mounts = parse_storage_mounts_text(&self.storage_mounts)?;

            let technical_auth = match self.auth_mode {
                AuthMode::Interactive => None,
                AuthMode::Technical => {
                    let secret_key = if self.technical_secret_key.trim().is_empty() {
                        current
                            .technical_auth
                            .as_ref()
                            .map(|value| value.secret_key.clone())
                            .unwrap_or_default()
                    } else {
                        self.technical_secret_key.clone()
                    };
                    Some(retaia_agent::TechnicalAuthConfig {
                        client_id: self.technical_client_id.clone(),
                        secret_key,
                    })
                }
            };

            let config = AgentRuntimeConfig {
                core_api_url: normalize_core_api_url(&self.core_api_url),
                ollama_url: self.ollama_url.clone(),
                auth_mode: self.auth_mode,
                technical_auth,
                storage_mounts,
                max_parallel_jobs,
                log_level: self.log_level,
            };
            validate_config(&config)
                .map_err(|errors| compact_validation_reason(&errors))
                .map(|_| config)
        }
    }

    fn parse_storage_mounts_text(
        value: &str,
    ) -> Result<std::collections::BTreeMap<String, String>, String> {
        let mut mounts = std::collections::BTreeMap::new();
        for line in value.lines() {
            let entry = line.trim();
            if entry.is_empty() {
                continue;
            }
            let Some((storage_id, raw_path)) = entry.split_once('=') else {
                return Err(format!(
                    "invalid storage mount '{entry}' (expected storage_id=/absolute/path)"
                ));
            };
            let storage_id = storage_id.trim();
            if storage_id.is_empty() {
                return Err("invalid storage mount: storage_id must not be empty".to_string());
            }
            let path = normalize_storage_mount_path(raw_path);
            if path.is_empty() {
                return Err(format!(
                    "invalid storage mount '{entry}' (mount path must not be empty)"
                ));
            }
            mounts.insert(storage_id.to_string(), path);
        }
        Ok(mounts)
    }

    fn format_duration(duration: Duration) -> String {
        let total = duration.as_secs();
        let hours = total / 3600;
        let minutes = (total % 3600) / 60;
        let seconds = total % 60;
        if hours > 0 {
            format!("{hours}h {minutes:02}m {seconds:02}s")
        } else if minutes > 0 {
            format!("{minutes}m {seconds:02}s")
        } else {
            format!("{seconds}s")
        }
    }

    fn now_ms() -> u64 {
        now_unix_ms()
    }

    fn default_tray_icon() -> Result<Icon, String> {
        #[cfg(target_os = "macos")]
        const TRAY_ICON_BYTES: &[u8] =
            include_bytes!("../../assets/icon/retaia-tray-macos-template.png");
        #[cfg(not(target_os = "macos"))]
        const TRAY_ICON_BYTES: &[u8] = include_bytes!("../../assets/icon/retaia-tray-default.png");
        if let Ok(decoded) = image::load_from_memory(TRAY_ICON_BYTES) {
            let rgba = decoded.into_rgba8();
            return Icon::from_rgba(rgba.to_vec(), rgba.width(), rgba.height())
                .map_err(|error| format!("unable to build tray icon image: {error}"));
        }

        const WIDTH: u32 = 32;
        const HEIGHT: u32 = 32;
        let mut rgba = vec![0_u8; (WIDTH * HEIGHT * 4) as usize];
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let idx = ((y * WIDTH + x) * 4) as usize;
                let edge = x == 0 || y == 0 || x == WIDTH - 1 || y == HEIGHT - 1;
                let diagonal = x == y || x + y == WIDTH - 1;
                if edge || diagonal {
                    rgba[idx] = 18;
                    rgba[idx + 1] = 52;
                    rgba[idx + 2] = 86;
                    rgba[idx + 3] = 255;
                } else {
                    rgba[idx] = 244;
                    rgba[idx + 1] = 244;
                    rgba[idx + 2] = 244;
                    rgba[idx + 3] = 255;
                }
            }
        }
        Icon::from_rgba(rgba, WIDTH, HEIGHT)
            .map_err(|error| format!("unable to build tray icon image: {error}"))
    }

    fn daemon_status_label(status: Option<&DaemonStatus>) -> &'static str {
        match status {
            Some(DaemonStatus::Running) => "running",
            Some(DaemonStatus::NotInstalled) => "not-installed",
            Some(DaemonStatus::Stopped(_)) => "stopped",
            None => "unknown",
        }
    }

    fn run() -> Result<(), String> {
        let lang = detect_language();
        let cli = Cli::parse();
        let settings = load_settings(cli.config.clone())?;

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title(t(lang, "gui.title"))
                .with_inner_size([960.0, 600.0]),
            ..Default::default()
        };

        eframe::run_native(
            t(lang, "gui.title"),
            options,
            Box::new(move |_cc| Ok(Box::new(ControlCenterApp::new(settings, cli.config, lang)?))),
        )
        .map_err(|error| error.to_string())
    }

    pub fn main() {
        if let Err(error) = run() {
            eprintln!("{error}");
            exit(1);
        }
    }
}

#[cfg(feature = "desktop-shell")]
fn main() {
    desktop_shell::main();
}
