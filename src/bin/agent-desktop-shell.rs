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
    use winit::application::ApplicationHandler;
    use winit::event::{ElementState, WindowEvent};
    use winit::event_loop::{ActiveEventLoop, EventLoop};
    use winit::keyboard::{KeyCode, PhysicalKey};
    use winit::window::{Window, WindowAttributes};

    #[derive(Debug, Parser)]
    #[command(
        name = "agent-desktop-shell",
        about = "Retaia desktop shell (feature-gated minimal GUI)"
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

    struct WindowBridge {
        window: Option<Window>,
        quit_requested: bool,
    }

    impl WindowBridge {
        fn new() -> Self {
            Self {
                window: None,
                quit_requested: false,
            }
        }

        fn set_window(&mut self, window: Window) {
            self.window = Some(window);
        }
    }

    impl DesktopShellBridge for WindowBridge {
        fn render_menu(&mut self, view: &GuiMenuView) {
            let title = format!(
                "Retaia Agent Desktop | state={:?} | play={} pause={} stop={} | [S]tatus [C]onfig [P]lay [A]pause [X]stop [D]startd [E]stopd [R]refresh [Q]quit",
                view.run_state, view.can_play_resume, view.can_pause, view.can_stop
            );
            if let Some(window) = self.window.as_ref() {
                window.set_title(&title);
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
                .set_title("Retaia Agent Settings")
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
    }

    impl ApplicationHandler for DesktopRuntimeApp {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.bridge.window.is_some() {
                return;
            }

            let attributes: WindowAttributes = Window::default_attributes()
                .with_title("Retaia Agent Desktop")
                .with_inner_size(winit::dpi::LogicalSize::new(880.0_f64, 120.0_f64));
            match event_loop.create_window(attributes) {
                Ok(window) => {
                    self.bridge.set_window(window);
                    self.controller.render_initial_menu(&mut self.bridge);
                }
                Err(error) => {
                    eprintln!("unable to create desktop window: {error}");
                    event_loop.exit();
                }
            }
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            _window_id: winit::window::WindowId,
            event: WindowEvent,
        ) {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
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
                        PhysicalKey::Code(KeyCode::KeyQ) => Some(GuiMenuAction::Quit),
                        _ => None,
                    };

                    if let Some(action) = action {
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
                }
                _ => {}
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

    fn run() -> Result<(), String> {
        let cli = Cli::parse();
        let settings = load_settings(cli.config)?;
        let event_loop = EventLoop::new().map_err(|error| error.to_string())?;
        let mut app = DesktopRuntimeApp::new(settings)?;
        println!(
            "Desktop shell active: use keys [S] status [C] settings [P] play [A] pause [X] stop [D] start daemon [E] stop daemon [R] refresh daemon [Q] quit"
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
