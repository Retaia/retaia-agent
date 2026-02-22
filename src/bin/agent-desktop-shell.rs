#[cfg(not(feature = "desktop-shell"))]
fn main() {
    eprintln!("agent-desktop-shell requires --features desktop-shell");
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
        AgentRuntimeConfig, AuthMode, ConfigRepository, DaemonLabelRequest, DaemonLevel,
        DaemonManager, DaemonManagerError, DaemonRuntimeStats, DaemonStatus, FileConfigRepository,
        Language, LogLevel, RuntimeStatsStoreError, SystemConfigRepository, detect_language,
        load_runtime_stats, t,
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
        config: AgentRuntimeConfig,
        daemon_status: Option<DaemonStatus>,
        stats: Option<DaemonRuntimeStats>,
        started_at: Instant,
        status_modal: Option<String>,
        settings_modal: Option<String>,
        last_error: Option<String>,
        quit_requested: bool,
    }

    impl ControlCenterApp {
        fn new(config: AgentRuntimeConfig, lang: Language) -> Result<Self, String> {
            let tray = TrayHandle::new(lang)?;
            let manager = NativeDaemonManager;
            let mut app = Self {
                lang,
                manager,
                tray,
                config,
                daemon_status: None,
                stats: None,
                started_at: Instant::now(),
                status_modal: None,
                settings_modal: None,
                last_error: None,
                quit_requested: false,
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
            let command = match self.daemon_status {
                Some(DaemonStatus::Running) => self.manager.stop(Self::daemon_request()),
                _ => self.manager.start(Self::daemon_request()),
            };
            if let Err(error) = command {
                self.last_error = Some(error.to_string());
            }
            self.refresh_daemon_status();
            self.refresh_stats();
            self.refresh_tray();
        }

        fn open_status(&mut self) {
            self.status_modal = Some(self.format_status_modal());
        }

        fn open_preferences(&mut self) {
            self.settings_modal = Some(format_settings(&self.config));
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
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Q)) {
                self.quit_requested = true;
            }
        }

        fn daemon_toggle_label(&self) -> &'static str {
            match self.daemon_status {
                Some(DaemonStatus::Running) => t(self.lang, "gui.button.stop_daemon"),
                _ => t(self.lang, "gui.button.start_daemon"),
            }
        }

        fn format_status_modal(&self) -> String {
            let mut lines = Vec::new();
            lines.push(format!(
                "daemon_status={}",
                daemon_status_label(self.daemon_status.as_ref())
            ));
            if let Some(stats) = self.stats.as_ref() {
                lines.push(format!("updated_at_unix_ms={}", stats.updated_at_unix_ms));
                lines.push(format!("run_state={}", stats.run_state));
                lines.push(format!("tick={}", stats.tick));
                if let Some(job) = stats.current_job.as_ref() {
                    lines.push(format!("current_job_id={}", job.job_id));
                    lines.push(format!("current_asset_uuid={}", job.asset_uuid));
                    lines.push(format!("current_progress_percent={}", job.progress_percent));
                    lines.push(format!("current_stage={}", job.stage));
                    lines.push(format!("current_status={}", job.status));
                    lines.push(format!(
                        "current_started_at_unix_ms={}",
                        job.started_at_unix_ms
                    ));
                } else {
                    lines.push("current_job_id=-".to_string());
                    lines.push("current_asset_uuid=-".to_string());
                    lines.push("current_progress_percent=-".to_string());
                    lines.push("current_stage=-".to_string());
                    lines.push("current_status=idle".to_string());
                    lines.push("current_started_at_unix_ms=-".to_string());
                }
                if let Some(job) = stats.last_job.as_ref() {
                    lines.push(format!("last_job_id={}", job.job_id));
                    lines.push(format!("last_job_duration_ms={}", job.duration_ms));
                    lines.push(format!(
                        "last_job_completed_at_unix_ms={}",
                        job.completed_at_unix_ms
                    ));
                } else {
                    lines.push("last_job_id=-".to_string());
                    lines.push("last_job_duration_ms=-".to_string());
                    lines.push("last_job_completed_at_unix_ms=-".to_string());
                }
            } else {
                lines.push("stats=unavailable".to_string());
            }
            lines.join("\n")
        }
    }

    impl eframe::App for ControlCenterApp {
        fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            self.process_tray_events(ctx);
            self.handle_keyboard_shortcuts(ctx);

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
                        }
                        if ui.button(t(self.lang, "gui.button.open_status")).clicked() {
                            self.open_status();
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

            if let Some(content) = self.settings_modal.as_mut() {
                let mut open = true;
                egui::Window::new(t(self.lang, "gui.modal.preferences"))
                    .open(&mut open)
                    .resizable(true)
                    .show(ctx, |ui| {
                        ui.code(content.as_str());
                    });
                if !open {
                    self.settings_modal = None;
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

    fn format_settings(config: &AgentRuntimeConfig) -> String {
        let auth_mode = match config.auth_mode {
            AuthMode::Interactive => "interactive",
            AuthMode::Technical => "technical",
        };
        let log_level = match config.log_level {
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        };

        [
            format!("core_api_url={}", config.core_api_url),
            format!("ollama_url={}", config.ollama_url),
            format!("auth_mode={auth_mode}"),
            format!(
                "technical_client_id={}",
                config
                    .technical_auth
                    .as_ref()
                    .map(|value| value.client_id.as_str())
                    .unwrap_or("-")
            ),
            format!(
                "technical_secret_key_set={}",
                config.technical_auth.is_some()
            ),
            format!("max_parallel_jobs={}", config.max_parallel_jobs),
            format!("log_level={log_level}"),
        ]
        .join("\n")
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
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(duration) => duration.as_millis() as u64,
            Err(_) => 0,
        }
    }

    fn default_tray_icon() -> Result<Icon, String> {
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
        let settings = load_settings(cli.config)?;

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title(t(lang, "gui.title"))
                .with_inner_size([960.0, 600.0]),
            ..Default::default()
        };

        eframe::run_native(
            t(lang, "gui.title"),
            options,
            Box::new(move |_cc| Ok(Box::new(ControlCenterApp::new(settings, lang)?))),
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
