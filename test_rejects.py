"""Invalid programs in rejects/ must be rejected before they run.

Each `rejects/<name>.rs` starts with `// error: <regex>` naming the expected diagnostic. Programs
rustc rejects but this checker still accepts carry a `// known-gap: <reason>` line and are skipped.
"""

from pathlib import Path
import re
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parent
REJECTS = sorted((ROOT / "rejects").glob("*.rs"))


def expectation(path):
    lines = path.read_text(encoding="utf-8").splitlines()
    match = re.fullmatch(r"// error: (.+)", lines[0])
    if not match:
        raise AssertionError(f"{path.name} must start with '// error: <regex>'")
    gap = next((line for line in lines[1:] if line.startswith("// known-gap")), None)
    return match.group(1), gap


class RejectedPrograms(unittest.TestCase):
    def test_rejected_before_running(self):
        self.assertGreater(len(REJECTS), 0)
        for path in REJECTS:
            pattern, gap = expectation(path)
            with self.subTest(program=path.name):
                if gap:
                    self.skipTest(f"{path.name}: {gap}")
                result = subprocess.run([sys.executable, str(ROOT / "rust_in_python.py"), str(path)],
                                        capture_output=True, text=True, timeout=120, check=False)
                self.assertNotEqual(result.returncode, 0, f"{path.name} was accepted")
                self.assertEqual(result.stdout, "", f"{path.name} ran before being rejected")
                self.assertRegex(result.stderr, pattern)


if __name__ == "__main__":
    unittest.main()
