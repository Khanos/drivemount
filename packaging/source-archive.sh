#!/usr/bin/env bash
set -euo pipefail
project_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
exec python3 "$project_dir/packaging/source_archive.py" "${1:-$project_dir/packaging/arch}"
