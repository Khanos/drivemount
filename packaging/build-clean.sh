#!/usr/bin/env bash
# Developer helper: requires a disposable Arch rootfs with build dependencies installed.
set -euo pipefail
if (( $# != 4 )); then
  echo 'Usage: build-clean.sh ROOTFS ISOLATED_RUST_TOOLCHAIN ISOLATED_CARGO_CACHE OUTPUT_DIR' >&2
  exit 2
fi
project_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
rootfs=$(realpath -- "$1")
toolchain=$(realpath -- "$2")
cargo_cache=$(realpath -- "$3")
mkdir -p -- "$4"
output=$(realpath -- "$4")
[[ "$rootfs" != / && -x "$rootfs/usr/bin/makepkg" && -x "$toolchain/cargo/bin/rustup" ]]
chmod u+w -- "$rootfs"
mkdir -p -- "$rootfs/build/arch" "$rootfs/home/builder" "$rootfs/run/user/1000" "$rootfs/cache/cargo" "$rootfs/opt/toolchain" "$rootfs/out"
chmod 700 -- "$rootfs/home/builder" "$rootfs/run/user/1000"
if ! rg -q '^builder:' "$rootfs/etc/passwd"; then
  printf 'builder:x:1000:1000:Release Builder:/home/builder:/bin/bash\n' >> "$rootfs/etc/passwd"
  printf 'builder:x:1000:\n' >> "$rootfs/etc/group"
fi
"$project_dir/packaging/source-archive.sh" "$rootfs/build/arch"
# The clean rootfs package database is retained for reproducible .BUILDINFO.
exec bwrap --unshare-user --uid 1000 --gid 1000 --unshare-pid --unshare-uts --hostname buildhost \
  --bind "$rootfs" / --dev /dev --proc /proc --tmpfs /tmp \
  --ro-bind /etc/resolv.conf /etc/resolv.conf \
  --ro-bind "$toolchain" /opt/toolchain --bind "$cargo_cache" /cache/cargo --bind "$output" /out \
  --clearenv --setenv HOME /home/builder --setenv USER builder --setenv LOGNAME builder \
  --setenv PATH /opt/toolchain/cargo/bin:/usr/bin --setenv RUSTUP_HOME /opt/toolchain/rustup \
  --setenv CARGO_HOME /cache/cargo --setenv XDG_RUNTIME_DIR /run/user/1000 \
  --setenv LANG C.UTF-8 --setenv LC_ALL C.UTF-8 --setenv TZ UTC \
  --setenv PACKAGER 'DriveMount Release Builder' --setenv MAKEFLAGS '-j4' \
  --setenv CARGO_BUILD_JOBS 4 \
  --setenv RUSTFLAGS '--remap-path-prefix=/build=/usr/src/drivemount --remap-path-prefix=/cache/cargo=/usr/src/cargo --remap-path-prefix=/opt/toolchain=/usr/src/rust' \
  --chdir /build/arch /usr/bin/bash -euc '
    makepkg --nodeps --force
    source ./PKGBUILD
    cd "src/$pkgname-$pkgver"
    python3 packaging/verify_vendor.py
    cargo test --locked --release --test glib_regression --manifest-path src-tauri/Cargo.toml
    npm run tauri -- build --bundles deb,rpm
    cp src-tauri/target/release/bundle/deb/*.deb /out/
    cp src-tauri/target/release/bundle/rpm/*.rpm /out/
    cd /build/arch
    cp drivemount-*.pkg.tar.zst drivemount-*.tar.gz PKGBUILD /out/
    { printf "Clean builder: Arch Linux base-devel; generic identity and paths\n"; rustc --version; cargo --version; node --version; } > /out/BUILD-PROVENANCE.txt
    cd /out
    sha256sum -- *.deb *.rpm *.pkg.tar.zst *.tar.gz PKGBUILD BUILD-PROVENANCE.txt > SHA256SUMS
  '
