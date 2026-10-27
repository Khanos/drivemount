# Local GLib compatibility patch

Base: crates.io `glib` 0.18.5, retained under its original license. The Tauri 2 Linux GTK3 dependency chain requires the 0.18 API family.

Change: backport the two-line mutable-out-parameter correction in `src/variant_iter.rs` from [gtk-rs-core PR 1343](https://github.com/gtk-rs/gtk-rs-core/pull/1343). `VariantStrIter::impl_get` now initializes a mutable pointer and passes `&mut p` to `g_variant_get_child`, matching its output-parameter contract.

This fixes the implementation described in [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html) without pretending this crate is GLib 0.20 or changing GTK APIs. An optimized regression test exercises the affected iterator methods. Cargo-audit still recognizes the conservative version range, so the single documented advisory exception is coupled to a source-integrity check and the regression test. Other advisories are not suppressed.

Remove the patch and exception once the upstream Tauri/GTK stack supports a fixed GLib API family. Do not modify the vendored crate beyond this documented patch without repeating the dependency review.
