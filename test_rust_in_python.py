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
    ("partial move and field reinitialization", """
        struct Data(i32);
        struct Pair { left: Data, right: Data }
        fn main() {
            let mut pair = Pair { left: Data(1), right: Data(2) };
            let Data(first) = pair.left;
            let left = pair.left;
            let right = pair.right;
            pair.left = Data(first + left.0);
            pair.right = Data(right.0);
            let rebuilt = pair;
            println!("{} {}", rebuilt.left.0, rebuilt.right.0);
        }
    """, "2 2\n"),
    ("copy clone and debug derives", """
        #[derive(Clone, Copy, Debug)]
        struct Coord(i32);
        #[derive(Debug)]
        struct Point { x: Coord }
        #[derive(Clone, Debug)]
        enum Label { Active(&'static str), Pending() }
        fn main() {
            let coord = Coord(7);
            let first = coord;
            let second = coord;
            let label = Label::Active("now");
            let cloned = label.clone();
            println!("{:?} {:?} {label:?} {cloned:?} {:?}",
                Point { x: first }, second, Label::Pending());
        }
    """, 'Point { x: Coord(7) } Coord(7) Active("now") Active("now") Pending\n'),
    ("immutable delayed initialization across branches", """
        struct Data(i32);
        fn get(flag: bool) -> Data {
            let x: Data;
            if flag { x = Data(1); } else { x = Data(2); }
            x
        }
        fn main() { println!("{} {}", get(true).0, get(false).0); }
    """, "1 2\n"),
    ("loop reinitialization and break", """
        struct Data(i32);
        fn consume(_: Data) {}
        fn main() {
            let mut value = Data(1);
            for n in 2..4 {
                consume(value);
                value = Data(n);
                if n == 2 { continue; }
            }
            let result: Data;
            loop { result = value; break; }
            println!("{}", result.0);
        }
    """, "3\n"),
    ("wildcards only inspect aggregate shape", """
        struct Data(i32);
        fn main() {
            let uninitialized: Data;
            let Data(_) = uninitialized;
            let initialized = Data(2);
            let taken = initialized;
            let _ = initialized;
            match initialized { Data(_) => println!("{}", taken.0) }
        }
    """, "2\n"),
    ("failed guard preserves value", """
        struct Data(i32);
        fn use_value(value: Data) -> i32 { value.0 }
        fn main() {
            let value = Data(7);
            let score = match value {
                x if x.0 == 2 => use_value(x),
                _ => use_value(value),
            };
            println!("{score}");
        }
    """, "7\n"),
    ("return does not pollute a merge", """
        struct Data(i32);
        fn consume(_: Data) {}
        fn get(flag: bool) -> i32 {
            let data = Data(3);
            if flag { consume(data); return 2; }
            data.0
        }
        fn main() { println!("{} {}", get(true), get(false)); }
    """, "2 3\n"),
    ("labeled loops return moved values", """
        struct Data(i32);
        fn main() {
            let mut n = 0;
            let result = 'outer: loop {
                n += 1;
                for i in 0..2 {
                    if n < 3 { continue 'outer; }
                    break 'outer Data(n + i);
                }
            };
            let answer = 'finish: {
                let Data(x) = result;
                if x == 3 { break 'finish x; }
                0
            };
            println!("{answer}");
        }
    """, "3\n"),
    ("break through a labeled while condition", """
        fn main() {
            let mut n = 0;
            'outer: while {
                n += 1;
                if n == 3 { break 'outer; }
                true
            } {}
            println!("{n}");
        }
    """, "3\n"),
    ("tuple rest skips unread fields", """
        struct Data(i32);
        struct Row(Data, i32, i32);
        fn main() {
            let row = Row(Data(1), 2, 3);
            let moved = row.0;
            let Row(.., last) = row;
            println!("{} {last}", moved.0);
        }
    """, "1 3\n"),
    ("guard permits writing unrelated field", """
        struct Pair { first: i32, second: i32 }
        fn main() {
            let mut p = Pair { first: 0, second: 1 };
            match p {
                Pair { first: 0, .. } if { p.second = 2; true } => (),
                _ => (),
            }
            println!("{}", p.second);
        }
    """, "2\n"),
    ("shared borrow ends at last use", """
        fn main() {
            let mut value = 1;
            let first = &value;
            let second = first;
            println!("{first} {second}");
            value = 2;
            println!("{value}");
        }
    """, "1 1\n2\n"),
    ("disjoint fields borrowed across calls", """
        struct Pair { left: i32, right: i32 }
        fn add(x: &mut i32, y: &i32) { *x += *y; }
        fn main() {
            let mut pair = Pair { left: 1, right: 2 };
            let left = &mut pair.left;
            let right = &pair.right;
            add(left, right);
            add(left, right);
            println!("{} {}", pair.left, pair.right);
        }
    """, "5 2\n"),
    ("split mutable reborrow", """
        struct Pair { left: i32, right: i32 }
        fn main() {
            let mut pair = Pair { left: 1, right: 2 };
            let whole = &mut pair;
            let left = &mut whole.left;
            let right = &mut whole.right;
            *left += 3;
            *right += 4;
            println!("{} {}", whole.left, whole.right);
        }
    """, "4 6\n"),
    ("mutable argument reservation allows a later read", """
        fn add(x: &mut i32, delta: i32) { *x += delta; }
        fn main() {
            let mut value = 3;
            let handle = &mut value;
            add(handle, *handle);
            add(handle, *handle);
            println!("{value}");
        }
    """, "12\n"),
    ("shared coercion from mutable reference", """
        fn sum(x: &i32, y: &i32) -> i32 { *x + *y }
        fn main() {
            let mut value = 3;
            let handle = &mut value;
            let shared: &i32 = handle;
            println!("{}", sum(shared, handle));
            *handle += 1;
            println!("{value}");
        }
    """, "6\n4\n"),
    ("temporary values can be borrowed", """
        fn sum(a: &i32, b: &i32) -> i32 { *a + *b }
        fn main() {
            let x = 3;
            let temp = &mut { x + 2 };
            *temp += 1;
            println!("{}", sum(temp, &2));
        }
    """, "8\n"),
    ("branch-sensitive borrow uses", """
        fn use_value(flag: bool) {
            let mut value = 1;
            let shared = &value;
            if flag { value = 2; println!("{value}"); } else { println!("{shared}"); }
        }
        fn main() { use_value(true); use_value(false); }
    """, "2\n1\n"),
    ("moved aggregate replaced after its borrow ends", """
        #[derive(Clone, Debug)]
        struct Item(i32);
        fn replace(place: &mut Item, value: Item) { *place = value; }
        fn main() {
            let mut item = Item(2);
            let cloned = (&item);
            let copied = cloned.clone();
            replace(&mut item, Item(5));
            println!("{copied:?} {item:?}");
        }
    """, "Item(2) Item(5)\n"),
    ("borrow recreated on loop iteration", """
        fn main() {
            let mut value = 0;
            for n in 1..4 {
                let handle = &mut value;
                *handle += n;
                println!("{value}");
            }
        }
    """, "1\n3\n6\n"),
    ("temporary references can be dereferenced immediately", """
        struct Point { x: i32 }
        fn main() {
            let mut point = Point { x: 1 };
            (&mut point).x += 1;
            *(&mut point.x) += 1;
            let shared = &*(&point);
            println!("{}", shared.x);
        }
    """, "3\n"),
    ("diverging reference initializer", """
        fn main() {
            let missing: &i32 = return;
            println!("{missing}");
        }
    """, ""),
    ("elided reference returned from a field", """
        struct Pair { left: i32, right: i32 }
        fn left(value: &mut Pair) -> &mut i32 { &mut value.left }
        fn main() {
            let mut pair = Pair { left: 1, right: 2 };
            *left(&mut pair) += 2;
            let selected = left(&mut pair);
            *selected += 1;
            println!("{} {}", pair.left, pair.right);
        }
    """, "4 2\n"),
    ("named lifetime ties multiple inputs through match", """
        fn pick<'a>(first: &'a i32, second: &'a i32, flag: bool) -> &'a i32 {
            match flag { true => first, false => second }
        }
        fn main() {
            let a = 1;
            let b = 2;
            let chosen = pick(&a, &b, false);
            println!("{chosen}");
        }
    """, "2\n"),
    ("outlives bounds are transitive", """
        fn choose<'a, 'b: 'a, 'c>(a: &'a i32, b: &'b i32, c: &'c i32) -> &'a i32
        where 'c: 'b {
            c
        }
        fn main() { println!("{}", choose(&1, &2, &3)); }
    """, "3\n"),
    ("early reference return and promoted fallback", """
        fn pick(first: &'_ i32, flag: bool) -> &'_ i32 {
            if flag { return first; }
            &42
        }
        fn main() { println!("{} {}", pick(&1, true), pick(&2, false)); }
    """, "1 42\n"),
    ("immutable static references and constant promotion", """
        fn number() -> &'static i32 { &{ 6 * 7 } }
        fn keep(value: &'static i32) -> &'static i32 { value }
        fn main() {
            let data: &'static i32 = keep(number());
            println!("{data}");
        }
    """, "42\n"),
    ("mutable return shares its lifetime with an immutable parameter", """
        fn pick<'a>(ignored: &'a i32, chosen: &'a mut i32) -> &'a mut i32 { chosen }
        fn main() {
            let other = 1;
            let mut value = 2;
            let chosen = pick(&other, &mut value);
            println!("{other}");
            *chosen += 1;
            println!("{value}");
        }
    """, "1\n3\n"),
    ("unused returned reference to an argument temporary", """
        fn make() -> i32 { 7 }
        fn same(value: &i32) -> &i32 { value }
        fn main() {
            let _unused = same(&make());
            println!("{}", same(&make()));
        }
    """, "7\n"),
    ("explicitly extended temporary survives later call", """
        fn make() -> i32 { 7 }
        fn same(value: &i32) -> &i32 { value }
        fn main() {
            let temp = { &make() };
            let borrowed = same(temp);
            println!("{borrowed}");
        }
    """, "7\n"),
    ("returned reborrow ends before parent reference is reused", """
        fn same(value: &mut i32) -> &mut i32 { &mut *value }
        fn main() {
            let mut value = 1;
            let parent = &mut value;
            let child = same(parent);
            *child += 1;
            *parent += 2;
            println!("{value}");
        }
    """, "4\n"),
    ("borrowed owner initialized on its own control-flow path", """
        struct Item(i32);
        fn drop_item(_: Item) {}
        fn choose(flag: bool) {
            let first = Item(1);
            let second = Item(2);
            let borrowed = if flag {
                drop_item(first);
                &second
            } else {
                drop_item(second);
                &first
            };
            println!("{}", borrowed.0);
        }
        fn main() { choose(true); choose(false); }
    """, "2\n1\n"),
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
    ("repeat move", """
        struct Data(i32);
        fn main() { let data = Data(1); let a = data; let b = data; }
    """, "use of moved value"),
    ("read whole after partial move", """
        struct Data(i32);
        struct Pair(Data, Data);
        fn main() { let pair = Pair(Data(1), Data(2)); let a = pair.0; let b = pair; }
    """, "partially moved"),
    ("write into moved field", """
        struct Data(i32);
        struct Pair(Data, Data);
        fn main() { let mut pair = Pair(Data(1), Data(2)); let old = pair.0; pair.0.0 = 3; }
    """, "field of moved value"),
    ("possibly moved in branch", """
        struct Data(i32);
        fn consume(_: Data) {}
        fn f(flag: bool) { let data = Data(1); if flag { consume(data); } consume(data); }
        fn main() {}
    """, "use of moved value"),
    ("move across continue back edge", """
        struct Data(i32);
        fn consume(_: Data) {}
        fn main() {
            let mut data = Data(1);
            for n in 1..4 { consume(data); if n == 2 { continue; } data = Data(n); }
        }
    """, "use of moved value"),
    ("possibly uninitialized", """
        fn f(flag: bool) -> i32 { let x: i32; if flag { x = 2; } x }
        fn main() {}
    """, "use of uninitialized value"),
    ("move not permission to reassign immutable", """
        struct Data(i32);
        fn main() { let data = Data(1); let saved = data; data = Data(2); }
    """, "immutable binding"),
    ("copy requires copy fields", """
        #[derive(Clone)]
        struct Data(i32);
        #[derive(Clone, Copy)]
        struct Wrapper(Data);
        fn main() {}
    """, "Data does not implement Copy"),
    ("pattern guard cannot move from binding", """
        struct Data(i32);
        fn consume(_: Data) -> bool { true }
        fn main() { match Data(1) { data if consume(data) => (), _ => () } }
    """, "in a pattern guard"),
    ("print cannot borrow moved value", """
        #[derive(Debug)]
        struct Data(i32);
        fn main() { let data = Data(1); let saved = data; println!("{data:?}"); }
    """, "use of moved value"),
    ("guard cannot change future pattern test", """
        struct Pair { first: i32, second: i32 }
        fn main() {
            let mut p = Pair { first: 0, second: 1 };
            match p {
                Pair { first: 0, .. } if { p.second = 2; true } => (),
                Pair { second: 1, .. } => (),
                _ => (),
            }
        }
    """, "in a pattern guard"),
    ("break values must agree", """
        fn f(flag: bool) -> i32 { loop { if flag { break 1; } break false; } }
        fn main() {}
    """, "expected i32, found bool"),
    ("while let cannot return a break value", """
        enum E { A, B }
        fn main() { while let E::A = E::B { break 1; } }
    """, "break values require loop or a labeled block"),
    ("continue cannot target a block", """
        fn main() { 'outer: { loop { continue 'outer; } } }
    """, "continue requires a loop label"),
    ("labeled break carries move to exit", """
        struct Data(i32);
        fn main() {
            let data = Data(1);
            let moved = 'exit: { loop { break 'exit data; } };
            let again = data;
        }
    """, "use of moved value"),
    ("shared alias retains loan", """
        fn main() {
            let mut value = 1;
            let shared = &value;
            let alias = shared;
            value = 2;
            println!("{alias}");
        }
    """, "immutably borrowed"),
    ("no read through owner of active mutable loan", """
        fn main() {
            let mut value = 1;
            let handle = &mut value;
            let copy = value;
            *handle += copy;
        }
    """, "mutably borrowed"),
    ("parent loan stays live through reborrow", """
        struct Pair { left: i32, right: i32 }
        fn main() {
            let mut pair = Pair { left: 1, right: 2 };
            let handle = &mut pair;
            let child = &mut handle.left;
            pair.right = 4;
            *child = 3;
        }
    """, "mutably borrowed"),
    ("second mutable borrow overlaps", """
        fn main() {
            let mut value = 1;
            let first = &mut value;
            let second = &mut value;
            *first += *second;
        }
    """, "mutably borrowed"),
    ("cannot borrow immutable binding as mutable", """
        fn main() { let value = 1; let _ = &mut value; }
    """, "cannot borrow immutable place"),
    ("cannot borrow uninitialized value", """
        fn main() { let value: i32; let _ = &value; }
    """, "uninitialized value"),
    ("cannot write through shared reference", """
        fn main() { let value = 1; let shared = &value; *shared = 2; }
    """, "cannot write through a shared reference"),
    ("cannot move noncopy dereference", """
        struct Item(i32);
        fn main() { let item = Item(1); let shared = &item; let moved = *shared; }
    """, "cannot move out of borrowed place"),
    ("mutable reference moves when assigned", """
        fn main() { let mut value = 1; let first = &mut value; let alias = first; *first = 2; *alias = 3; }
    """, "use of moved reference"),
    ("two mutable reservations cannot activate together", """
        fn add(left: &mut i32, right: &mut i32) { *left += *right; }
        fn main() { let mut value = 1; let handle = &mut value; add(handle, handle); }
    """, "cannot borrow"),
    ("explicit mutable argument does not reserve", """
        fn add(left: &mut i32, right: i32) { *left += right; }
        fn main() { let mut value = 1; add(&mut value, value); }
    """, "mutably borrowed"),
    ("loop back edge keeps a shared borrow alive", """
        fn main() {
            let mut value = 1;
            let shared = &value;
            for n in 0..2 { println!("{shared}"); value = n; }
        }
    """, "immutably borrowed"),
    ("format arguments retain loans together", """
        fn main() { let mut value = 1; println!("{:?} {:?}", &mut value, &value); }
    """, "mutably borrowed"),
    ("ambiguous output lifetime even with same named input lifetimes", """
        fn same<'a>(first: &'a i32, second: &'a i32) -> &i32 { second }
        fn main() {}
    """, "missing lifetime for reference return"),
    ("cannot explicitly return a reference to a local", """
        fn bad(input: &i32) -> &i32 { let local = 1; return &local; }
        fn main() {}
    """, "cannot return a reference to 'local'"),
    ("cannot return shorter unrelated lifetime", """
        fn bad<'a, 'b>(first: &'a i32, second: &'b i32) -> &'a i32 { second }
        fn main() {}
    """, "with the declared lifetime"),
    ("mutable field return keeps whole loan from call", """
        struct Pair { left: i32, right: i32 }
        fn left(value: &mut Pair) -> &mut i32 { &mut value.left }
        fn main() {
            let mut pair = Pair { left: 1, right: 2 };
            let selected = left(&mut pair);
            pair.right = 3;
            *selected += 1;
        }
    """, "mutably borrowed"),
    ("shared return keeps both possible input loans", """
        fn pick<'a>(a: &'a i32, b: &'a i32) -> &'a i32 { a }
        fn main() {
            let a = 1;
            let mut b = 2;
            let selected = pick(&a, &b);
            b = 3;
            println!("{selected}");
        }
    """, "immutably borrowed"),
    ("return through a call cannot extend a temporary", """
        fn make() -> i32 { 7 }
        fn same(value: &i32) -> &i32 { value }
        fn main() { let reference = same(&make()); println!("{reference}"); }
    """, "temporary value dropped while borrowed"),
    ("return through a call cannot extend a mutable literal", """
        fn same(value: &mut i32) -> &mut i32 { value }
        fn main() { let reference = same(&mut 7); *reference += 1; }
    """, "temporary value dropped while borrowed"),
    ("borrowed match binding stays within its arm", """
        fn main() { let reference = match 1 { value => &value }; println!("{reference}"); }
    """, "outlives its block"),
    ("extended temporary stays within defining block", """
        fn make() -> i32 { 7 }
        fn main() {
            let reference = { let inner = &make(); inner };
            println!("{reference}");
        }
    """, "outlives its block"),
    ("local does not satisfy static parameter", """
        fn keep(reference: &'static i32) -> &'static i32 { reference }
        fn main() { let value = 3; let _ignored = keep(&value); }
    """, "does not live for 'static"),
    ("wildcard annotation still requires static lifetime", """
        fn main() {
            let value = 3;
            let borrowed = &value;
            let _: &'static i32 = borrowed;
        }
    """, "does not live for 'static"),
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
