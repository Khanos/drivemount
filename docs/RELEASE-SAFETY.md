# Public artifact safety

Version 0.1.1 fixes the privacy findings in the initial local engineering packages. Do not distribute older packages or source archives. Keep any old build copies private.

## Source exports

`packaging/source-archive.sh [output-directory]` exports only exact entries reviewed in `packaging/source-files.json`. Unknown files, local tool state, npm credentials, build outputs and user configurations are excluded. Symlinks in selected paths are refused. Archive UID/GID, owner names and timestamps are normalized. The generated PKGBUILD pins the actual archive SHA-256; the source contains a template to avoid a circular checksum.

Run `python3 -m unittest discover -s packaging -p 'test_*.py'`. Fixtures prove that future hidden credentials are excluded, selected symlinks are rejected, exports reproduce byte-for-byte, and headers/checksums are normalized. This is a packaging boundary, not a replacement for reviewing deliberately selected source files for secrets.

## Clean builders

Build public packages in a disposable target-distribution environment with a generic builder identity and paths. Never use a personal host package database as the source of Arch `.BUILDINFO`. Preserve clean-builder metadata rather than deleting it. CI actions are pinned to immutable commits.

The local verification builder uses rootless Bubblewrap namespaces and an official Arch base-devel OCI root filesystem. It mounts neither the user's home nor the host package database. Only the reviewed source archive, isolated toolchain/registry cache and output directory are supplied. Its hostname is `buildhost`, HOME is `/home/builder`, build directory is `/build`, and packager is `DriveMount Release Builder`. Rust source paths are remapped to generic `/usr/src` locations. The final package must be inspected after building; the Debian and RPM formats still require rebuilding and install testing on their target distributions.

## Dependency exception

GLib 0.18.5 is required by the current Tauri GTK3 stack. The known iterator unsoundness RUSTSEC-2024-0429 is fixed by the minimal upstream mutable-output-pointer backport retained in `src-tauri/vendor/glib`. Original licenses are included. The crate version remains honest, so cargo-audit's conservative version check requires one documented exception in `.cargo/audit.toml`. Source hashes, lock resolution and an optimized iterator regression test gate that exception:

```sh
python3 packaging/verify_vendor.py
cargo test --locked --release --manifest-path src-tauri/Cargo.toml --test glib_regression
cargo audit --file src-tauri/Cargo.lock
```

Four upstream maintenance warnings remain: proc-macro-error, zvariant, zbus_names and zvariant_derive. They are not hidden or evidence of a credential leak. Monitor the Tauri/GTK and zbus dependency chains for supported replacements. Remove the local GLib patch/exception when the supported dependency stack can use a fixed upstream GLib release.

## Final release gate

Scan the exact source export and extracted final packages locally for secrets and personal data, inspect tar ownership, Arch build paths/package inventory, package metadata and binary strings, and validate all published checksums. Do not upload credentials or source to a scanning service. A clean scan cannot prove the absence of every possible secret; preserve target-distribution, real OAuth/mount and reboot acceptance as separate release gates.
