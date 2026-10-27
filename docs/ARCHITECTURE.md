# Architecture and invariants

The React layer describes product states, collects preferences, and renders native command outcomes. Zustand persists only appearance, selected connection, folder choice, and non-secret preferences. Temporary OAuth credentials remain component state until the native request is accepted; they are then cleared.

The native command layer owns a shared `Engine`. Mutations take a process-local mutex and an advisory filesystem lock. The engine owns registry transactions, setup orchestration, status interpretation, cache guards, and recovery. `Runner` abstracts bounded subprocess execution; `Host` abstracts network availability and mount snapshots for workflow tests. Unit generation, remote-name validation, OS parsing, mountinfo decoding, protocol answers, package argument construction, and redaction remain independently testable.

System integration uses rclone's supported config protocol and API; systemd is the sole owner of persistent mount processes. The app does not spawn an independent mount for a service it already manages. Each new connection gets a UUID, unit, cache, and configuration. Imported remotes can remain in their existing configuration while a new service uses a distinct cache, or an explicitly adopted existing service can remain externally owned.

Startup inspection reads `/etc/os-release`, binary versions, FUSE access, rclone's reported configuration path, remote names/types, systemd state, `/proc/self/mountinfo`, running service process arguments, and recognizable service definitions. Shell wrappers and unresolved unit semantics fail closed. Inspection never performs automatic migration or account authorization.

Setup validates settings, confirms the folder is safe, verifies the remote, saves recovery metadata, creates directories, generates an owned unit, reloads systemd, configures startup, starts the service, checks the actual mount source and target, verifies the remote again, and probes filesystem metadata. Success requires those checks rather than a zero subprocess exit code alone.

Dashboard health checks distinguish filesystem mount presence, user service activation, general Google reachability, authorization failures, and pending writes. Visible dashboards poll at a moderate 30-second cadence and support manual refresh. Network reachability is a best-effort probe; captive portals, proxies, DNS failures, and enterprise filtering can need additional diagnosis. A running mount can remain present while offline.

App-owned units use `Type=notify`, explicit config/cache paths, `RuntimeDirectory`, `UMask=0077`, and restart-on-failure with a 15-second delay. RuntimeDirectory preservation avoids stopping one mount removing another mount's socket directory. Startup is attached to the user's `default.target`; user lingering is never enabled.

The app's executable, metadata, desktop entry, icons, and UI are packaged with Tauri's Linux bundler. Runtime connection tools remain distribution packages. Build artifacts from a newer host must be rebuilt against the target distribution's ABI before public release.
