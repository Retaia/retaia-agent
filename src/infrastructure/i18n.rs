use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    En,
    Fr,
}

pub fn detect_language() -> Language {
    if let Ok(explicit) = env::var("RETAIA_AGENT_LANG") {
        return parse_language(&explicit).unwrap_or(Language::En);
    }
    if let Ok(lang) = env::var("LANG") {
        return parse_language(&lang).unwrap_or(Language::En);
    }
    Language::En
}

pub fn parse_language(raw: &str) -> Option<Language> {
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized.starts_with("fr") {
        return Some(Language::Fr);
    }
    if normalized.starts_with("en") {
        return Some(Language::En);
    }
    None
}

pub fn t(lang: Language, key: &str) -> &'static str {
    match lang {
        Language::En => en(key),
        Language::Fr => fr(key),
    }
}

fn en(key: &str) -> &'static str {
    match key {
        "desktop.feature_required" => "agent-desktop-shell requires --features desktop-shell",
        "runtime.feature_required" => {
            "agent-runtime requires daemon mode; run `agent-runtime daemon`."
        }
        "runtime.invalid_config" => "invalid runtime config",
        "runtime.interactive_disabled" => {
            "interactive mode is disabled; run `agent-runtime daemon` and control it with `agentctl daemon ...`"
        }
        "runtime.history_store_unavailable" => {
            "history store unavailable; continuing without sqlite history"
        }
        "runtime.daemon_started" => "runtime daemon started",
        "runtime.cycle" => "runtime cycle",
        "runtime.throttled" => "core API throttled; backoff plan applied",
        "runtime.persist_completed_failed" => "unable to persist completed job",
        "runtime.persist_stats_failed" => "unable to persist daemon stats",
        "runtime.persist_cycle_failed" => "unable to persist cycle history",
        "runtime.compact_failed" => "unable to compact cycle history",
        "daemon.installed" => "Daemon installed.",
        "daemon.uninstalled" => "Daemon uninstalled.",
        "daemon.started" => "Daemon started.",
        "daemon.stopped" => "Daemon stopped.",
        "config.valid" => "Config is valid.",
        "config.initialized" => "Config initialized.",
        "config.updated" => "Config updated.",
        "report.copy_block_body_start" => "--- COPY BODY BELOW ---",
        "report.copy_block_body_end" => "--- END BODY ---",
        "report.copy_block_desc_start" => "--- COPY DESCRIPTION BELOW ---",
        "report.copy_block_desc_end" => "--- END DESCRIPTION ---",
        "gui.title" => "Retaia Agent Control Center",
        "gui.run_state" => "Run state",
        "gui.daemon" => "Daemon",
        "gui.ui_uptime" => "UI uptime",
        "gui.controls" => "Daemon Controls",
        "gui.button.refresh_daemon" => "Refresh Daemon Status",
        "gui.button.open_status" => "Open Status",
        "gui.button.open_preferences" => "Open Preferences",
        "gui.button.hide_to_tray" => "Hide to Tray",
        "gui.button.quit" => "Quit",
        "gui.stats" => "Runtime Stats (from daemon)",
        "gui.updated" => "Updated",
        "gui.tick" => "Tick",
        "gui.current_job" => "Current job",
        "gui.asset" => "Asset",
        "gui.progress" => "Progress",
        "gui.stage" => "Stage",
        "gui.status" => "Status",
        "gui.current_job_duration" => "Current job duration",
        "gui.idle" => "idle",
        "gui.last_job" => "Last Job",
        "gui.last_job_id" => "Last job id",
        "gui.duration" => "Duration",
        "gui.completed_at" => "Completed at",
        "gui.no_stats" => "No daemon stats available yet.",
        "gui.start_daemon_hint" => "Start daemon and wait for at least one runtime tick.",
        "gui.shortcuts" => "Shortcuts: S status, C preferences, D daemon toggle, R refresh, Q quit",
        "gui.modal.status" => "Status",
        "gui.modal.preferences" => "Preferences",
        "gui.modal.error" => "Error",
        "gui.button.start_daemon" => "Start Daemon",
        "gui.button.stop_daemon" => "Stop Daemon",
        "gui.tray.open_window" => "Open Window",
        "gui.tray.open_status" => "Open Status",
        "gui.tray.open_preferences" => "Open Preferences",
        "gui.tray.start_stop_daemon" => "Start/Stop Daemon",
        "gui.tray.refresh_daemon" => "Refresh Daemon Status",
        "gui.tray.quit" => "Quit",
        _ => "",
    }
}

