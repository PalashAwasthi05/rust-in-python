"""Record rustc's output for each program in this folder as the expected output."""

from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory


def bless(paths):
    with TemporaryDirectory() as scratch:
        for source in paths:
            binary = Path(scratch) / source.stem
            subprocess.run(["rustc", "--edition=2021", "-C", "overflow-checks=yes", "-A", "warnings",
                            str(source), "-o", str(binary)], check=True)
            run = subprocess.run([str(binary)], capture_output=True, text=True, timeout=30)
            source.with_suffix(".out").write_text(f"{run.returncode}\n{run.stdout}", encoding="utf-8")
            print(f"blessed {source.name}")


if __name__ == "__main__":
    folder = Path(__file__).parent
    bless([Path(arg) for arg in sys.argv[1:]] or sorted(folder.glob("*.rs")))
