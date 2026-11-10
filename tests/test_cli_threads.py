"""Tests for --search-threads/--download-threads range validation.

An explicit out-of-range value (e.g. 0, -5, 500) must fail fast with a
clean argparse error (exit code 2) instead of being silently ignored
(0 falls through to an interactive prompt that hangs or crashes with
EOFError when stdin is closed; negative values blow up the thread pool).
"""
import importlib.machinery
import importlib.util
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

CLI_PATH = Path(__file__).resolve().parent.parent / "lrc_puller_cli.py"


def _load_cli():
    loader = importlib.machinery.SourceFileLoader("lrc_puller_cli", str(CLI_PATH))
    spec = importlib.util.spec_from_loader(loader.name, loader)
    module = importlib.util.module_from_spec(spec)
    loader.exec_module(module)
    return module


class ThreadCountValidatorTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.cli = _load_cli()

    def test_accepts_boundaries(self):
        self.assertEqual(self.cli._thread_count("1"), 1)
        self.assertEqual(self.cli._thread_count("40"), 40)
        self.assertEqual(self.cli._thread_count("5"), 5)

    def test_rejects_zero_negative_and_huge(self):
        import argparse

        for bad in ("0", "-5", "41", "500", "abc", ""):
            with self.subTest(value=bad):
                with self.assertRaises(argparse.ArgumentTypeError):
                    self.cli._thread_count(bad)


class ThreadFlagCliTests(unittest.TestCase):
    def _run(self, *extra):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        audio = Path(tmp.name) / "audio"
        out = Path(tmp.name) / "out"
        audio.mkdir()
        out.mkdir()
        return subprocess.run(
            [sys.executable, str(CLI_PATH),
             "--audio-dir", str(audio), "--output-dir", str(out), *extra],
            stdin=subprocess.DEVNULL,
            capture_output=True, text=True, check=False,
        )

    def test_search_threads_zero_fails_fast(self):
        proc = self._run("--search-threads", "0")
        self.assertEqual(proc.returncode, 2)
        self.assertIn("--search-threads", proc.stderr)
        self.assertNotIn("Traceback", proc.stderr)

    def test_download_threads_out_of_range_fails_fast(self):
        for bad in ("0", "-5", "500"):
            with self.subTest(value=bad):
                proc = self._run("--download-threads", bad)
                self.assertEqual(proc.returncode, 2)
                self.assertIn("--download-threads", proc.stderr)
                self.assertNotIn("Traceback", proc.stderr)


if __name__ == "__main__":
    unittest.main()
