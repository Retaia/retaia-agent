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
        AgentRuntimeConfig, ConfigRepository, DaemonLabelRequest, DaemonLevel, DaemonManager,
        DaemonManagerError, DaemonStatus, DesktopShellBridge, DesktopShellController,
        FileConfigRepository, GuiMenuAction, GuiMenuView, RuntimeSession, SystemConfigRepository,
        compact_validation_reason,
    };
    use service_manager::{
        ServiceLabel, ServiceLevel, ServiceStartCtx, ServiceStatusCtx, ServiceStopCtx,
    };
    use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem};
    use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

    #[derive(Debug, Parser)]
    #[command(
        name = "agent-desktop-shell",
        about = "Retaia desktop shell (tray + control center window)"
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
        play_resume: MenuItem,
        pause: MenuItem,
        stop: MenuItem,
        daemon_toggle_id: MenuId,
        refresh_daemon_id: MenuId,
        quit_id: MenuId,
    }

    impl TrayHandle {
        fn new() -> Result<Self, String> {
            let menu = Menu::new();

            let open_window = MenuItem::new("Open Window", true, None);
            let open_status = MenuItem::new("Open Status", true, None);
            let open_settings = MenuItem::new("Open Preferences", true, None);
            let separator_top = PredefinedMenuItem::separator();
            let play_resume = MenuItem::new("Play/Resume", true, None);
            let pause = MenuItem::new("Pause", true, None);
            let stop = MenuItem::new("Stop", true, None);
            let separator_mid = PredefinedMenuItem::separator();
            let daemon_toggle = MenuItem::new("Start/Stop Daemon", true, None);
            let refresh_daemon = MenuItem::new("Refresh Daemon Status", true, None);
            let separator_bottom = PredefinedMenuItem::separator();
            let quit = MenuItem::new("Quit", true, None);

            menu.append(&open_window)
                .map_err(|error| format!("unable to append tray menu item: {error}"))?;
            menu.append(&open_status)
                .map_err(|error| format!("unable to append tray menu item: {error}"))?;
            menu.append(&open_settings)
                .map_err(|error| format!("unable to append tray menu item: {error}"))?;
            menu.append(&separator_top)
                .map_err(|error| format!("unable to append tray menu separator: {error}"))?;
            menu.append(&play_resume)
                .map_err(|error| format!("unable to append tray menu item: {error}"))?;
            menu.append(&pause)
                .map_err(|error| format!("unable to append tray menu item: {error}"))?;
            menu.append(&stop)
                .map_err(|error| format!("unable to append tray menu item: {error}"))?;
            menu.append(&separator_mid)
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
                play_resume,
                pause,
                stop,
                daemon_toggle_id: daemon_toggle.id().clone(),
                refresh_daemon_id: refresh_daemon.id().clone(),
                quit_id: quit.id().clone(),
            })
        }

        fn refresh_from_view(&self, view: &GuiMenuView) {
            let _ = self.play_resume.set_enabled(view.can_play_resume);
            let _ = self.pause.set_enabled(view.can_pause);
            let _ = self.stop.set_enabled(view.can_stop);
            let tooltip = format!(
                "Retaia Agent | state={:?} | daemon={}",
                view.run_state,
                daemon_status_label(view.daemon_status.as_ref())
            );
            let _ = self._tray.set_tooltip(Some(&tooltip));
        }
    }

    enum TrayCommand {
        OpenWindow,
        Gui(GuiMenuAction),
        ToggleDaemon,
        Quit,
    }

    struct UiBridge {
        latest_view: Option<GuiMenuView>,
        status_content: Option<String>,
        settings_content: Option<String>,
        quit_requested: bool,
    }

    impl UiBridge {
        fn new() -> Self {
            Self {
                latest_view: None,
                status_content: None,
                settings_content: None,
                quit_requested: false,
            }
        }

        fn take_status_content(&mut self) -> Option<String> {
            self.status_content.take()
        }

        fn take_settings_content(&mut self) -> Option<String> {
            self.settings_content.take()
        }
    }

    impl DesktopShellBridge for UiBridge {
        fn render_menu(&mut self, view: &GuiMenuView) {
            self.latest_view = Some(view.clone());
        }

        fn open_status_window(&mut self, content: &str) {
            self.status_content = Some(content.to_string());
        }

        fn open_settings_panel(&mut self, content: &str) {
            self.settings_content = Some(content.to_string());
        }

        fn request_quit(&mut self) {
            self.quit_requested = true;
        }
    }

    struct ControlCenterApp {
        controller: DesktopShellController<NativeDaemonManager>,
        bridge: UiBridge,
        tray: TrayHandle,
        started_at: Instant,
        current_job_id: Option<String>,
        current_job_started_at: Option<Instant>,
        last_job_id: Option<String>,
        last_job_duration: Option<Duration>,
        status_modal: Option<String>,
        settings_modal: Option<String>,
        last_error: Option<String>,
    }

    impl ControlCenterApp {
        fn new(settings: AgentRuntimeConfig) -> Result<Self, String> {
            let session = RuntimeSession::new(retaia_agent::ClientRuntimeTarget::Agent, settings)
                .map_err(|errors| compact_validation_reason(&errors))?;
            let controller =
                DesktopShellController::with_default_user_daemon(session, NativeDaemonManager);
            let tray = TrayHandle::new()?;
            let mut app = Self {
                controller,
                bridge: UiBridge::new(),
                tray,
                started_at: Instant::now(),
                current_job_id: None,
                current_job_started_at: None,
                last_job_id: None,
                last_job_duration: None,
                status_modal: None,
                settings_modal: None,
                last_error: None,
            };
            app.controller.render_initial_menu(&mut app.bridge);
            app.sync_modal_content();
            app.refresh_tray();
            Ok(app)
        }

        fn refresh_tray(&self) {
            if let Some(view) = self.bridge.latest_view.as_ref() {
                self.tray.refresh_from_view(view);
            }
        }

        fn sync_modal_content(&mut self) {
            if let Some(content) = self.bridge.take_status_content() {
                self.status_modal = Some(content);
            }
            if let Some(content) = self.bridge.take_settings_content() {
                self.settings_modal = Some(content);
            }
        }

        fn handle_gui_action(&mut self, action: GuiMenuAction) {
            let result = self.controller.handle_action(action, &mut self.bridge);
            if let Err(error) = result {
                self.last_error = Some(error.to_string());
            }
            self.sync_modal_content();
            self.refresh_tray();
        }

        fn handle_tray_command(&mut self, command: TrayCommand, ctx: &egui::Context) {
            match command {
                TrayCommand::OpenWindow => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                }
                TrayCommand::Gui(action) => self.handle_gui_action(action),
                TrayCommand::ToggleDaemon => {
                    let action = match self.controller.daemon_status() {
                        Some(DaemonStatus::Running) => GuiMenuAction::StopDaemon,
                        _ => GuiMenuAction::StartDaemon,
                    };
                    self.handle_gui_action(action);
                }
                TrayCommand::Quit => {
                    self.bridge.quit_requested = true;
                }
            }
        }

        fn process_tray_events(&mut self, ctx: &egui::Context) {
            while let Ok(event) = MenuEvent::receiver().try_recv() {
                if let Some(command) = self.map_tray_event(event.id) {
                    self.handle_tray_command(command, ctx);
                }
            }
        }

        fn map_tray_event(&self, id: MenuId) -> Option<TrayCommand> {
            if id == self.tray.open_window_id {
                return Some(TrayCommand::OpenWindow);
            }
            if id == self.tray.open_status_id {
                return Some(TrayCommand::Gui(GuiMenuAction::OpenStatusWindow));
            }
            if id == self.tray.open_settings_id {
                return Some(TrayCommand::Gui(GuiMenuAction::OpenSettings));
            }
            if id == *self.tray.play_resume.id() {
                return Some(TrayCommand::Gui(GuiMenuAction::PlayResume));
            }
            if id == *self.tray.pause.id() {
                return Some(TrayCommand::Gui(GuiMenuAction::Pause));
            }
            if id == *self.tray.stop.id() {
                return Some(TrayCommand::Gui(GuiMenuAction::Stop));
            }
            if id == self.tray.daemon_toggle_id {
                return Some(TrayCommand::ToggleDaemon);
            }
            if id == self.tray.refresh_daemon_id {
                return Some(TrayCommand::Gui(GuiMenuAction::RefreshDaemonStatus));
            }
            if id == self.tray.quit_id {
                return Some(TrayCommand::Quit);
            }
            None
        }

        fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
            if ctx.input(|i| i.key_pressed(egui::Key::S)) {
                self.handle_gui_action(GuiMenuAction::OpenStatusWindow);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::C)) {
                self.handle_gui_action(GuiMenuAction::OpenSettings);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::P)) {
                self.handle_gui_action(GuiMenuAction::PlayResume);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::A)) {
                self.handle_gui_action(GuiMenuAction::Pause);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::X)) {
                self.handle_gui_action(GuiMenuAction::Stop);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::D)) {
                let action = match self.controller.daemon_status() {
                    Some(DaemonStatus::Running) => GuiMenuAction::StopDaemon,
                    _ => GuiMenuAction::StartDaemon,
                };
                self.handle_gui_action(action);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::R)) {
                self.handle_gui_action(GuiMenuAction::RefreshDaemonStatus);
            }
            if ctx.input(|i| i.key_pressed(egui::Key::Q)) {
                self.bridge.quit_requested = true;
            }
        }

        fn track_jobs(&mut self) {
            let view = self.controller.session().status_view();
            match view.current_job {
                Some(job) => match self.current_job_id.as_deref() {
                    Some(active_id) if active_id == job.job_id => {}
                    Some(previous_id) => {
                        if let Some(started) = self.current_job_started_at.take() {
                            self.last_job_id = Some(previous_id.to_string());
                            self.last_job_duration = Some(started.elapsed());
                        }
                        self.current_job_id = Some(job.job_id);
                        self.current_job_started_at = Some(Instant::now());
                    }
                    None => {
                        self.current_job_id = Some(job.job_id);
                        self.current_job_started_at = Some(Instant::now());
                    }
                },
                None => {
                    if let Some(previous_id) = self.current_job_id.take()
                        && let Some(started) = self.current_job_started_at.take()
                    {
                        self.last_job_id = Some(previous_id);
                        self.last_job_duration = Some(started.elapsed());
                    }
                }
            }
        }

        fn daemon_toggle_label(&self) -> &'static str {
            match self.controller.daemon_status() {
                Some(DaemonStatus::Running) => "Stop Daemon",
                _ => "Start Daemon",
            }
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

            self.track_jobs();
            let view = self.bridge.latest_view.clone().unwrap_or_else(|| {
                retaia_agent::menu_view(
                    self.controller.session(),
                    self.controller.daemon_status().cloned(),
                )
            });

            egui::TopBottomPanel::top("top").show(ctx, |ui| {
                ui.heading("Retaia Agent Control Center");
                ui.horizontal(|ui| {
                    ui.label(format!("Run state: {:?}", view.run_state));
                    ui.separator();
                    ui.label(format!(
                        "Daemon: {}",
                        daemon_status_label(view.daemon_status.as_ref())
                    ));
                    ui.separator();
                    ui.label(format!(
                        "Uptime: {}",
                        format_duration(self.started_at.elapsed())
                    ));
                });
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                ui.columns(2, |columns| {
                    columns[0].group(|ui| {
                        ui.heading("Controls");
                        ui.add_enabled_ui(view.can_play_resume, |ui| {
                            if ui.button("Play / Resume").clicked() {
                                self.handle_gui_action(GuiMenuAction::PlayResume);
                            }
                        });
                        ui.add_enabled_ui(view.can_pause, |ui| {
                            if ui.button("Pause").clicked() {
                                self.handle_gui_action(GuiMenuAction::Pause);
                            }
                        });
                        ui.add_enabled_ui(view.can_stop, |ui| {
                            if ui.button("Stop").clicked() {
                                self.handle_gui_action(GuiMenuAction::Stop);
                            }
                        });
                        if ui.button(self.daemon_toggle_label()).clicked() {
                            let action = match self.controller.daemon_status() {
                                Some(DaemonStatus::Running) => GuiMenuAction::StopDaemon,
                                _ => GuiMenuAction::StartDaemon,
                            };
                            self.handle_gui_action(action);
                        }
                        if ui.button("Refresh Daemon Status").clicked() {
                            self.handle_gui_action(GuiMenuAction::RefreshDaemonStatus);
                        }
                        if ui.button("Open Status").clicked() {
                            self.handle_gui_action(GuiMenuAction::OpenStatusWindow);
                        }
                        if ui.button("Open Preferences").clicked() {
                            self.handle_gui_action(GuiMenuAction::OpenSettings);
                        }
                        if ui.button("Hide to Tray").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                        }
                        if ui.button("Quit").clicked() {
                            self.bridge.quit_requested = true;
                        }
                    });

                    columns[1].group(|ui| {
                        ui.heading("Runtime Stats");
                        let status = self.controller.session().status_view();
                        match status.current_job {
                            Some(job) => {
                                ui.label(format!("Current job: {}", job.job_id));
                                ui.label(format!("Asset: {}", job.asset_uuid));
                                ui.label(format!("Progress: {}%", job.progress_percent));
                                ui.label(format!("Stage: {:?}", job.stage));
                                ui.label(format!("Status: {}", job.short_status));
                                if let Some(started) = self.current_job_started_at {
                                    ui.label(format!(
                                        "Current job duration: {}",
                                        format_duration(started.elapsed())
                                    ));
                                }
                            }
                            None => {
                                ui.label("Current job: idle");
                                ui.label("Asset: -");
                                ui.label("Progress: -");
                                ui.label("Stage: -");
                                ui.label("Status: idle");
                            }
                        }

                        ui.separator();
                        ui.heading("Last Job");
                        ui.label(format!(
                            "Last job id: {}",
                            self.last_job_id.as_deref().unwrap_or("-")
                        ));
                        ui.label(format!(
                            "Duration: {}",
                            self.last_job_duration
                                .map(format_duration)
                                .unwrap_or_else(|| "-".to_string())
                        ));
                    });
                });

                ui.separator();
                ui.label(
                    "Shortcuts: S status, C prefs, P play, A pause, X stop, D daemon toggle, R refresh daemon, Q quit",
                );
            });

            if let Some(content) = self.status_modal.as_mut() {
                let mut open = true;
                egui::Window::new("Status")
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
                egui::Window::new("Preferences")
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
                egui::Window::new("Error")
                    .open(&mut open)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(error.as_str());
                    });
                if !open {
                    self.last_error = None;
                }
            }

            if self.bridge.quit_requested {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }

            ctx.request_repaint_after(Duration::from_millis(200));
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
        let cli = Cli::parse();
        let settings = load_settings(cli.config)?;

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("Retaia Agent Control Center")
                .with_inner_size([980.0, 620.0]),
            ..Default::default()
        };

        eframe::run_native(
            "Retaia Agent Control Center",
            options,
            Box::new(move |_cc| Ok(Box::new(ControlCenterApp::new(settings)?))),
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
