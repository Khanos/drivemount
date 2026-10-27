import contextlib
import io
import json
from pathlib import Path
import shutil
import tarfile
import tempfile
import unittest
from source_archive import ROOT, create

class ExportTests(unittest.TestCase):
    def fixture(self, path):
        entries = json.loads((ROOT / 'packaging/source-files.json').read_text())
        for entry in entries:
            target = path / entry
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / entry, target)
        return entries

    def test_allowlist_metadata_checksum_and_reproducibility(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / 'project'
            self.fixture(root)
            (root / '.npmrc').write_text('//registry.example/:_authToken=SYNTHETIC_TEST_ONLY')
            (root / '.tools').mkdir()
            (root / '.tools/personal.json').write_text('{"person":"private-test"}')
            with contextlib.redirect_stdout(io.StringIO()):
                first = create(root, Path(temporary) / 'a')
                second = create(root, Path(temporary) / 'b')
            self.assertEqual(first.read_bytes(), second.read_bytes())
            with tarfile.open(first) as archive:
                for member in archive:
                    self.assertEqual((member.uid, member.gid, member.uname, member.gname), (0, 0, '', ''))
                    self.assertEqual(member.mtime, 0)
                    self.assertNotIn('.npmrc', member.name)
                    self.assertNotIn('.tools', member.name)
            recipe = (first.parent / 'PKGBUILD').read_text()
            import hashlib
            self.assertIn(hashlib.sha256(first.read_bytes()).hexdigest(), recipe)
            self.assertNotIn('SKIP', recipe)

    def test_symlink_refused(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary) / 'project'
            self.fixture(root)
            (root / 'LICENSE').unlink()
            (root / 'LICENSE').symlink_to('/etc/hostname')
            with self.assertRaises(ValueError), contextlib.redirect_stdout(io.StringIO()):
                create(root, Path(temporary) / 'out')

if __name__ == '__main__':
    unittest.main()
