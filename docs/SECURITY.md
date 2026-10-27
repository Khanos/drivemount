# Security and data ownership

## Boundaries

The frontend can call only typed, registered Tauri commands. It has no shell or filesystem plugin permission and cannot supply executable paths, arbitrary systemd verbs, package names, URLs, or RC endpoints. Help links are a fixed backend allowlist. The production CSP disallows frames, remote scripts, and remote frontend networking. A single-instance plugin prevents accidental duplicate desktop processes; backend mutexes and a private file lock prevent competing mutations across operations/processes.

Native execution uses argument arrays with no `sh -c`. All child operations have timeouts. OAuth is handled by a dedicated rclone helper over a private Unix socket. Rclone RC error responses can echo input parameters, so their raw bodies are discarded rather than forwarded to the UI.

The only elevated operation is `/usr/bin/pkexec` running the verified root-owned system package manager with fixed flags and allowlisted missing packages (`rclone`, `fuse3`). User-controlled PATH entries cannot select the elevated package-manager binary. The app does not receive the administrator password and does not run as root.

## Files and services

New credentials are kept in separate owned configurations. Existing remotes are reused after consent, never rewritten during import. Registry updates are atomic and private, lock files use no-follow semantics, and owned directories/files reject inappropriate symlinks. Mount paths reject parent traversal, control characters, symlink traversal, populated directories, other mounts, and application-data overlap.

Generated service arguments quote systemd syntax, double literal percent/dollar signs, and reject control characters. Services carry a connection ownership marker. An unowned unit is never replaced; controls on app-owned connections recheck the marker. Existing external services remain external and cannot be rewritten through Settings.

A partial setup preserves its record. Failure before mounting attempts to undo its owned unit/startup changes. A mounted connection whose later verification fails is left intact to avoid discarding cached writes. Recognizable partial units are exposed for management on reopen. Existing external configuration and cloud data are not rollback targets.

## Cache and stopping

Every new service has its own cache identity and private RC socket. Before a disruptive stop, the backend reads `vfs/stats`: queued/in-progress/errored uploads and in-use files block the stop. Missing statistics require explicit acknowledgement, including externally managed mounts. A known nonzero pending count cannot be overridden through the acknowledgement flag.

There is no destructive cache-cleaning endpoint. No cloud files, mount folders, or credentials are deleted by package uninstall. A pre-stop check cannot prevent a different application from starting a new write immediately afterward; users must close editing applications before stopping. Statistics loss is treated as uncertainty rather than proof of safety.

## Diagnostics and scope

The application never asks rclone to dump its configuration. It lists remote names/types, checks configuration paths, and reads selected service properties. The shareable report is constructed only from a fixed service-state vocabulary, app version, and booleans. It excludes journal messages, service/remote/account identities, paths, and computer names. Separate collapsed local events use `journalctl --output=cat`, omit credential/opaque lines, strip control characters, and filter home paths, hostnames and email addresses. Arbitrary messages can still contain private filenames; these local events carry a review warning and are never copied by the shareable report button.

Read-only host inspection is not token validation: it intentionally avoids cloud-account operations that could refresh an existing token. Explicit connection health checks perform bounded read-only Google quota requests, and rclone may naturally refresh its stored token while doing so.

No telemetry, hosted backend, server-side OAuth proxy, or automatic updater is included.

The app requires a current-user-owned mode-0700 XDG runtime directory, rejects symbolic links and replaceable ancestor directories, and verifies RC socket type, permissions, owner, and peer UID before use. The OAuth helper also checks its child PID. Missing or unsafe runtime directories fail closed; there is no predictable /tmp fallback.

The GTK3-compatible GLib dependency has a documented source backport for RUSTSEC-2024-0429. Run `python3 packaging/verify_vendor.py` and the optimized `glib_regression` test before accepting the single cargo-audit exception. Four transitive maintenance warnings remain visible; see release safety.
