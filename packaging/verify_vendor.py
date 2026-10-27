#!/usr/bin/env python3
"""Fail closed if the reviewed GLib advisory backport changes or disappears."""
import hashlib
import json
from pathlib import Path
import tomllib
root = Path(__file__).resolve().parents[1]
vendor = root / 'src-tauri/vendor/glib'
expected = json.loads((root / 'packaging/glib-sha256.json').read_text())
actual = {p.relative_to(vendor).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
          for p in vendor.rglob('*') if p.is_file()}
assert actual == expected, 'Vendored GLib differs from the reviewed backport'
manifest = tomllib.loads((root / 'src-tauri/Cargo.toml').read_text())
assert manifest['patch']['crates-io']['glib']['path'] == 'vendor/glib'
lock = tomllib.loads((root / 'src-tauri/Cargo.lock').read_text())
glib = [p for p in lock['package'] if p['name'] == 'glib']
assert len(glib) == 1 and 'source' not in glib[0], 'GLib must resolve to the patched local crate'
print('Reviewed GLib backport and lock resolution verified')
