# Verification record and release boundaries

## Current 0.1.1 engineering build

The reviewed source and clean-built 0.1.1 artifacts passed 10 frontend tests, 8 browser scenarios, 29 Rust unit tests, 11 isolated workflows, 2 opt-in real-tool fixtures, 2 source-export regressions and the optimized GLib regression. The source archive reproduced byte for byte before the public README update. Secret scans and personal-identifier checks of the source, package payloads/metadata and executable strings found no leaks. npm audit found no known vulnerabilities; RustSec returned four visible upstream maintenance warnings with the documented verified GLib backport exception.

The 0.1.1 Arch package records generic `/build/arch` paths, a generic release-builder identity and the isolated builder package inventory. The Debian/RPM packages were constructed on that same clean Arch builder and still require target-distribution builds and installation acceptance. No binary release is advertised as verified.

Native startup in the local Wayland graphics environment failed with a GDK protocol error until `WEBKIT_DISABLE_DMABUF_RENDERER=1` was set for the test process. With that workaround, a visible running window was observed. The existing Drive configuration and service remained unchanged and active. Real Google authorization, a new Google Drive mount, write-back, reconnect, login and reboot acceptance remain outstanding.

The checks below describe the earlier 0.1.0 engineering build and are retained as historical evidence.

## Historical 0.1.0 verification

Local work: 2026-10-26, Arch-based Omarchy, x86_64. The build environment uses an isolated current stable Rust toolchain under `/tmp`; no system Rust installation or existing Drive reconfiguration was performed.

## Automated evidence

Final automated totals: **9 frontend unit/component tests, 8 browser scenarios, 26 Rust unit tests, 11 isolated workflow tests, and 2 explicitly enabled real-tool integration tests passed.** Type checking, ESLint, rustfmt, Clippy with warnings denied, and the production build passed.

- TypeScript type checking, ESLint including React hook rules, Vite production build.
- Frontend unit/component tests: navigation, dependency-consent gating, existing-installation consent, resumed drafts, preference persistence, credential validation, and cache preset mapping.
- Playwright browser scenarios using the explicit mock backend: full setup/dashboard flow, external connection import, missing dependencies, OAuth failure, failed mount, offline startup, pending uploads, and light/dark/minimum-width layout.
- Rust unit tests: OS families, version/remote parsing, argument restrictions, path safety, configuration permissions/atomic persistence/locking, config protocol states, unit quoting/ownership, existing-service parsing, status parsing, cache uncertainty, and redaction.
- Isolated native workflow tests with simulated services/mounts: setup, restore after process restart, failed startup rollback, auth failure, unknown upload acknowledgement, disconnect/reconnect, offline state, competing operations, external-unit preservation, populated/competing folders, interrupted setup, and replaced-unit refusal.
- Two opt-in real-tool checks: rclone config/create/config/update/config/listremotes over a private Unix socket with a temporary local remote, and systemd-analyze validation of a temporary generated unit containing spaces, Unicode, `%`, and `$`. No real service was loaded and no Google account was authorized.
- Native CLI inspection correctly detected the development host’s existing Google Drive remote, writable mount, FUSE access, systemd user manager, and external service, with no adoption performed.

The normal backend suite leaves the two real-tool tests ignored; they were additionally run explicitly in the isolated temporary environment. Package artifact and launch checks are recorded below as they are completed.

## What these tests do not prove

The browser acceptance suite validates UI behavior against fixtures. It does not prove real Polkit prompts, browser OAuth consent, Google token issuance, a new Google Drive mount, file-manager writes, automatic login mounting, or successful reboot recovery.

The real-tool configuration test uses the local backend. The Google-specific non-interactive state handling is based on current rclone documentation and source, and needs an end-to-end disposable Google account test.

Building Debian and RPM files on Arch does not establish compatibility with the target distribution's glibc, WebKit, GTK, package dependencies, or security policy. Build and install natively on supported Ubuntu/Debian and Fedora versions before public release. The provided CI definition has not run remotely.

No host package installation, OAuth credential rotation, existing-service adoption, mount stop/restart, or cloud file modification was used as an automated acceptance test.

## Remaining product limits

- Public application-owned OAuth registration and Google verification are external release prerequisites; the personal-client guide works without a shipped publisher registration.
- Rclone older than 1.70 is flagged rather than automatically replaced from a third-party repository.
- Complex manually configured units (shell wrappers, unresolved environment expansion, detected drop-ins, unusual search paths) may require management in their original tool.
- Import currently selects remotes from the default configuration reported by rclone. Separately configured services can be detected but may need their original management tool.
- Existing encrypted rclone configurations have no GUI password-unlock flow.
- Imported external services have lifecycle/startup controls, but their configuration and account renewal remain with their original tool.
- Unknown write-back state requires explicit acknowledgement; known pending uploads block disconnect. Another application can start an operation after the pre-stop check; the app cannot freeze other applications’ I/O.
- The app never clears caches or deletes credentials. An in-app configuration-removal workflow is intentionally absent until recovery and pending-write guarantees can be demonstrated.
- AppImage packaging/runtime behavior, non-systemd desktops, and arbitrary custom XDG runtime locations are not verified.
- No reliable account email is currently obtained, so the UI displays the remote identity rather than inventing account details.

## Manual acceptance before public release

Use a disposable account and test files. Exercise personal OAuth and a verified publisher client, denial/cancellation/timeout, reauthorization after token revocation, an empty custom folder with spaces/Unicode, read-only enforcement, safe disposable edits, network loss/recovery, pending-write behavior, file-manager access, GUI close/reopen, multiple mounts, login, and reboot. Exercise package installation/removal in clean target-distribution VMs. Keep existing credentials and cache snapshots until all edits are confirmed uploaded.

## Package and native-window checks

The final Debian and RPM packages were built with Tauri. Both archives were readable. The Debian payload was extracted into a temporary directory, its desktop entry passed validation, three PNG icons were present, its executable returned `DriveMount 0.1.0`, and `ldd` reported no missing libraries on the build host. The RPM archive was also readable; target-Fedora installation remains untested.

The real release executable was briefly launched with temporary XDG configuration, data, and cache directories. Hyprland reported a visible `drivemount` window titled `DriveMount`; the process remained running, and startup logs contained no fatal errors. The test closed only that exact application process. Hashes of the existing rclone configuration and external service file were unchanged, and the existing Drive service remained active afterward.

These are native launch and package-content checks, not installation acceptance on Ubuntu or Fedora, and not Google OAuth/mount acceptance.

The Arch source recipe disables makepkg C/C++ LTO for the bundled `ring` crate, following the [Arch Rust package guidance](https://wiki.archlinux.org/title/Rust_package_guidelines#Build). Cargo’s Rust release optimization remains enabled.

The Arch source package also built successfully with `makepkg` after the LTO adjustment. Its package check phase passed all 9 frontend tests and all 37 default backend tests. The resulting `drivemount-0.1.0-1-x86_64.pkg.tar.zst` was recognized by pacman, extracted successfully, returned `DriveMount 0.1.0`, and contained a valid desktop entry, SVG icon, and license. Dependency checks were bypassed for this local build because Rust/Cargo were provided by the isolated toolchain; the normal recipe declares its build and runtime dependencies. No package was installed on the host.
