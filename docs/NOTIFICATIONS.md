# Notifications

## Principle

- Notifications émises sur événement/transition.
- Aucune répétition en boucle sur polling stable.

## Required Notifications

- `New job received`
  - émission à l'arrivée d'un nouveau job (nouveau `job_id` ou file vide -> non vide)
  - pas de répétition pour le même job
- `All jobs done`
  - émission unique sur transition `has_running_jobs=true -> false`
  - pas de répétition tant que l'état reste sans job actif
- `Job failed`
  - émission à l'échec d'un job
  - inclure un code court + action suggérée si disponible
- `Agent disconnected / reconnecting`
  - émission sur perte de connexion backend et démarrage de reconnexion
- `Auth expired / re-auth required`
  - émission quand l'auth runtime ne permet plus les appels
- `Settings saved`
  - émission après sauvegarde valide
- `Settings invalid`
  - émission sur erreur de validation config (ex: endpoint injoignable)

## Optional Notification

- `Updates available`
  - émission lors de la détection d'une nouvelle version agent

## Bridge DDD

- Port application: `NotificationSink`
- Service application: `dispatch_notifications(...)`
- Adapter infra OS: `SystemNotificationSink`
  - utilise `notify-rust` sur OS supportés (`macOS`, `Linux`, `Windows`)
  - comportement strict: succès si dispatch OS réussi, erreur sinon (`OK/NOK`)
- Adapter infra de base: `StdoutNotificationSink`
- Adapter GUI: `TauriNotificationSink` (feature `tauri-notifications`)
- Façade runtime: `RuntimeSession::update_snapshot_and_dispatch(...)` pour enchaîner projection + dispatch dans le flux agent.
- Règle: la déduplication reste dans le domaine (`AgentUiRuntime`), le bridge ne doit pas réintroduire de logique métier.
