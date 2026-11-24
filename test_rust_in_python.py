"""Regression cases that also run against rustc when it is installed."""

from io import StringIO
from pathlib import Path
import shutil
import subprocess
from tempfile import TemporaryDirectory
import unittest

from rust_in_python import DEMO, RustError, run


VALID = [
    ("factorial", DEMO, "1! = 1\n2! = 2\n3! = 6\n4! = 24\n5! = 120\n6! = 720\n"),
    ("legacy loops and arithmetic", """
        fn main() {
            let mut sum = 0;
            for n in 0..=7 {
                if n == 3 { continue; }
                if n == 6 { break; }
                sum += n;
            }
            while sum < 15 { sum += 1; }
            println!("{sum} {} {}", -7 / 3, -7 % 3);
        }
    """, "15 -2 -1\n"),
    ("named and tuple struct fields", """
        struct Inner(i32, i32);
        struct Outer { inner: Inner, active: bool }
        fn main() {
            let mut value = Outer { inner: Inner(7, 8), active: true };
            value.inner.0 += 1;
            value.active = !value.active;
            let Outer { inner: Inner(a, b), .. } = value;
            println!("{a} {b} {}", value.active);
        }
    """, "8 8 false\n"),
    ("field evaluation order", """
        struct Pair { a: i32, b: i32 }
        fn say(n: i32) -> i32 { println!("{n}"); n }
        fn main() {
            let value = Pair { b: say(2), a: say(1) };
            println!("{} {}", value.a, value.b);
        }
    """, "2\n1\n1 2\n"),
    ("nested enum coverage", """
        struct Flag(bool);
        enum Item { Stop, Flag(Flag), Record { left: bool, right: bool } }
        fn score(item: Item) -> i32 {
            match item {
                Item::Stop => 0,
                Item::Flag(Flag(true)) => 1,
                Item::Flag(Flag(false)) => 2,
                Item::Record { left: false, .. } => 3,
                Item::Record { left: true, right: false } => 4,
                Item::Record { left: true, right: true } => 5,
            }
        }
        fn main() {
            println!("{} {}", score(Item::Flag(Flag(false))),
                score(Item::Record { left: true, right: true }));
        }
    """, "2 5\n"),
    ("integer range coverage", """
        fn classify(n: i32) -> i32 {
            match n { ..0 => -1, 0 => 0, 1..=i32::MAX => 1 }
        }
        fn main() {
            println!("{} {} {}", classify(i32::MIN), classify(0), classify(i32::MAX));
        }
    """, "-1 0 1\n"),
    ("unit struct", """
        struct Ready;
        fn main() {
            let Ready = Ready;
            match Ready { Ready => println!("ready") }
        }
    """, "ready\n"),
    ("guard retries alternative bindings", """
        enum E { Pair(i32, i32) }
        fn main() {
            let mut hits = 0;
            match E::Pair(1, 2) {
                E::Pair(x, _) | E::Pair(_, x) if { hits += 1; x == 2 } => println!("{x}"),
                _ => println!("no"),
            }
            println!("{hits}");
        }
    """, "2\n2\n"),
    ("pattern conditionals and let-else", """
        enum E { Pair(i32), End }
        fn take(n: i32) -> E { if n < 2 { E::Pair(n) } else { E::End } }
        fn print_one(value: E) {
            let E::Pair(x) = value else { return; };
            println!("{x}");
        }
        fn main() {
            let mut n = 0;
            while let E::Pair(x) = take(n) { println!("{x}"); n += 1; }
            if let E::Pair(x) = take(n) { println!("{x}"); } else { println!("end"); }
            print_one(take(n));
            print_one(take(1));
        }
    """, "0\n1\nend\n1\n"),
    ("empty enum", """
        enum Empty {}
        fn never(value: Empty) -> ! { match value {} }
        fn main() {}
    """, ""),
]

INVALID = [
    ("missing variant", """
        enum E { A, B }
        fn main() { match E::A { E::A => () } }
    """, "non-exhaustive match"),
    ("missing nested case", """
        enum E { A(bool, bool) }
        fn main() { match E::A(true, true) { E::A(false, _) => (), E::A(true, false) => () } }
    """, r"missing E::A\(true, true\)"),
    ("guard does not cover", """
        fn main() { match true { true if true => (), false => () } }
    """, "non-exhaustive match"),
    ("integer gap", """
        fn main() { match 0 { ..0 => (), 1.. => () } }
    """, "missing 0"),
    ("inconsistent alternative bindings", """
        enum E { A(i32), B(bool) }
        fn main() { match E::A(1) { E::A(x) | E::B(x) => { let _ = x; } } }
    """, "all alternatives must bind the same names and types"),
    ("refutable binding", """
        enum E { A(i32), B }
        fn main() { let E::A(x) = E::B; }
    """, "refutable let pattern"),
    ("let-else must diverge", """
        enum E { A(i32), B }
        fn main() { let E::A(x) = E::B else { () }; }
    """, "expected !"),
    ("wrong field type", """
        struct Point { x: bool }
        fn main() { let _ = Point { x: 4 }; }
    """, "expected bool"),
    ("immutable field", """
        struct Point { x: i32 }
        fn main() { let point = Point { x: 0 }; point.x = 4; }
    """, "immutable binding"),
    ("recursive layout", """
        struct Node { next: Node }
        fn main() {}
    """, "infinite size"),
]


class InterpreterTests(unittest.TestCase):
    def test_valid_programs(self):
        for name, source, expected in VALID:
            with self.subTest(name=name):
                output = StringIO()
                run(source, output)
                self.assertEqual(output.getvalue(), expected)

    def test_invalid_programs(self):
        for name, source, message in INVALID:
            with self.subTest(name=name):
                with self.assertRaisesRegex(RustError, message):
                    run(source, StringIO())

    @unittest.skipUnless(shutil.which("rustc"), "rustc is optional")
    def test_rustc_agrees(self):
        with TemporaryDirectory() as folder:
            source_file = Path(folder) / "case.rs"
            binary = Path(folder) / "case"
            for name, source, expected in VALID:
                with self.subTest(name=name):
                    compiled = self.compile(source_file, binary, source)
                    self.assertEqual(compiled.returncode, 0, compiled.stderr)
                    executed = subprocess.run([binary], capture_output=True, text=True, timeout=10)
                    self.assertEqual(executed.returncode, 0, executed.stderr)
                    self.assertEqual(executed.stdout, expected)
            for name, source, _ in INVALID:
                with self.subTest(name=name):
                    compiled = self.compile(source_file, binary, source)
                    self.assertNotEqual(compiled.returncode, 0)

    @staticmethod
    def compile(source_file, binary, source):
        source_file.write_text(source, encoding="utf-8")
        return subprocess.run(
            ["rustc", "--edition=2021", "-C", "overflow-checks=yes", source_file, "-o", binary],
            capture_output=True,
            text=True,
            timeout=30,
        )


if __name__ == "__main__":
    unittest.main()
