use retaia_agent::{AgentRunState, MenuAction, base_menu_actions, menu_visibility};

#[test]
fn tdd_menu_toggle_visibility_respects_running_and_paused_state() {
    let running = menu_visibility(AgentRunState::Running);
    assert!(!running.show_play_resume);
    assert!(running.show_pause);

    let paused = menu_visibility(AgentRunState::Paused);
    assert!(paused.show_play_resume);
    assert!(!paused.show_pause);
}

#[test]
fn tdd_base_menu_actions_are_stable() {
    let actions = base_menu_actions();
    assert_eq!(
        actions,
        vec![
            MenuAction::OpenStatusWindow,
            MenuAction::OpenSettings,
            MenuAction::Stop,
            MenuAction::Quit,
        ]
    );
}