fn fr(key: &str) -> &'static str {
    match key {
        "desktop.feature_required" => "agent-desktop-shell nécessite --features desktop-shell",
        "runtime.feature_required" => {
            "agent-runtime nécessite le mode daemon: lance `agent-runtime daemon`."
        }
        "runtime.invalid_config" => "configuration runtime invalide",
        "runtime.interactive_disabled" => {
            "le mode interactif est désactivé; lance `agent-runtime daemon` et pilote avec `agentctl daemon ...`"
        }
        "runtime.history_store_unavailable" => {
            "store d'historique indisponible; poursuite sans historique sqlite"
        }
        "runtime.daemon_started" => "daemon runtime démarré",
        "runtime.cycle" => "cycle runtime",
        "runtime.throttled" => "API Core limitée; backoff appliqué",
        "runtime.persist_completed_failed" => "impossible de persister le job terminé",
        "runtime.persist_stats_failed" => "impossible de persister les stats daemon",
        "runtime.persist_cycle_failed" => "impossible de persister l'historique des cycles",
        "runtime.compact_failed" => "impossible de compacter l'historique des cycles",
        "daemon.installed" => "Daemon installé.",
        "daemon.uninstalled" => "Daemon désinstallé.",
        "daemon.started" => "Daemon démarré.",
        "daemon.stopped" => "Daemon arrêté.",
        "config.valid" => "Configuration valide.",
        "config.initialized" => "Configuration initialisée.",
        "config.updated" => "Configuration mise à jour.",
        "report.copy_block_body_start" => "--- COPIE LE CONTENU CI-DESSOUS ---",
        "report.copy_block_body_end" => "--- FIN DU CONTENU ---",
        "report.copy_block_desc_start" => "--- COPIE LA DESCRIPTION CI-DESSOUS ---",
        "report.copy_block_desc_end" => "--- FIN DE LA DESCRIPTION ---",
        "gui.title" => "Retaia Agent Control Center",
        "gui.run_state" => "État runtime",
        "gui.daemon" => "Daemon",
        "gui.ui_uptime" => "Uptime UI",
        "gui.controls" => "Contrôles daemon",
        "gui.button.refresh_daemon" => "Rafraîchir le statut daemon",
        "gui.button.open_status" => "Ouvrir le statut",
        "gui.button.open_preferences" => "Ouvrir les préférences",
        "gui.button.hide_to_tray" => "Masquer dans le tray",
        "gui.button.quit" => "Quitter",
        "gui.stats" => "Stats runtime (depuis daemon)",
        "gui.updated" => "Mis à jour",
        "gui.tick" => "Tick",
        "gui.current_job" => "Job courant",
        "gui.asset" => "Asset",
        "gui.progress" => "Progression",
        "gui.stage" => "Étape",
        "gui.status" => "Statut",
        "gui.current_job_duration" => "Durée job courant",
        "gui.idle" => "idle",
        "gui.last_job" => "Dernier job",
        "gui.last_job_id" => "ID dernier job",
        "gui.duration" => "Durée",
        "gui.completed_at" => "Terminé à",
        "gui.no_stats" => "Aucune stat daemon disponible pour le moment.",
        "gui.start_daemon_hint" => "Démarre le daemon et attends au moins un tick runtime.",
        "gui.shortcuts" => {
            "Raccourcis: S statut, C préférences, D toggle daemon, R refresh, Q quitter"
        }
        "gui.modal.status" => "Statut",
        "gui.modal.preferences" => "Préférences",
        "gui.modal.error" => "Erreur",
        "gui.button.start_daemon" => "Démarrer daemon",
        "gui.button.stop_daemon" => "Arrêter daemon",
        "gui.tray.open_window" => "Ouvrir la fenêtre",
        "gui.tray.open_status" => "Ouvrir le statut",
        "gui.tray.open_preferences" => "Ouvrir les préférences",
        "gui.tray.start_stop_daemon" => "Démarrer/Arrêter daemon",
        "gui.tray.refresh_daemon" => "Rafraîchir statut daemon",
        "gui.tray.quit" => "Quitter",
        _ => "",
    }
}
