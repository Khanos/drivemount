#!/usr/bin/env python3
"""Export only reviewed, exact manifest entries with normalized archive metadata."""
import gzip
import hashlib
import io
import json
import os
from pathlib import Path, PurePosixPath
import re
import tarfile
import sys

ROOT = Path(__file__).resolve().parents[1]

def create(root: Path, output: Path) -> Path:
    version = json.loads((root / 'package.json').read_text())['version']
    if not re.fullmatch(r'[0-9]+\.[0-9]+\.[0-9]+', version):
        raise ValueError('Expected a release version')
    entries = json.loads((root / 'packaging/source-files.json').read_text())
    if len(entries) != len(set(entries)):
        raise ValueError('Duplicate manifest entries')
    epoch = int(os.environ.get('SOURCE_DATE_EPOCH', '0'))
    output.mkdir(parents=True, exist_ok=True)
    archive = output / f'drivemount-{version}.tar.gz'
    # Read and validate every entry before writing an export.
    files = []
    for entry in sorted(entries):
        name = PurePosixPath(entry)
        if name.is_absolute() or '..' in name.parts or str(name) != entry:
            raise ValueError('Unsafe manifest path')
        if any(part.startswith('.') and part not in ('.github', '.cargo', '.gitignore', '.prettierignore') for part in name.parts):
            raise ValueError('Unapproved hidden file in manifest')
        if any(part in ('node_modules', 'target', 'release', 'rclone.conf') for part in name.parts):
            raise ValueError('Private or generated path in manifest')
        path = root
        for part in name.parts:
            path = path / part
            if path.is_symlink():
                raise ValueError('Source export refuses symbolic links')
        if not path.is_file():
            raise ValueError(f'Missing source file: {entry}')
        files.append((entry, path.read_bytes()))
    buffer = io.BytesIO()
    with gzip.GzipFile(fileobj=buffer, mode='wb', filename='', mtime=epoch) as compressed:
        with tarfile.open(fileobj=compressed, mode='w', format=tarfile.PAX_FORMAT) as tar:
            for entry, data in files:
                info = tarfile.TarInfo(f'drivemount-{version}/{entry}')
                info.uid = info.gid = 0
                info.uname = info.gname = ''
                info.mtime = epoch
                info.mode = 0o755 if entry.endswith('.sh') else 0o644
                info.size = len(data)
                tar.addfile(info, io.BytesIO(data))
    archive.write_bytes(buffer.getvalue())
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    template = (root / 'packaging/arch/PKGBUILD.in').read_text()
    recipe = template.replace('@VERSION@', version).replace('@SOURCE_SHA256@', digest)
    (output / 'PKGBUILD').write_text(recipe)
    print(f'{digest}  {archive.name}')
    return archive

if __name__ == '__main__':
    create(ROOT, Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / 'packaging/arch')
