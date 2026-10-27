<p align="center">
  <img src="src-tauri/icons/icon.svg" width="96" alt="DriveMount icon" />
</p>

# DriveMount

**Your Google Drive, right in your Linux files.**

DriveMount is an independent Linux desktop app that helps you connect Google Drive through a graphical setup wizard. It manages [rclone](https://rclone.org/), FUSE3 and systemd user services, so you can browse and edit cloud files in your normal file manager.

**Status: experimental engineering build, version 0.1.1.** The native app, package builds and automated tests have been exercised locally. Real Google authorization followed by a new Google Drive mount, installation on each target distribution, and login/reboot recovery still need acceptance testing. There is no verified binary release yet. See [verification](docs/VERIFICATION.md) before using valuable data.

DriveMount is a connected filesystem, **not a complete offline copy, backup or synchronization client**. Files load from the cloud as needed. Changes made through a read/write mount can change or delete files in Google Drive.

## Features

- Guided system checks and dependency installation after explicit approval.
- Google account authorization in your normal browser, with a personal OAuth client guide.
- A native folder picker with validation for empty, accessible mount locations.
- Read/write or read-only access, cache presets and automatic connection on sign-in.
- A dashboard for connection status, opening Drive, reconnecting and safe disconnects.
- Existing rclone remote/service discovery and consent-based import without overwriting credentials.
- Separate shareable status reports and local troubleshooting events.
- Light, dark and system themes, keyboard navigation and an explicit mock mode for development.

## Requirements

| Distribution family  | Dependency installer   | Connection components |
| -------------------- | ---------------------- | --------------------- |
| Arch and derivatives | pacman through Polkit  | rclone, fuse3         |
| Ubuntu / Debian      | apt-get through Polkit | rclone, fuse3         |
| Fedora               | dnf through Polkit     | rclone, fuse3         |

The app requires a systemd user session, a private `XDG_RUNTIME_DIR`, `/dev/fuse` access, a browser, a file manager and a Polkit authentication agent. It runs as your normal user and never enables user lingering.

Rclone **1.70 or newer** is the compatibility baseline; the local protocol tests used 1.75.2. Installer adapters exist for all three distribution families. Arch-based Linux has been inspected locally; Ubuntu/Debian and Fedora installation acceptance remains outstanding. Unsupported systems are reported without automatic installation.

## Build and run

Install current stable Rust (minimum 1.99), Node.js 22.12+ (Node 24 LTS recommended), npm, Python 3.11+ for packaging checks, and the [Tauri Linux prerequisites](https://v2.tauri.app/start/prerequisites/).

Typical native build libraries:

- **Arch:** `base-devel`, `webkit2gtk-4.1`, `gtk3`, `librsvg`, `pkgconf`.
- **Ubuntu/Debian:** `build-essential`, `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `librsvg2-dev`, `libssl-dev`.
- **Fedora:** the corresponding WebKitGTK 4.1, GTK3, librsvg2 and C development packages.

```sh
git clone https://github.com/Khanos/drivemount.git
cd drivemount
npm ci
npm run desktop
```

Startup inspection is read-only. Installing components, authorizing an account, importing a connection and changing mounts require explicit actions in the app.

For a browser-only preview that cannot modify the host:

```sh
npm run dev:mock
# Open http://127.0.0.1:1420
```

The preview displays a mock-mode banner. Fixture scenarios include `?scenario=existing`, `missing`, `offline`, `oauth-failure`, `mount-failure` and `pending`. Ordinary `npm run dev` does not silently substitute a mock backend.

## Connect your Google account

No Google credentials ship with the project, and the app does not fall back to rclone's shared Google OAuth client.

The default setup guides you through creating a Google Cloud project, enabling the Drive API, configuring consent and creating a **Desktop app** OAuth client. Enter the client ID and client secret locally in DriveMount, then authorize access in your browser. Do not post credentials in issues or commit the downloaded client JSON.

A distributor can provide an application-owned public Desktop registration. Google verification and consent requirements must be completed before distributing that client broadly. A distributed Desktop client parameter is not a private server secret. See [OAuth setup and release requirements](docs/OAUTH.md).

Tokens remain under rclone's management. The app requests full Drive scope; choosing a read-only mount restricts that mount's operations, rather than narrowing an existing OAuth grant.

## Build installation packages

```sh
npm run package:deb
npm run package:rpm

# Generate a normalized source archive and checksummed Arch PKGBUILD
./packaging/source-archive.sh
cd packaging/arch
makepkg -s
```

Tauri writes packages under `src-tauri/target/release/bundle/`. The generated Arch recipe installs the native executable, launcher, icon and license.

**Build and test packages on their target distribution.** A Debian or RPM file built on Arch does not establish compatibility with another distribution's glibc, WebKit, dependencies or security policy. The current local packages are engineering artifacts, not verified distribution releases.

Source exports use an explicit reviewed file list in `packaging/source-files.json`. Review and add new source files individually. Build public packages in a clean environment with generic paths and identity; see [release safety](docs/RELEASE-SAFETY.md).

`npm run package:appimage` is experimental. AppImage launchers may need an older FUSE runtime even when FUSE3 is installed for rclone; AppImage creation and execution have not been verified.

## Tests

```sh
npm run lint
npm run typecheck
npm test
npm run build
python3 packaging/verify_vendor.py
python3 -m unittest discover -s packaging -p 'test_*.py'
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --locked --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --locked --no-default-features --manifest-path src-tauri/Cargo.toml
cargo test --locked --release --manifest-path src-tauri/Cargo.toml --test glib_regression

# Mock browser acceptance tests; set CHROMIUM_PATH for your Chromium installation
CHROMIUM_PATH=/usr/bin/chromium npm run test:e2e
```

The normal backend suite simulates service and mount operations in temporary fixtures. Some workflow tests still need installed `rclone`, `fusermount3`, `systemctl` and `stat` binaries for path resolution.

Two real-tool fixture checks are explicitly opt-in. They create only a temporary **local** rclone remote and validate a temporary unit without loading it:

```sh
cargo test --locked --no-default-features --manifest-path src-tauri/Cargo.toml \
  --test rclone_protocol -- --ignored
```

The local 0.1.1 verification passed 10 frontend tests, 8 browser scenarios, 29 Rust unit tests, 11 isolated workflows, 2 opt-in tool tests, 2 source-export tests and an optimized GLib regression. These results do not establish real-account OAuth or mount acceptance.

## Security and data ownership

- The frontend calls a restricted typed Rust API; it cannot supply arbitrary executables, privileged commands or package names.
- Credentials travel to a private rclone helper through a validated Unix socket, rather than command arguments or frontend persistence.
- Credential files are created with mode `0600`; private application directories use `0700`.
- Existing external configurations and services are preserved. Taking management actions requires consent.
- Known pending uploads and open files block disconnect. Unavailable write-back statistics require an explicit risk acknowledgement.
- There is no destructive cache-cleaning endpoint, telemetry backend or automatic updater.
- The copy button exports fixed connection-status fields. Local troubleshooting events may still contain private filenames and require manual review before sharing.

Four upstream maintenance warnings remain in the Rust dependency chain. GTK3-compatible GLib includes a minimal upstream backport for RUSTSEC-2024-0429, guarded by source hashes and an optimized regression test. The single documented audit exception is not a blanket advisory suppression. See [security](docs/SECURITY.md) and [release safety](docs/RELEASE-SAFETY.md).

A cache limit such as `10G` is a cleanup target, not a hard disk cap. Cached reads are not a complete offline copy, and writes can remain pending before upload. Close editing applications and wait for uploads before disconnecting; another application can begin a write after a pre-stop check.

## Troubleshooting and current limits

- **Wayland protocol error on startup:** the local graphics environment required this process-level WebKit workaround:

  ```sh
  WEBKIT_DISABLE_DMABUF_RENDERER=1 npm run desktop
  # For an installed build:
  WEBKIT_DISABLE_DMABUF_RENDERER=1 drivemount
  ```

  The app does not apply this setting automatically. Wayland graphics compatibility remains a release gate.

- **No systemd session or FUSE access:** use a supported desktop session and ensure `/dev/fuse` is accessible. A missing or unsafe runtime directory fails closed.
- **Expired authorization:** renew an owned account in the app; externally managed accounts remain with their original tool.
- **Folder conflict:** choose an empty folder. DriveMount will not hide populated local directories.
- **Network unavailable:** a running rclone mount retries operations; a failed managed service is configured to restart.
- **Unknown upload state:** wait or review the explicit acknowledgement before stopping. Known pending writes cannot be overridden.
- Encrypted rclone configurations have no GUI password-unlock flow. Complex external service wrappers, drop-ins, separate config files and custom systemd search paths may require the original tool.
- Account email is not reliably obtained, so the app shows the remote identity.

`drivemount --inspect` is available to developers. Its output includes local paths and remote names; review it before sharing.

## Project structure

```text
src/                  React wizard, dashboard, preferences and typed native API
src-tauri/src/        Rust commands, rclone, systemd, validation and platform adapters
src-tauri/vendor/     Documented GLib compatibility backport and original licenses
src-tauri/tests/      Isolated workflows and opt-in native protocol checks
tests/                Playwright mock acceptance scenarios
packaging/            Source exporter, Arch template, clean-builder helper and checks
docs/                 Architecture, OAuth, security and verification details
```

Configuration and cache paths follow XDG conventions. Each managed connection has its own registry identity, configuration, cache and user service. The application requires a trusted `XDG_RUNTIME_DIR`; there is no temporary-directory fallback. See [architecture](docs/ARCHITECTURE.md).

## Uninstall safely

Removing the application package does not remove Google files, credentials, mount directories, caches or user services. A configured rclone service can continue after the GUI is uninstalled.

Before retiring a connection, close files, wait for uploads, disconnect it and disable automatic connection. Keep its cache until all edits have reached Google. Revoke the OAuth grant through your Google account if desired. Configuration removal is deliberately separate from package uninstall.

## License and independence

[MIT](LICENSE). Vendored dependencies retain their original notices and licenses.

DriveMount is an independent project and is not affiliated with, endorsed by, or an official product of Google. Google Drive is a trademark of Google LLC.
