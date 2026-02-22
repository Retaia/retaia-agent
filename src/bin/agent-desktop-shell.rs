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

    use clap::Parser;
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
    use winit::application::ApplicationHandler;
    use winit::event::{ElementState, WindowEvent};
    use winit::event_loop::{ActiveEventLoop, EventLoop};
    use winit::keyboard::{KeyCode, PhysicalKey};
    use winit::window::{Window, WindowAttributes};

    #[derive(Debug, Parser)]
    #[command(
        name = "agent-desktop-shell",
        about = "Retaia desktop shell (tray + window controls)"
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
        start_daemon: MenuItem,
        stop_daemon: MenuItem,
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
            let start_daemon = MenuItem::new("Start Daemon", true, None);
            let stop_daemon = MenuItem::new("Stop Daemon", true, None);
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
            menu.append(&start_daemon)
                .map_err(|error| format!("unable to append tray menu item: {error}"))?;
            menu.append(&stop_daemon)
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
                start_daemon,
                stop_daemon,
                refresh_daemon_id: refresh_daemon.id().clone(),
                quit_id: quit.id().clone(),
            })
        }

        fn refresh_from_view(&self, view: &GuiMenuView) {
            let _ = self.play_resume.set_enabled(view.can_play_resume);
            let _ = self.pause.set_enabled(view.can_pause);
            let _ = self.stop.set_enabled(view.can_stop);

            match view.daemon_status {
                Some(DaemonStatus::Running) => {
                    let _ = self.start_daemon.set_enabled(false);
                    let _ = self.stop_daemon.set_enabled(true);
                }
                Some(DaemonStatus::NotInstalled) => {
                    let _ = self.start_daemon.set_enabled(false);
                    let _ = self.stop_daemon.set_enabled(false);
                }
                Some(DaemonStatus::Stopped(_)) | None => {
                    let _ = self.start_daemon.set_enabled(true);
                    let _ = self.stop_daemon.set_enabled(false);
                }
            }

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
        Quit,
    }

    struct WindowBridge {
        window: Option<Window>,
        quit_requested: bool,
        tray: Option<TrayHandle>,
    }

    impl WindowBridge {
        fn new() -> Self {
            Self {
                window: None,
                quit_requested: false,
                tray: None,
            }
        }

        fn set_window(&mut self, window: Window) {
            self.window = Some(window);
        }

        fn ensure_tray(&mut self) -> Result<(), String> {
            if self.tray.is_none() {
                self.tray = Some(TrayHandle::new()?);
            }
            Ok(())
        }

        fn show_window(&mut self) {
            if let Some(window) = self.window.as_ref() {
                window.set_visible(true);
                window.request_redraw();
            }
        }

        fn hide_window(&mut self) {
            if let Some(window) = self.window.as_ref() {
                window.set_visible(false);
            }
        }

        fn on_tray_menu_event(&self, event: MenuEvent) -> Option<TrayCommand> {
            let tray = self.tray.as_ref()?;
            if event.id == tray.open_window_id {
                return Some(TrayCommand::OpenWindow);
            }
            if event.id == tray.open_status_id {
                return Some(TrayCommand::Gui(GuiMenuAction::OpenStatusWindow));
            }
            if event.id == tray.open_settings_id {
                return Some(TrayCommand::Gui(GuiMenuAction::OpenSettings));
            }
            if event.id == *tray.play_resume.id() {
                return Some(TrayCommand::Gui(GuiMenuAction::PlayResume));
            }
            if event.id == *tray.pause.id() {
                return Some(TrayCommand::Gui(GuiMenuAction::Pause));
            }
            if event.id == *tray.stop.id() {
                return Some(TrayCommand::Gui(GuiMenuAction::Stop));
            }
            if event.id == *tray.start_daemon.id() {
                return Some(TrayCommand::Gui(GuiMenuAction::StartDaemon));
            }
            if event.id == *tray.stop_daemon.id() {
                return Some(TrayCommand::Gui(GuiMenuAction::StopDaemon));
            }
            if event.id == tray.refresh_daemon_id {
                return Some(TrayCommand::Gui(GuiMenuAction::RefreshDaemonStatus));
            }
            if event.id == tray.quit_id {
                return Some(TrayCommand::Quit);
            }
            None
        }

        fn process_tray_events(&self) -> Vec<TrayCommand> {
            let mut commands = Vec::new();
            while let Ok(event) = MenuEvent::receiver().try_recv() {
                if let Some(command) = self.on_tray_menu_event(event) {
                    commands.push(command);
                }
            }
            commands
        }
    }

    impl DesktopShellBridge for WindowBridge {
        fn render_menu(&mut self, view: &GuiMenuView) {
            let title = format!(
                "Retaia Agent | state={:?} daemon={} | [S]tatus [C]prefs [P]lay [A]pause [X]stop [D]startd [E]stopd [R]refresh [W]indow [Q]quit",
                view.run_state,
                daemon_status_label(view.daemon_status.as_ref())
            );
            if let Some(window) = self.window.as_ref() {
                window.set_title(&title);
            }
            if let Some(tray) = self.tray.as_ref() {
                tray.refresh_from_view(view);
            }
        }

        fn open_status_window(&mut self, content: &str) {
            let _ = rfd::MessageDialog::new()
                .set_title("Retaia Agent Status")
                .set_description(content)
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
        }

        fn open_settings_panel(&mut self, content: &str) {
            let _ = rfd::MessageDialog::new()
                .set_title("Retaia Agent Preferences")
                .set_description(content)
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
        }

        fn request_quit(&mut self) {
            self.quit_requested = true;
        }
    }

    struct DesktopRuntimeApp {
        controller: DesktopShellController<NativeDaemonManager>,
        bridge: WindowBridge,
    }

    impl DesktopRuntimeApp {
        fn new(settings: AgentRuntimeConfig) -> Result<Self, String> {
            let session = RuntimeSession::new(retaia_agent::ClientRuntimeTarget::Agent, settings)
                .map_err(|errors| compact_validation_reason(&errors))?;
            Ok(Self {
                controller: DesktopShellController::with_default_user_daemon(
                    session,
                    NativeDaemonManager,
                ),
                bridge: WindowBridge::new(),
            })
        }

        fn handle_gui_action(&mut self, event_loop: &ActiveEventLoop, action: GuiMenuAction) {
            let result = {
                let controller = &mut self.controller;
                let bridge = &mut self.bridge;
                controller.handle_action(action, bridge)
            };
            if let Err(error) = result {
                let _ = rfd::MessageDialog::new()
                    .set_title("Retaia Agent Desktop Error")
                    .set_description(format!("{error}"))
                    .set_buttons(rfd::MessageButtons::Ok)
                    .show();
            }
            if self.bridge.quit_requested {
                event_loop.exit();
            }
        }

        fn handle_tray_command(&mut self, event_loop: &ActiveEventLoop, command: TrayCommand) {
            match command {
                TrayCommand::OpenWindow => self.bridge.show_window(),
                TrayCommand::Gui(action) => self.handle_gui_action(event_loop, action),
                TrayCommand::Quit => event_loop.exit(),
            }
        }
    }

    impl ApplicationHandler for DesktopRuntimeApp {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.bridge.window.is_none() {
                let attributes: WindowAttributes = Window::default_attributes()
                    .with_title("Retaia Agent")
                    .with_inner_size(winit::dpi::LogicalSize::new(900.0_f64, 120.0_f64));
                match event_loop.create_window(attributes) {
                    Ok(window) => {
                        self.bridge.set_window(window);
                    }
                    Err(error) => {
                        eprintln!("unable to create desktop window: {error}");
                        event_loop.exit();
                        return;
                    }
                }
            }

            if let Err(error) = self.bridge.ensure_tray() {
                eprintln!("unable to create tray icon: {error}");
                event_loop.exit();
                return;
            }

            self.controller.render_initial_menu(&mut self.bridge);
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            _window_id: winit::window::WindowId,
            event: WindowEvent,
        ) {
            match event {
                WindowEvent::CloseRequested => {
                    self.bridge.hide_window();
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.state != ElementState::Pressed {
                        return;
                    }
                    let action = match event.physical_key {
                        PhysicalKey::Code(KeyCode::KeyS) => Some(GuiMenuAction::OpenStatusWindow),
                        PhysicalKey::Code(KeyCode::KeyC) => Some(GuiMenuAction::OpenSettings),
                        PhysicalKey::Code(KeyCode::KeyP) => Some(GuiMenuAction::PlayResume),
                        PhysicalKey::Code(KeyCode::KeyA) => Some(GuiMenuAction::Pause),
                        PhysicalKey::Code(KeyCode::KeyX) => Some(GuiMenuAction::Stop),
                        PhysicalKey::Code(KeyCode::KeyD) => Some(GuiMenuAction::StartDaemon),
                        PhysicalKey::Code(KeyCode::KeyE) => Some(GuiMenuAction::StopDaemon),
                        PhysicalKey::Code(KeyCode::KeyR) => {
                            Some(GuiMenuAction::RefreshDaemonStatus)
                        }
                        PhysicalKey::Code(KeyCode::KeyW) => {
                            self.bridge.show_window();
                            None
                        }
                        PhysicalKey::Code(KeyCode::KeyQ) => Some(GuiMenuAction::Quit),
                        _ => None,
                    };

                    if let Some(action) = action {
                        self.handle_gui_action(event_loop, action);
                    }
                }
                _ => {}
            }
        }

        fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
            let commands = self.bridge.process_tray_events();
            for command in commands {
                self.handle_tray_command(event_loop, command);
            }
            if self.bridge.quit_requested {
                event_loop.exit();
            }
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
        let event_loop = EventLoop::new().map_err(|error| error.to_string())?;
        let mut app = DesktopRuntimeApp::new(settings)?;
        println!(
            "Desktop shell active (tray + window). Window close hides to tray. Controls: tray menu or keys [S] status [C] prefs [P] play [A] pause [X] stop [D] start daemon [E] stop daemon [R] refresh daemon [W] show window [Q] quit"
        );
        event_loop
            .run_app(&mut app)
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
