"""Compare executable Rust crate roots against rustc and the interpreter."""

import argparse
from dataclasses import asdict, dataclass
import json
import math
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory


INTERPRETER = Path(__file__).with_name("rust_in_python.py").resolve()


@dataclass
class Result:
    status: int | None
    stdout: str
    stderr: str
    timed_out: bool = False

    @property
    def succeeded(self):
        return self.status == 0 and not self.timed_out

    @property
    def crashed(self):
        return self.status is None or self.status < 0 or "Traceback (most recent call last):" in self.stderr


def invoke(command, timeout, *, directory):
    try:
        completed = subprocess.run(command, cwd=directory, text=True, encoding="utf-8",
                                   errors="replace", capture_output=True, timeout=timeout, check=False)
    except subprocess.TimeoutExpired as error:
        stdout, stderr = (part.decode("utf-8", errors="replace") if isinstance(part, bytes) else part or ""
                          for part in (error.stdout, error.stderr))
        return Result(None, stdout, stderr, timed_out=True)
    return Result(completed.returncode, completed.stdout, completed.stderr)


def compare(source, *, rustc, edition, timeout):
    directory = source.parent
    python = [sys.executable, str(INTERPRETER)]
    report = {"source": str(source), "edition": edition}
    with TemporaryDirectory(prefix="rust-conformance-") as scratch:
        binary = Path(scratch) / "native"
        native = invoke([rustc, f"--edition={edition}", "-C", "overflow-checks=yes",
                         str(source), "-o", str(binary)], timeout, directory=directory)
        interpreted = invoke([*python, "--check", str(source)], timeout, directory=directory)
        report["compile"] = {"rustc": asdict(native), "python": asdict(interpreted)}
        if native.timed_out or interpreted.timed_out:
            report["outcome"] = "compile_timeout"
        elif native.crashed or interpreted.crashed or "internal compiler error:" in native.stderr:
            report["outcome"] = "compile_crash"
        elif native.succeeded != interpreted.succeeded:
            report["outcome"] = "acceptance_mismatch"
        elif not native.succeeded:
            report["outcome"] = "both_rejected"
        else:
            rust_run = invoke([str(binary)], timeout, directory=directory)
            python_run = invoke([*python, str(source)], timeout, directory=directory)
            report["run"] = {"rustc": asdict(rust_run), "python": asdict(python_run)}
            if rust_run.timed_out or python_run.timed_out:
                report["outcome"] = "runtime_timeout"
            elif rust_run.crashed or python_run.crashed:
                report["outcome"] = "runtime_crash"
            elif rust_run.status != python_run.status or rust_run.stdout != python_run.stdout:
                report["outcome"] = "runtime_mismatch"
            else:
                report["outcome"] = "runtime_match"
    return report


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("sources", nargs="+", type=Path, help="Rust crate roots to compare")
    parser.add_argument("--rustc", default="rustc", help="path to the reference compiler")
    parser.add_argument("--edition", choices=("2015", "2018", "2021", "2024"), default="2021")
    parser.add_argument("--timeout", type=float, default=30.0, help="seconds per command")
    parser.add_argument("--json", action="store_true", help="include subprocess output and diagnostics")
    args = parser.parse_args(argv)
    if not math.isfinite(args.timeout) or args.timeout <= 0:
        parser.error("timeout must be finite and positive")
    sources = [path.resolve() for path in args.sources]
    if any(not path.is_file() for path in sources):
        parser.error("each source must be an existing file")
    rustc = str(Path(args.rustc).resolve()) if "/" in args.rustc else args.rustc
    try:
        compiler = invoke([rustc, "--version", "--verbose"], args.timeout, directory=Path.cwd())
        if not compiler.succeeded:
            parser.error("could not identify the reference compiler")
        cases = [compare(source, rustc=rustc, edition=args.edition, timeout=args.timeout)
                 for source in sources]
    except OSError as error:
        parser.error(str(error))
    if args.json:
        print(json.dumps({"rustc": compiler.stdout.strip(), "cases": cases}, indent=2))
    else:
        print(compiler.stdout.splitlines()[0])
        for case in cases:
            outcome = case["outcome"].replace("_", " ")
            print(f"{outcome:22} {case['source']}")
    return int(any(case["outcome"] not in {"both_rejected", "runtime_match"} for case in cases))


if __name__ == "__main__":
    sys.exit(main())
