# Notifications (Agent local)

> Cadrage fonctionnel global: `retaia-docs/agent/NOTIFICATIONS-UX.md`

## Principle

- Notifications émises sur événement/transition.
- Aucune répétition en boucle sur polling stable.

## Bridge DDD

- Port application: `NotificationSink`
- Service application: `dispatch_notifications(...)`
- Adapter infra OS: `SystemNotificationSink`
  - utilise `notify-rust` sur OS supportés (`macOS`, `Linux`, `Windows`)
  - comportement strict: succès si dispatch OS réussi, erreur sinon (`OK/NOK`)
- Adapter infra de base: `StdoutNotificationSink`
- Sélecteur runtime: `select_notification_sink(profile)` + `notification_sink_profile_for_target(target)`
  - `AGENT`/`MCP` => profil `HeadlessCli` (`StdoutNotificationSink`)
  - `UI_WEB`/`UI_MOBILE` => profil `DesktopSystem` (`SystemNotificationSink`)
- Adapter GUI: `TauriNotificationSink` (feature `tauri-notifications`)
- Façade runtime: `RuntimeSession::update_snapshot_and_dispatch(...)` pour enchaîner projection + dispatch dans le flux agent.
- Intégration shell desktop (`agent-desktop-shell`): dispatch via `dispatch_notifications(...)` + `select_notification_sink(notification_sink_profile_for_target(UI_WEB))`.
- Regle: la deduplication reste dans le domaine, le bridge ne doit pas reintroduire de logique metier.
