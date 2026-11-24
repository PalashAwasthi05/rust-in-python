"""Regression cases that also run against rustc when it is installed."""

from contextlib import redirect_stderr
from io import StringIO
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
from tempfile import TemporaryDirectory
import unittest

from rust_in_python import DEMO, RustError, RustPanic, run


VALID = [
    ("sibling fields stay disjoint through a borrow selected by a branch", """
        struct Pair { x: i32, y: i32 }
        fn exercise(pick: bool) -> i32 {
            let mut left = Pair { x: 1, y: 2 };
            let mut right = Pair { x: 3, y: 4 };
            let selected = if pick { &mut left } else { &mut right };
            let x = &mut selected.x;
            selected.y = 10;
            *x + selected.y
        }
        fn main() { println!("{} {}", exercise(true), exercise(false)); }
    """, "11 13\n"),
    ("string searches return UTF-8 byte offsets for character and string patterns", """
        fn main() {
            let text = String::from("é🦀β🦀");
            let crab = String::from("🦀");
            println!("{} {} {} {:?} {:?} {:?} {:?}", text.contains(&crab), text.starts_with('é'),
                     text.ends_with("β🦀"), text.find('β'), text.rfind(&crab), text.find('x'), text.rfind(""));
        }
    """, "true true true Some(6) Some(8) None Some(12)\n"),
    ("uninitialized patterns declare independent locals of the inferred field types", """
        struct Record { a: u8, b: u16 }
        fn main() {
            let (number, flag);
            (number, flag) = (5, true);
            let _: u8 = number;
            let [left, right];
            [left, right] = [2, 3u16];
            let Record { a, b };
            (a, b) = (1, 256);
            println!("{number} {flag} {left} {right} {a} {b}");
        }
    """, "5 true 2 3 1 256\n"),
    ("struct update evaluates explicit fields before the base and moves only inherited fields", """
        struct Pair<T> { replaced: T, retained: T }
        fn value() -> String { print!("explicit "); String::from("replacement") }
        fn base() -> Pair<String> {
            print!("base ");
            Pair { replaced: String::from("old"), retained: String::from("retained") }
        }
        fn main() {
            let old = Pair { replaced: String::from("old"), retained: String::from("retained") };
            let new = Pair { replaced: value(), ..old };
            println!("{} {} {}", old.replaced, new.replaced, new.retained);
            let replaced = Pair { replaced: value(), retained: String::from("explicit"), ..base() };
            println!("{}", replaced.retained);
        }
    """, "explicit old replacement retained\nexplicit base explicit\n"),
    ("destructuring assignment evaluates the right side before destination places", """
        struct Record { first: i32, second: i32 }
        struct Tuple(i32, i32, i32);
        fn rhs() -> (u8, u8) { print!("rhs "); (7, 8) }
        fn index() -> usize { print!("index "); 0 }
        fn main() {
            let (mut a, mut b) = (1u8, 2u8);
            (a, b) = (b, a);
            let mut slots = [0u8; 2];
            (slots[index()], slots[1]) = rhs();
            let piece;
            [piece, ..] = [4, 5, 6];
            let first;
            let end;
            Tuple(first, .., end) = Tuple(10, 20, 30);
            let field;
            Record { second: field, .. } = Record { first: 1, second: 6 };
            println!("{a} {b} {slots:?} {piece} {first} {end} {field}");
        }
    """, "rhs index 2 1 [7, 8] 4 10 30 6\n"),
    ("destructuring assignment skips discarded places and infers nested tuple elements", """
        struct Marker;
        fn main() {
            let taken = (String::from("keep"), String::from("moved"));
            let moved;
            (_, moved) = taken;
            _ = taken.0;
            let a: u8;
            let b: u64;
            ((a, ..), .., b) = ((5, true), "ignored", 8);
            (..) = (String::from("drop"),);
            Marker = Marker;
            println!("{} {moved} {a} {b}", taken.0);
        }
    """, "keep moved 5 8\n"),
    ("built-in macro text preserves token boundaries and literal contents", """
        macro_rules! original { ($item:expr) => { stringify!($item) }; }
        macro_rules! merged {
            ($left:ident, $right:ident) => { stringify!($left + $right) };
        }
        fn main() {
            let combined = concat!("a", 'β', 0x20, 1_000, 1.20, 2f32, true, -12, 3.5e4);
            let spaced = stringify!( foo /* comment */ +   bar );
            let path = std::concat!(r#"raw"#, ':', stringify!(路径::函数));
            println!("{combined}\n{spaced}\n{path}\n{} {}\n{:?}", original!(1+2), merged!(left, right), std::vec![1, 2]);
        }
    """, "aβ3210001.202true-123.5e4\nfoo + bar\nraw:路径::函数\n1+2 left + right\n[1, 2]\n"),
    ("local inner macros use the defining crate's exported helpers", """
        mod provider {
            #[macro_export]
            macro_rules! helper {
                () => { 7 };
            }
            #[macro_export(local_inner_macros)]
            macro_rules! calculate {
                ($input:expr) => { helper!() + ($input) };
            }
        }
        mod caller {
            macro_rules! helper {
                () => { 5 };
            }
            pub fn value() -> i32 { crate::calculate!(helper!()) }
        }
        fn main() { println!("{}", caller::value()); }
    """, "12\n"),
    ("macro_use imports module and generated macros at their textual positions", """
        macro_rules! inherited { () => { 3 }; }
        #[macro_use]
        mod helpers {
            #[macro_use]
            mod more { macro_rules! extra { () => { 5 }; } }
            macro_rules! combined { () => { inherited!() + extra!() }; }
        }
        macro_rules! introduce {
            () => { macro_rules! generated { () => { 9 }; } };
        }
        fn main() {
            introduce!();
            println!("{} {} {}", combined!(), extra!(), generated!());
        }
    """, "8 5 9\n"),
    ("main can propagate a result and terminate successfully", """
        #[derive(Debug)]
        struct Failure(u8);
        fn lookup() -> Result<u8, Failure> { Ok(7) }
        fn main() -> Result<Result<(), Failure>, Failure> {
            let number = lookup()?;
            println!("{number}");
            Ok(Ok(()))
        }
    """, "7\n"),
    ("Unicode names normalize across source positions and format arguments", """
        fn 量(数量: i32) -> i32 { 数量 + 1 }
        fn main() {
            let cafe\u0301 = 量(6);
            let 宽度 = 5;
            let 结果 = '外: loop { break '外 café; };
            println!("[{结果:>宽度$}]");
        }
    """, "[    7]\n"),
    ("raw identifiers remain available in imports, fields, and macros", """
        macro_rules! r#match { ($r#type:expr) => { $r#type }; }
        mod r#impl { pub fn r#type(value: i32) -> i32 { value + 1 } }
        use r#impl::r#type as r#fn;
        struct Row { r#loop: i32 }
        fn raw_lifetime<'r#type>(value: &'r#type i32) -> &'r#type i32 { value }
        fn main() {
            let r#false = 3;
            let row = Row { r#loop: r#match!(r#false) };
            let value = 'r#match: loop { break 'r#match r#fn(row.r#loop); };
            println!("{}", raw_lifetime(&value));
        }
    """, "4\n"),
    ("boxed self receivers consume trait objects through dynamic dispatch", """
        trait Drain { fn take(self: Box<Self>) -> String; }
        struct Name { text: String }
        impl Drain for Name {
            fn take(self: Box<Self>) -> String { self.text }
        }
        fn boxed() -> Box<dyn Drain> { Box::new(Name { text: String::from("hello") }) }
        fn generic<T: Drain>(value: Box<T>) -> String { value.take() }
        fn main() {
            let value = boxed();
            let direct = Box::new(Name { text: String::from("world") });
            println!("{} {}", value.take(), generic(direct));
        }
    """, "hello world\n"),
    ("owned trait objects support mutable dispatch and supertrait upcasts", """
        trait Measure { type Value; fn read(&self) -> Self::Value; }
        trait Meter: Measure { fn change(&mut self, n: i32); }
        struct Small(i32);
        struct Big(i32);
        impl Measure for Small { type Value = i32; fn read(&self) -> i32 { self.0 } }
        impl Measure for Big { type Value = i32; fn read(&self) -> i32 { self.0 * 10 } }
        impl Meter for Small { fn change(&mut self, n: i32) { self.0 = n; } }
        impl Meter for Big { fn change(&mut self, n: i32) { self.0 = n; } }
        fn make(choose: bool) -> Box<dyn Meter<Value = i32>> {
            if choose { Box::new(Small(2)) } else { Box::new(Big(2)) }
        }
        fn report(value: &dyn Measure<Value = i32>) -> i32 { value.read() }
        fn main() {
            let mut gauge = make(false);
            gauge.change(3);
            gauge.as_mut().change(4);
            let base: Box<dyn Measure<Value = i32>> = gauge;
            println!("{} {}", base.read(), report(base.as_ref()));
            let small: Box<dyn Measure<Value = i32>> = Box::new(Small(8));
            println!("{}", report(&*small));
        }
    """, "40 40\n8\n"),
    ("boxes own concrete values and clone their contents", """
        use std::boxed::Box as Owner;
        struct Node { value: i32, next: Option<Owner<Node>> }
        fn main() {
            let mut concrete = Owner::<u8>::new(7);
            let clone = concrete.clone();
            *concrete = 9;
            let node = Node { value: 6, next: None };
            println!("{} {:?} {} {} {}", concrete, clone, concrete == clone, node.value, node.next.is_none());
        }
    """, "9 7 false 6 true\n"),
    ("higher-ranked associated equalities preserve the receiver's borrow", """
        trait Lending {
            type Item<'a> where Self: 'a;
            fn get<'a>(&'a self) -> Self::Item<'a>;
        }
        struct Packet(i32);
        impl Lending for Packet {
            type Item<'a> = &'a i32;
            fn get<'a>(&'a self) -> &'a i32 { &self.0 }
        }
        fn borrow<'s, R>(r: &'s R) -> &'s i32 where for<'a> R: Lending<Item<'a> = &'a i32> {
            r.get()
        }
        fn wrap(value: i32) -> impl for<'a> Lending<Item<'a> = &'a i32> { Packet(value) }
        fn main() {
            let item = Packet(17);
            let second = wrap(21);
            let shared = borrow(&item);
            println!("{shared} {}", second.get());
        }
    """, "17 21\n"),
    ("const generic associated bindings normalize exact specializations", """
        trait Read {
            type Item<const N: usize>;
            fn read<const N: usize>(&self) -> Self::Item<N>;
        }
        struct Packet(u8);
        impl Read for Packet {
            type Item<const N: usize> = [u8; N];
            fn read<const N: usize>(&self) -> [u8; N] { [self.0; N] }
        }
        fn get<const N: usize>(source: impl Read<Item<N> = [u8; N]>) -> [u8; N] {
            source.read::<N>()
        }
        fn packet(n: u8) -> impl Read<Item<2> = [u8; 2], Item<3> = [u8; 3]> { Packet(n) }
        fn main() {
            let source = packet(4);
            println!("{:?} {:?} {:?}", get::<1>(Packet(9)), source.read::<2>(), source.read::<3>());
        }
    """, "[9] [4, 4] [4, 4, 4]\n"),
    ("generic associated bindings accept type and lifetime arguments", """
        trait Tagged { type Value<T>; type Word<'a>; }
        struct Source;
        impl Tagged for Source {
            type Value<T> = [T; 1];
            type Word<'a> = u8;
        }
        fn validate(_: impl Tagged<Value<u16> = [u16; 1], Word<'static> = u8>) {}
        fn main() { validate(Source); }
    """, ""),
    ("associated type constraints infer generic results and select blanket implementations", """
        trait Read { type Item; fn read(&self) -> Self::Item; }
        struct Packet<T: Copy>(T);
        impl<T: Copy> Read for Packet<T> {
            type Item = T;
            fn read(&self) -> T { self.0 }
        }
        trait Echo { fn echo(&self) -> u8; }
        impl<T> Echo for T where T: Read<Item = u8> {
            fn echo(&self) -> u8 { self.read() }
        }
        fn get<T, U>(source: T) -> U where T: Read<Item = U> { source.read() }
        fn bytes(source: impl Read<Item = u8>) -> u8 { source.read() }
        fn wide<T: Read<Item = u16>>(source: T) -> u16 { source.read() }
        fn hide() -> impl Read<Item = u8> { Packet(13u8) }
        fn main() {
            let inferred = get(Packet(5u32));
            println!("{} {} {} {}", inferred, bytes(Packet(7u8)), wide(Packet(11u16)), hide().echo());
        }
    """, "5 7 11 13\n"),
    ("bounded generic structs accept matching associated types", """
        trait Read { type Item; }
        struct Source;
        impl Read for Source { type Item = u8; }
        struct Envelope<T: Read<Item = u8>>(T);
        fn main() { let _: Envelope<Source> = Envelope(Source); }
    """, ""),
    ("opaque associated bindings are visible after specialization", """
        trait Read { type Item; fn read(&self) -> Self::Item; }
        trait Source: Read {}
        struct Packet<T: Copy>(T);
        impl<T: Copy> Read for Packet<T> {
            type Item = T;
            fn read(&self) -> T { self.0 }
        }
        impl<T: Copy> Source for Packet<T> {}
        fn packet<T: Copy>(value: T) -> impl Source<Item = T> { Packet(value) }
        fn via_object(source: &dyn Read<Item = u8>) -> u8 { source.read() }
        fn main() {
            let exact: u16 = packet(7u16).read();
            let input = packet(3u8);
            println!("{exact} {}", via_object(&input));
        }
    """, "7 3\n"),
    ("opaque returns support trait calls and generic specialization", """
        trait Number { fn read(&self) -> i32; }
        struct Meter(i32);
        impl Number for Meter { fn read(&self) -> i32 { self.0 } }
        impl Meter { fn hide(self) -> impl Number { self } }
        fn meter(n: i32) -> impl Number { Meter(n) }
        fn wrap<T: Number>(n: T) -> impl Number { n }
        fn relay() -> impl Number { wrap(meter(7)) }
        fn display() -> impl std::fmt::Display { 12u8 }
        fn read_dynamic(value: &dyn Number) -> i32 { value.read() }
        fn main() {
            let first = relay();
            let second = wrap(wrap(Meter(5)));
            println!("{} {} {} {}", first.read(), read_dynamic(&second), Meter(3).hide().read(), display());
        }
    """, "7 5 3 12\n"),
    ("explicit and tail returns use the same opaque type", """
        fn counted(flag: bool) -> impl Copy + std::fmt::Debug {
            if flag { return 3u8; }
            4u8
        }
        fn never_returns() -> impl Copy { never_returns() }
        fn same<T: std::fmt::Debug + Copy>(a: T, b: T) { println!("{a:?} {b:?}"); }
        fn main() { same(counted(true), counted(false)); }
    """, "3 4\n"),
    ("scalar statics have crate-wide storage and static borrow lifetime", """
        macro_rules! bake { ($number:expr) => { $number + 2 }; }
        macro_rules! block_value { () => { const { 3 } }; }
        mod settings { pub static VALUE: u32 = bake!(40u32); }
        use settings::VALUE;
        const CACHED: u32 = VALUE;
        static MIRROR: u32 = CACHED;
        fn read() -> &'static u32 { &MIRROR }
        fn main() {
            let value: &'static u32 = &VALUE;
            let adjustment = block_value!();
            println!("{VALUE} {} {} {adjustment}", *value, read());
        }
    """, "42 42 42 3\n"),
    ("macro item, visibility, and attribute fragments", """
        macro_rules! declare {
            ($access:vis $name:ident, $attribute:meta) => {
                #[$attribute]
                $access struct $name;
            };
        }
        macro_rules! forward {
            ($item:item) => { $item };
        }
        mod api {
            declare!(pub(crate) Exported, derive(Debug));
            declare!(Hidden, derive(Debug));
            forward!(pub fn five() -> i32 { let _ = Hidden; 5 });
        }
        use api::{Exported, five};
        fn main() { println!("{:?} {}", Exported, five()); }
    """, "Exported 5\n"),
    ("macro statement, lifetime, pattern parameter, and literal fragments", """
        macro_rules! region {
            ('static) => { 3 };
            ($other:lifetime) => { 4 };
        }
        macro_rules! forward {
            ($lt:lifetime) => { region!($lt) };
        }
        macro_rules! with_binding {
            ($binding:stmt, $value:expr_2021) => {{ $binding; $value }};
        }
        macro_rules! either {
            ($value:expr, $a:pat_param | $b:pat_param) => {
                match $value { $a | $b => true, _ => false }
            };
        }
        macro_rules! keep {
            ($word:literal) => { $word };
        }
        fn main() {
            let result = with_binding!(let n = 7, n + 1);
            println!("{} {} {} {} {:?} {}", forward!('static), forward!('r), result,
                     either!(2, 1 | 2), keep!(b"ok"), keep!('Ω'));
        }
    """, "3 4 8 true [111, 107] Ω\n"),
    ("separated macro repetitions can capture empty visibility", """
        macro_rules! visibility_items {
            ($($visibility:vis),*) => {
                $($visibility struct Unit;)*
            };
        }
        macro_rules! definitions {
            ($($visibility:vis $item:ident),*) => {
                $($visibility struct $item;)*
            };
        }
        macro_rules! tag { ($visibility:vis marker) => { 1 }; }
        macro_rules! count { ($($visibility:vis),+) => { 0 $(+ tag!($visibility marker))+ }; }
        visibility_items!();
        definitions!(Open, pub Public);
        fn main() { let _ = (Open, Public); println!("{}", count!(,pub)); }
    """, "2\n"),
    ("numeric local types inferred from later uses", """
        fn byte(value: u8) -> u8 { value }
        fn signed(value: i8) -> i8 { value }
        fn narrow(value: f32) -> f32 { value }
        fn main() {
            let mut item = 197;
            item += 3;
            let lower = -128;
            let magnitude = 2.25;
            let width = 5;
            let twice = byte(item + (item / 2 - 100));
            let signed = signed(lower);
            let float = narrow(-magnitude);
            let cast = 4.75;
            println!("[{twice:>width$}] {signed} {float} {}", cast as u8);
        }
    """, "[  200] -128 -2.25 4\n"),
    ("conditional constant functions evaluate only the selected branch", """
        #![feature(generic_const_exprs)]
        const fn width(n: usize) -> usize { if n == 0 { 1 } else { 8 / n } }
        const fn test(n: usize) -> bool { n == 0 || 8 / n == 4 }
        fn repeat<const N: usize>() -> [u8; width(N)] where [(); width(N)]: {
            [7; width(N)]
        }
        const YES: bool = test(0);
        const NO: bool = test(1);
        const EDGE: i8 = if YES { -128i8 } else { 0 };
        fn main() { println!("{:?} {:?} {YES} {NO} {EDGE}", repeat::<0>(), repeat::<2>()); }
    """, "[7] [7, 7, 7, 7] true false -128\n"),
    ("inline constant blocks with generic parameters", """
        fn limit<const N: usize>() -> usize {
            const { let adjustment: usize = 2; N + adjustment }
        }
        fn main() {
            let tiny: u8 = const { 17 };
            let divisor: usize = const { if false { 1 } else { 2 } };
            println!("{} {} {}", limit::<3>(), tiny, divisor);
        }
    """, "5 17 2\n"),
    ("scalar const functions with forward references and runtime calls", """
        mod sizing {
            pub const fn width(x: usize) -> usize {
                let doubled = x * 2;
                doubled + super::PADDING
            }
        }
        const SIZE: usize = sizing::width(3);
        const PADDING: usize = 1;
        use sizing::width;
        type Grid = [u8; width(2)];
        const fn alternate(n: u8) -> u8 { let mask: u8 = 3; n ^ mask }
        fn main() {
            let row: Grid = [alternate(4); width(2)];
            let n = 5;
            println!("{} {} {} {}", SIZE, row.len(), row[0], width(n));
        }
    """, "7 5 7 11\n"),
    ("const functions in dependent array lengths", """
        #![feature(generic_const_exprs)]
        const fn twice(n: usize) -> usize { n * 2 }
        fn repeat<const N: usize>() -> [u8; twice(N)] where [(); twice(N)]: {
            [7; twice(N)]
        }
        fn main() { println!("{:?}", repeat::<2>()); }
    """, "[7, 7, 7, 7]\n"),
    ("irrefutable function and closure parameters", """
        struct Pair(i32, i32);
        struct Marker;
        struct Data { values: [i32; 3] }
        fn total(Pair(mut a, b): Pair, Data { values: [c, .., d] }: Data) -> i32 {
            a += 1;
            a + b + c + d
        }
        fn edit(Pair(ref mut left, _): Pair) -> i32 {
            *left += 2;
            *left
        }
        fn borrow(Pair(left, _): &Pair) -> &i32 { left }
        fn marker(Marker: Marker) -> bool { true }
        fn main() {
            let p = Pair(5, 6);
            let add = |(a, b)| a + b;
            let square = |&x| x * x;
            println!("{} {} {} {} {} {}", total(Pair(1, 2), Data { values: [3, 4, 5] }),
                     edit(Pair(0, 1)), borrow(&p), add((1, 8)), square(&4), marker(Marker));
        }
    """, "12 2 5 9 16 true\n"),
    ("argument position impl Trait", """
        trait Number { fn value(&self) -> i32; }
        struct Wrapped(i32);
        impl Number for Wrapped { fn value(&self) -> i32 { self.0 } }
        impl Number for i32 { fn value(&self) -> i32 { *self } }
        macro_rules! reader {
            ($kind:ty) => { fn first(input: [$kind; 1]) -> i32 { input[0].value() } };
        }
        reader!(impl Number);
        fn total<T: Copy>(prefix: T, a: impl Number, b: &impl Number) -> (T, i32) {
            (prefix, a.value() + b.value())
        }
        trait Combine { fn combine(&self, other: impl Number) -> i32; }
        impl Combine for Wrapped {
            fn combine(&self, other: impl Number) -> i32 { self.0 + other.value() }
        }
        fn main() {
            println!("{:?} {} {}", total::<u8>(8, Wrapped(3), &4), Wrapped(2).combine(9), first([Wrapped(6)]));
        }
    """, "(8, 7) 11 6\n"),
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
    ("generic calls infer, explicitly name, and split type arguments", """
        fn identity<T>(value: T) -> T { value }
        fn second<A, B>(_: A, b: B) -> B { b }
        fn main() {
            println!("{} {} {}", identity(7), identity::<bool>(true), second::<bool, _>(false, 5));
        }
    """, "7 true 5\n"),
    ("expected type flows through generic default calls", """
        fn blank<T: Default>() -> T { T::default() }
        fn nested<T: Default>() -> T { blank() }
        fn identity<T>(x: T) -> T { x }
        fn main() {
            let answer: bool = identity(nested());
            println!("{answer} {}", 3 + blank::<i32>());
        }
    """, "false 3\n"),
    ("generic recursion preserves concrete default type", """
        fn recurse<T: Default>(depth: i32) -> T {
            if depth == 0 { T::default() } else { recurse(depth - 1) }
        }
        fn main() {
            let value: bool = recurse(4);
            println!("{value}");
        }
    """, "false\n"),
    ("generic trait bounds make cloning and debug available", """
        use std::fmt::Debug as Inspect;
        #[derive(Clone, Debug)]
        struct Data(i32);
        fn duplicate<T>(value: &T) -> T where T: Clone + Inspect {
            println!("{value:?}");
            value.clone()
        }
        fn main() { let x = Data(5); println!("{:?}", duplicate(&x)); }
    """, "Data(5)\nData(5)\n"),
    ("Copy bound implies Clone and prevents a move", """
        #[derive(Clone, Copy)]
        struct Data<T>(T);
        fn preserve<T: Copy>(x: T) -> T { let _copy = x; x.clone() }
        fn main() { let original = Data(3); println!("{} {}", preserve(original).0, original.0); }
    """, "3 3\n"),
    ("generic comparison borrows both operands", """
        #[derive(PartialEq)]
        struct Data(i32);
        fn select<T: PartialEq>(first: T, other: T) -> T {
            let _equal = first == other;
            first
        }
        fn main() { println!("{}", select(Data(1), Data(2)).0); }
    """, "1\n"),
    ("scalar comparison snapshots the left value", """
        fn main() {
            let mut n = 2;
            let answer = n == { n = 3; n };
            println!("{answer}");
        }
    """, "false\n"),
    ("derived generic ordering follows declaration order", """
        #[derive(PartialEq, Eq, PartialOrd, Ord)]
        enum Event<T> { Zebra(T), Ant }
        fn before<T: Ord>(a: T, b: T) -> bool { a < b }
        fn main() {
            println!("{} {}", before(Event::Zebra(2), Event::Ant), before(Event::Ant, Event::Zebra(2)));
        }
    """, "true false\n"),
    ("generic records and nested type placeholders", """
        struct Wrap<T> { value: T }
        fn main() {
            let wrapped: Wrap<Wrap<_>>=Wrap { value: Wrap::<i32> { value: 9 } };
            let Wrap { value: Wrap { value: n } } = wrapped;
            println!("{n}");
        }
    """, "9\n"),
    ("generic enum constructor inference uses return contexts", """
        #[derive(Debug)]
        enum Maybe<T> { Absent, Present(T) }
        fn make<T>(value: T, present: bool) -> Maybe<T> {
            if present { Maybe::Present(value) } else { Maybe::Absent }
        }
        fn main() {
            let none = Maybe::<i32>::Absent;
            let some = make(7, true);
            println!("{none:?} {some:?}");
            let Maybe::Present(n) = some else { return; };
            println!("{n}");
        }
    """, "Absent Present(7)\n7\n"),
    ("variant type argument inferred from later branch", """
        #[derive(Debug)]
        enum Maybe<T> { Absent, Present(T) }
        fn main() {
            let absent = Maybe::Absent;
            let result = if false { absent } else { Maybe::Present(2) };
            println!("{result:?}");
        }
    """, "Present(2)\n"),
    ("generic fields borrow and return input lifetimes", """
        struct Wrapper<T>(T);
        fn value<'a, T>(input: &'a mut Wrapper<T>) -> &'a mut T { &mut input.0 }
        fn main() {
            let mut owner = Wrapper(2);
            *value(&mut owner) += 3;
            println!("{}", owner.0);
        }
    """, "5\n"),
    ("generic static references", """
        fn keep<T>(value: &'static T) -> &'static T { value }
        fn main() { let result = keep(&3); println!("{result}"); }
    """, "3\n"),
    ("generic enum Default does not require an unused field to implement Default", """
        struct Item(i32);
        #[derive(Default)]
        enum Maybe<T> { Present(T), #[default] Absent }
        fn main() {
            let data: Maybe<Item> = Default::default();
            match data { Maybe::Present(x) => println!("{}", x.0), Maybe::Absent => println!("absent") }
        }
    """, "absent\n"),
    ("generic record default specializes nested fields", """
        #[derive(Default, Debug)]
        struct Wrapped<T>(T);
        fn blank<T: Default>() -> Wrapped<T> { Wrapped::default() }
        fn main() {
            let result = blank::<Wrapped<bool>>();
            println!("{result:?}");
        }
    """, "Wrapped(Wrapped(false))\n"),
    ("inherent generic impl and two-phase mutable receiver", """
        struct Data<T>(T);
        impl<T: Copy> Data<T> {
            fn new(value: T) -> Self { Data(value) }
            fn get(&self) -> T { self.0 }
            fn set(&mut self, value: T) { self.0 = value; }
        }
        fn main() {
            let mut value = Data::new(4);
            value.set(value.get() + 1);
            println!("{}", Data::get(&value));
        }
    """, "5\n"),
    ("trait defaults and dispatch through generic bounds", """
        trait Score { fn score(&self) -> i32; }
        trait Double: Score {
            fn doubled(&self) -> i32 { self.score() * 2 }
        }
        struct Data(i32);
        impl Score for Data { fn score(&self) -> i32 { self.0 } }
        impl Double for Data {}
        fn score<T: Double>(value: &T) -> i32 {
            value.doubled() + <T as Score>::score(value)
        }
        fn main() { println!("{}", score(&Data(7))); }
    """, "21\n"),
    ("associated functions and default Self dispatch", """
        trait Create: Sized {
            fn new() -> Self;
            fn default_value() -> Self { Self::new() }
        }
        struct Data(i32);
        impl Create for Data { fn new() -> Self { Data(9) } }
        fn main() {
            let first: Data = Create::default_value();
            let second = <Data as Create>::new();
            println!("{} {}", first.0, second.0);
        }
    """, "9 9\n"),
    ("trait associated type and constant through a generic caller", """
        use std::fmt::Debug;
        trait Read {
            type Item: Debug;
            const SIZE: usize = 0;
            fn read(&self) -> Self::Item;
        }
        struct Wrap<T>(T);
        impl<T: Copy + Debug> Read for Wrap<T> {
            type Item = T;
            const SIZE: usize = 1;
            fn read(&self) -> Self::Item { self.0 }
        }
        fn read<T: Read>(value: &T) -> T::Item {
            println!("{}", T::SIZE);
            value.read()
        }
        fn main() {
            let value: <Wrap<i32> as Read>::Item = read(&Wrap(3));
            println!("{value}");
        }
    """, "1\n3\n"),
    ("trait methods specialize their own generics", """
        trait Make { fn make<T: Default>(&self) -> T; }
        struct Data;
        impl Make for Data { fn make<U: Default>(&self) -> U { U::default() } }
        fn main() { let n: i32 = Data.make(); println!("{n} {}", Data.make::<bool>()); }
    """, "0 false\n"),
    ("blanket trait implementation uses bounds", """
        trait Score { fn score(&self) -> i32; }
        impl<T: Copy> Score for T { fn score(&self) -> i32 { 3 } }
        fn show<T: Copy>(value: T) -> i32 { value.score() }
        fn main() { println!("{}", show(true)); }
    """, "3\n"),
    ("trait receiver lifetime feeds borrow checking", """
        trait Read { fn read(&self) -> &i32; }
        struct Data(i32);
        impl Read for Data { fn read(&self) -> &i32 { &self.0 } }
        fn main() {
            let mut value = Data(4);
            let first = value.read();
            println!("{first}");
            value.0 = 8;
            println!("{}", value.read());
        }
    """, "4\n8\n"),
    ("const inference from arrays and contextual result", """
        fn fill<const N: usize>(value: i32) -> [i32; N] { [value; N] }
        fn length<T, const N: usize>(array: &[T; N]) -> usize { array.len() }
        fn main() {
            let mut array: [i32; 3] = fill(7);
            array[1] += 2;
            println!("{array:?} {}", length(&array));
        }
    """, "[7, 9, 7] 3\n"),
    ("const expressions and numeric const parameters", """
        const COUNT: usize = 4;
        fn fill<const N: usize>() -> [usize; N] {
            let mut result = [0; N];
            for i in 0..N { result[i] = i + 1; }
            result
        }
        fn flag<const FLAG: bool>() -> bool { FLAG }
        fn signed<const VALUE: i32>() -> i32 { VALUE }
        fn main() {
            println!("{:?} {} {}", fill::<COUNT>(), flag::<{true && false}>(), signed::<{-4}>());
        }
    """, "[1, 2, 3, 4] false -4\n"),
    ("const args through a generic type and trait impl", """
        trait Size { const LEN: usize; fn size(&self) -> usize { Self::LEN } }
        struct Chunk<T, const N: usize>([T; N]);
        impl<T, const N: usize> Size for Chunk<T, N> { const LEN: usize = N; }
        impl<T, const N: usize> Chunk<T, N> { fn new(data: [T; N]) -> Self { Chunk(data) } }
        fn main() {
            let wrapped = Chunk::new([1, 2, 3]);
            println!("{} {}", wrapped.size(), <Chunk<i32, {1 + 2}> as Size>::LEN);
        }
    """, "3 3\n"),
    ("const generic array trait impl", """
        trait Len { fn count(&self) -> usize; }
        impl<T, const N: usize> Len for [T; N] { fn count(&self) -> usize { N } }
        fn main() { println!("{}", [true, false].count()); }
    """, "2\n"),
    ("macro captures preserve expression precedence and local hygiene", """
        macro_rules! double { ($value:expr) => { $value * 2 }; }
        macro_rules! plus_one {
            ($value:expr) => {{ let n = 1; n + $value }};
        }
        fn main() {
            let n = 5;
            println!("{} {} {n}", double!(1 + 2), plus_one!(double!(n)));
        }
    """, "6 11 5\n"),
    ("macro repetitions with separators and optional trailing comma", """
        macro_rules! array {
            ($($item:expr),+ $(,)?) => { [$($item),+] };
        }
        macro_rules! sum {
            ($first:expr $(, $item:expr)*) => { $first $(+ $item)* };
        }
        fn main() {
            println!("{:?} {} {}", array!(1, 2, 3,), sum!(1, 2, 3), sum!(4));
        }
    """, "[1, 2, 3] 6 4\n"),
    ("macro emits an item and trait implementation", """
        trait Score { fn score(&self) -> i32; }
        macro_rules! make {
            ($name:ident, $value:literal) => {
                struct $name;
                impl Score for $name { fn score(&self) -> i32 { $value } }
            };
        }
        make!(Data, -7);
        fn main() { println!("{}", Data.score()); }
    """, "-7\n"),
    ("recursive token-tree macro and alternate rules", """
        macro_rules! count {
            () => { 0 };
            ($first:tt $($rest:tt)*) => { 1 + count!($($rest)*) };
        }
        fn main() { println!("{} {}", count!(a (b c) [d]), count!()); }
    """, "3 0\n"),
    ("borrowed trait objects dispatch to multiple implementations", """
        trait Measure {
            fn read(&self) -> i32;
            fn set(&mut self, value: i32);
            fn create() -> Self where Self: Sized;
        }
        struct First(i32);
        struct Second(i32);
        impl Measure for First {
            fn read(&self) -> i32 { self.0 }
            fn set(&mut self, value: i32) { self.0 = value; }
            fn create() -> Self { First(3) }
        }
        impl Measure for Second {
            fn read(&self) -> i32 { self.0 * 2 }
            fn set(&mut self, value: i32) { self.0 = value; }
            fn create() -> Self { Second(4) }
        }
        fn read(value: &dyn Measure) -> i32 { value.read() }
        fn main() {
            let first = First::create();
            let mut second = Second::create();
            let target: &mut dyn Measure = &mut second;
            target.set(read(&first as &dyn Measure));
            println!("{} {}", read(&first), target.read());
        }
    """, "3 6\n"),
    ("trait objects bind associated types and upcast to their supertrait", """
        trait Read { type Item; fn read(&self) -> Self::Item; }
        trait Rich: Read { fn bonus(&self) -> i32 { 1 } }
        struct Cell(i32);
        impl Read for Cell { type Item = i32; fn read(&self) -> i32 { self.0 } }
        impl Rich for Cell {}
        fn plain(item: &dyn Read<Item = i32>) -> i32 { item.read() }
        fn main() {
            let cell = Cell(9);
            let rich: &dyn Rich<Item = i32> = &cell;
            println!("{} {}", plain(rich), rich.bonus());
        }
    """, "9 1\n"),
    ("trait object chosen by control flow keeps its concrete dispatch tag", """
        trait Read { fn read(&self) -> i32; }
        struct First;
        struct Second;
        impl Read for First { fn read(&self) -> i32 { 1 } }
        impl Read for Second { fn read(&self) -> i32 { 2 } }
        fn choose(flag: bool) -> i32 {
            let first = First;
            let second = Second;
            let result: &dyn Read = if flag { &first } else { &second };
            result.read()
        }
        fn main() { println!("{} {}", choose(true), choose(false)); }
    """, "1 2\n"),
    ("generic associated type and const projections", """
        trait Family { type Member<T: Copy>; type Block<const N: usize>; }
        struct Owner;
        impl Family for Owner {
            type Member<T: Copy> = [T; 2];
            type Block<const N: usize> = [i32; N];
        }
        fn main() {
            let first: <Owner as Family>::Member<bool> = [true, false];
            let second: <Owner as Family>::Block<3> = [1, 2, 3];
            println!("{first:?} {second:?}");
        }
    """, "[true, false] [1, 2, 3]\n"),
    ("lending projection borrows a concrete receiver", """
        trait Lending {
            type Item<'a> where Self: 'a;
            fn get<'a>(&'a self) -> Self::Item<'a>;
        }
        struct Cell(i32);
        impl Lending for Cell {
            type Item<'a> = &'a i32;
            fn get<'a>(&'a self) -> Self::Item<'a> { &self.0 }
        }
        fn main() {
            let mut cell = Cell(7);
            let item = cell.get();
            println!("{item}");
            cell.0 = 9;
            println!("{}", cell.get());
        }
    """, "7\n9\n"),
    ("dependent const expressions specialize and compose", """
        #![feature(generic_const_exprs)]
        #![allow(incomplete_features)]
        fn repeat<const N: usize>() -> [i32; N] { [7; N] }
        fn padded<const N: usize>() -> [i32; N + 1] where [(); N + 1]: {
            repeat::<{ N + 1 }>()
        }
        fn main() {
            let result: [i32; 4] = padded::<3>();
            println!("{result:?}");
        }
    """, "[7, 7, 7, 7]\n"),
    ("macro definition-site locals and invocation tokens stay distinct", """
        struct Pair { x: i32 }
        macro_rules! shadow {
            ($input:expr) => {{ let x = 7; let pair = Pair { x }; pair.x + $input }};
        }
        fn main() {
            let x = 2;
            macro_rules! definition { () => { x }; }
            let x = 5;
            println!("{} {} {x}", shadow!(x), definition!());
        }
    """, "12 2 5\n"),
    ("macro labels resolve to definition-site target", """
        fn main() {
            let result = 'outer: loop {
                macro_rules! leave { () => { break 'outer 7 }; }
                'outer: loop { leave!(); }
            };
            println!("{result}");
        }
    """, "7\n"),
    ("forwarded macro expressions stay opaque to literal matchers", """
        macro_rules! inner { (1) => { 9 }; ($expr:expr) => { $expr }; }
        macro_rules! outer { ($expr:expr) => { inner!($expr) }; }
        fn main() { println!("{} {}", inner!(1), outer!(1)); }
    """, "9 1\n"),
    ("macro crate-qualified paths use crate-root items", """
        fn answer() -> i32 { 42 }
        macro_rules! root { () => { $crate::answer() }; }
        fn main() { let answer = 3; println!("{} {answer}", root!()); }
    """, "42 3\n"),
    ("nested modules resolve self, crate, and repeated super", """
        fn base() -> i32 { 4 }
        mod first {
            fn value() -> i32 { 6 }
            pub mod second {
                pub mod third {
                    pub fn run() -> i32 { super::super::value() + crate::base() }
                }
            }
            pub fn run() -> i32 { self::second::third::run() }
        }
        fn main() { println!("{}", first::run()); }
    """, "10\n"),
    ("grouped aliases and reexports preserve target namespace", """
        mod api {
            mod internal {
                #[derive(Debug)]
                pub struct Data(pub i32);
                pub fn make() -> Data { Data(5) }
            }
            pub use self::internal::{Data, make as create};
        }
        use crate::api::{self as alias, Data as Entry};
        fn main() {
            let item: Entry = alias::create();
            let Entry(value) = item;
            println!("{value}");
        }
    """, "5\n"),
    ("explicit imports take precedence over globs", """
        mod a { pub fn value() -> i32 { 1 } }
        mod b { pub fn value() -> i32 { 2 } }
        use a::*;
        use b::value;
        fn main() { println!("{}", value()); }
    """, "2\n"),
    ("glob follows a later reexport", """
        mod source { pub fn value() -> i32 { 7 } }
        mod reexport { pub use crate::source::value; }
        use reexport::*;
        fn main() { println!("{}", value()); }
    """, "7\n"),
    ("namespace with scoped trait method import", """
        mod api {
            pub trait Read { fn value(&self) -> i32; }
            pub struct Data(i32);
            impl Data { pub fn new(x: i32) -> Self { Self(x) } }
            impl Read for Data { fn value(&self) -> i32 { self.0 } }
        }
        use crate::api::Read as _;
        fn main() { let data = api::Data::new(8); println!("{}", data.value()); }
    """, "8\n"),
    ("sibling const references and namespaced turbofish", """
        mod operations {
            pub fn count<const N: usize>() -> usize { N }
        }
        mod sizes {
            pub const LARGE: usize = SMALL + 3;
            const SMALL: usize = 2;
        }
        fn main() { println!("{}", operations::count::<{ sizes::LARGE }>()); }
    """, "5\n"),
    ("inherited and re-exported macros follow textual and path scope", """
        macro_rules! add_one { ($x:expr) => { $x + 1 }; }
        mod declarations {
            pub fn local() -> i32 { add_one!(3) }
            #[macro_export]
            macro_rules! root { () => { $crate::number() }; }
        }
        mod exports { pub use crate::root as alias; }
        use crate::exports::alias;
        fn number() -> i32 { 7 }
        fn main() { println!("{} {} {}", declarations::local(), exports::alias!(), alias!()); }
    """, "4 7 7\n"),
    ("restricted visibility allows ancestor access", """
        mod api {
            pub mod nested {
                pub(in crate::api) fn internal() -> i32 { 5 }
            }
            pub fn external() -> i32 { nested::internal() }
        }
        fn main() { println!("{}", api::external()); }
    """, "5\n"),
    ("linked Option and Result propagate through question mark", """
        mod input {
            pub fn read(ok: bool) -> Result<i32, &'static str> {
                if ok { Ok(7) } else { Err("bad") }
            }
        }
        fn double(ok: bool) -> Result<i32, &'static str> {
            let number = input::read(ok)?;
            Ok(number * 2)
        }
        fn option(ok: bool) -> Option<i32> {
            let value = if ok { Some(5) } else { None };
            Some(value? + 1)
        }
        fn main() {
            println!("{:?} {:?} {:?} {}", double(false), double(true), option(false), option(true).unwrap_or(0));
        }
    """, 'Err("bad") Ok(14) None 6\n'),
    ("prelude macros share the declarative expander", """
        fn main() {
            let value = Some(3);
            assert!(matches!(value, Some(x) if x == 3));
            assert_eq!(value, Some(3));
            assert_ne!(value, None);
            println!("{}", value.expect("valid"));
        }
    """, "3\n"),
    ("prelude type names can be shadowed by user items", """
        struct Option(i32);
        fn main() {
            let own = Option(3);
            let standard: std::option::Option<i32> = Some(5);
            println!("{} {}", own.0, standard.unwrap());
        }
    """, "3 5\n"),
    ("function pointers and noncapturing closures take parameters", """
        fn inc(x: i32) -> i32 { x + 1 }
        fn apply(f: fn(i32) -> i32, x: i32) -> i32 { f(x) }
        mod ops { pub fn double(n: i32) -> i32 { n * 2 } }
        fn main() {
            let plain: fn(i32) -> i32 = inc;
            let closure = |n: i32| n + 3;
            let inferred = |n| n;
            println!("{} {} {} {} {}", apply(plain, 1), (ops::double)(3), closure(4), (|| 9)(), inferred(true));
        }
    """, "2 6 7 9 true\n"),
    ("function pointer retains generic type specialization", """
        fn value<T: Default>() -> T { T::default() }
        fn main() { let call: fn() -> bool = value; println!("{}", call()); }
    """, "false\n"),
    ("indirect reference return retains borrow until last use", """
        fn id(n: &i32) -> &i32 { n }
        fn main() {
            let pointer: fn(&i32) -> &i32 = id;
            let mut n = 3;
            let result = pointer(&n);
            println!("{result}");
            n = 7;
            println!("{n}");
        }
    """, "3\n7\n"),
    ("shared and move captures follow their own lifetimes", """
        fn main() {
            let mut base = 3;
            let add = |n: i32| base + n;
            let copy = add;
            println!("{} {}", add(2), copy(3));
            base = 8;
            let original = move || base;
            base = 9;
            println!("{} {base}", original());
        }
    """, "5 6\n8 9\n"),
    ("shared capture can read a non-Copy field repeatedly", """
        struct Value(i32);
        fn main() {
            let value = Value(3);
            let call = || value.0;
            println!("{} {}", call(), call());
        }
    """, "3 3\n"),
    ("mutable captures permit repeated calls and release at last use", """
        fn main() {
            let mut count = 1;
            let mut next = || { count += 1; count };
            println!("{} {}", next(), next());
            println!("{count}");
        }
    """, "2 3\n3\n"),
    ("move closure can consume an owned non-Copy capture", """
        struct Item(i32);
        fn consume(item: Item) -> i32 { item.0 }
        fn main() {
            let saved = Item(6);
            let once = move || consume(saved);
            println!("{}", once());
        }
    """, "6\n"),
    ("closures keep shared references alive across copies and moves", """
        fn main() {
            let mut n = 4;
            let r = &n;
            let f = || *r;
            let g = f;
            let move_reference = move || *r;
            println!("{} {} {}", f(), g(), move_reference());
            n = 6;
            println!("{n}");
        }
    """, "4 4 4\n6\n"),
    ("mutable reference captures can reborrow and take ownership", """
        fn bump(n: &mut i32) { *n += 1; }
        fn main() {
            let mut n = 1;
            let r = &mut n;
            let mut by_borrow = || { bump(r); *r };
            println!("{} {}", by_borrow(), by_borrow());
            let mut by_move = move || { *r += 1; *r };
            println!("{} {}", by_move(), by_move());
            println!("{n}");
        }
    """, "2 3\n4 5\n5\n"),
    ("tuple types support generics, methods, patterns, and nested fields", """
        #[derive(Debug)]
        struct Packet<T>((T,));
        trait Get { fn value(&self) -> i32; }
        impl<T> Get for (i32, T) { fn value(&self) -> i32 { self.0 } }
        fn identity<T>(value: (T,)) -> (T,) { value }
        fn main() {
            let mut pair: (usize, (_, bool)) = (1, (3, false));
            let old = pair;
            pair.1.0 += 1;
            let (first, (second, ..)) = pair;
            let _ = match pair {
                (x, (_, true)) => x,
                (x, (_, false)) => x,
            };
            let (..) = ();
            let p = Packet(identity((5,)));
            println!("{first} {second} {old:?} {p:?}");
            let inner: (i32,) = Default::default();
            println!("{inner:?} {}", (7, true).value());
        }
    """, "1 4 (1, (3, false)) Packet((5,))\n(0,) 7\n"),
    ("tuple destructuring and reassignment track partial moves", """
        #[derive(Debug)]
        struct Item(i32);
        fn main() {
            let mut pair = (Item(1), Item(2));
            let (first, ..) = pair;
            println!("{first:?} {:?}", pair.1);
            pair.0 = Item(3);
            println!("{pair:?}");
        }
    """, "Item(1) Item(2)\n(Item(3), Item(2))\n"),
    ("array patterns match prefix, suffix, and sparse positions", """
        fn main() {
            let [x, .., y] = [6; 2048];
            let data = [true, false, true];
            let n = match data {
                [false, ..] => 1,
                [.., false] => 2,
                [true, _, true] => 3,
            };
            assert!(matches!([1, 2], [..=1, (2..)]));
            let empty: [i32; 0] = [];
            let [] = empty;
            println!("{x} {y} {n}");
        }
    """, "6 6 3\n"),
    ("array patterns move disjoint elements", """
        #[derive(Debug)]
        struct Item(i32);
        fn main() {
            let values = [Item(1), Item(2)];
            let [first, ..] = values;
            let [.., last] = values;
            println!("{first:?} {last:?}");
        }
    """, "Item(1) Item(2)\n"),
    ("arrays coerce to shared, mutable, and static slices", """
        fn view<const N: usize>(data: &[i32; N]) -> &[i32] { data }
        fn replace<T: Copy>(values: &mut [T], item: T) -> T {
            let old = values[0];
            values[0] = item;
            old
        }
        fn second(values: &[i32]) -> i32 { values[1] }
        fn permanent() -> &'static [i32] { &[3, 4] }
        fn main() {
            let mut values = [3, 5, 6];
            println!("{} {}", replace(&mut values, 7), second(&values));
            let borrowed = view(&values);
            println!("{borrowed:?} {}", borrowed.len());
            println!("{:?}", permanent());
            println!("{values:?}");
        }
    """, "3 5\n[7, 5, 6] 3\n[3, 4]\n[7, 5, 6]\n"),
    ("subslice ranges share storage and keep their own bounds", """
        fn inside(input: &mut [i32], start: usize, stop: usize) -> &mut [i32] {
            &mut input[start..stop]
        }
        fn main() {
            let mut a = [3, 4, 5, 6, 7];
            let view = inside(&mut a, 1, 4);
            let middle = &mut view[1..=2];
            middle[0] += 10;
            middle[1] += 20;
            println!("{middle:?} {}", middle.len());
            let tail = &view[2..];
            println!("{tail:?} {}", tail.is_empty());
            println!("{a:?} {:?}", &a[..=1]);
            let empty: &[bool] = &[];
            println!("{}", empty.is_empty());
        }
    """, "[15, 26] 2\n[26] false\n[3, 4, 15, 26, 7] [3, 4]\ntrue\n"),
    ("slice exhaustiveness accounts for length and overlapping ends", """
        fn ends(s: &[bool]) -> i32 {
            match *s {
                [] => 0,
                [true] => 1,
                [false] => 2,
                [true, .., true] => 3,
                [false, ..] => 4,
                [.., false] => 5,
            }
        }
        fn outer(s: &[i32]) -> i32 {
            let [first, .., last] = *s else { return 0; };
            first + last
        }
        fn main() {
            println!("{} {} {} {}", ends(&[]), ends(&[false]),
                     ends(&[true, false]), ends(&[true, false, true]));
            let a = [1, 2, 3, 4];
            if let [start, ..] = a[1..] { println!("{start} {}", outer(&a)); }
            assert!(matches!(a[..], [1, ..]));
        }
    """, "0 2 5 3\n2 5\n"),
    ("integer widths, casts, bases, and const parameters", """
        const SMALL: u8 = 100 + 100;
        fn value<const N: u16>() -> u16 { N }
        fn classify(n: u8) -> i32 {
            match n { 0..=129 => 0, 130..=u8::MAX => 1 }
        }
        fn main() {
            let max = u128::MAX;
            let min: i8 = -128;
            let min128 = -170141183460469231731687303715884105728i128;
            let result: i64 = -4 + 0x10;
            let sum = 3000000000 + 1u64;
            let cast = 300i32 as u8;
            let widened = 3000000000 as u64;
            let bits = !0b_0000_0011u8;
            let mut n = -1isize;
            for x in 0..3u16 { n -= x as isize; }
            println!("{max} {min} {min128}");
            println!("{result} {sum} {cast} {widened} {bits} {n}");
            println!("{} {} {} {}", SMALL, value::<0o77u16>(), classify(bits), i64::BITS);
        }
    """, "340282366920938463463374607431768211455 -128 -170141183460469231731687303715884105728\n"
         "12 3000000001 44 3000000000 252 -4\n200 63 1 64\n"),
    ("characters are Unicode scalars in patterns and const generics", r"""
        const UPPER: char = 'Ω';
        const ORDER: bool = 'A' < UPPER;
        fn fixed<const C: char>() -> char { C }
        fn group(c: char) -> u8 {
            match c {
                char::MIN..='\u{D7FF}' => 1,
                '\u{E000}'..=char::MAX => 2,
            }
        }
        fn identity<'a>(x: &'a u8) -> &'a u8 { x }
        fn main() {
            let crab = loop { break '🦀'; };
            let backslash = '\\';
            let quote = '\'';
            let visible = 65 as char;
            let escaped = b'\xFF';
            let accent = fixed::<'é'>();
            println!("{visible} {} {crab:?} {accent}", *identity(&escaped));
            println!("{backslash:?} {quote:?} {} {} {}", ORDER, group(accent), group(crab));
            println!("{} {}", UPPER as u32, '\u{1F_680}');
        }
    """, "A 255 '🦀' é\n'\\\\' '\\'' true 1 2\n937 🚀\n"),
    ("raw strings and static byte strings", r"""
        fn literal() -> &'static [u8; 3] { b"AB\xFF" }
        fn payload(bytes: &[u8]) -> usize { bytes.len() }
        fn main() {
            let raw = r##"A "# and \n"##;
            let bytes = br#"A \n"#;
            println!(r#"{raw} {bytes:?}"#);
            println!("{:?} {:?} {}", literal(), b"", payload(b"abc"));
        }
    """, 'A "# and \\n [65, 32, 92, 110]\n[65, 66, 255] [] 3\n'),
    ("bitwise precedence, signed shifts, and compound assignment", """
        const HIGH: u128 = 1u128 << 127u8;
        const MASK: u8 = !0b11;
        const TRUNCATED: u8 = 300u16 as u8;
        const FLAG: bool = true ^ false;
        fn selection(x: i16, amount: u8) -> i16 { (x >> amount) & 127i16 }
        fn sample<const N: u8>() -> u8 { N }
        macro_rules! right { ($n:expr, $p:expr) => { $n >> $p }; }
        fn main() {
            let mut x = 0b1111u8;
            x <<= 6u16;
            x ^= 48;
            x &= 0b0111_1111;
            x >>= 1;
            x |= 3;
            let mut flag = true;
            flag &= false;
            flag ^= true;
            let n: u64 = (1 << 40) + 9;
            let l = 1 + 2 << 2 & 15 ^ 8 | 1;
            println!("{x} {flag} {n} {l}");
            println!("{} {} {} {}", selection(-30, 2), HIGH, sample::<{ 1u8 << 7 }>(), right!(29u16, 2));
            println!("{} {} {} {}", true | false & false, MASK, TRUNCATED, FLAG);
        }
    """, "59 true 1099511627785 5\n120 170141183460469231731687303715884105728 128 7\ntrue 252 44 true\n"),
    ("symbolic shifts and casts in const generic array lengths", """
        #![feature(generic_const_exprs)]
        #![allow(incomplete_features)]
        fn packet<const N: u8>() -> [u8; (N as usize) >> 1]
        where [(); (N as usize) >> 1]:
        {
            [0u8; (N as usize) >> 1]
        }
        fn main() { println!("{:?}", packet::<6>()); }
    """, "[0, 0, 0]\n"),
    ("forward and imported type aliases preserve generics and lifetimes", """
        type Output<T> = shapes::Pair<T>;
        type Borrowed<'a, T> = &'a T;
        type Static<T> = Borrowed<'static, T>;
        type Callback<T> = fn(T) -> T;
        type Bytes = [u8];
        mod shapes {
            pub type Pair<T> = (T, [u8; SIZE]);
            pub const SIZE: Width = 3;
            pub type Width = usize;
        }
        macro_rules! describe { () => { type Numeric = i64; }; }
        describe!();
        use shapes::Width as Count;
        fn transform(value: u8) -> u8 { value + 1 }
        fn pass<'a>(value: Borrowed<'a, u8>) -> Borrowed<'a, u8> { value }
        fn promoted() -> Static<u8> { &8 }
        fn pass_elided(value: Borrowed<u8>) -> Borrowed<u8> { value }
        fn total<const N: Count>() -> Count { N }
        fn length(data: &Bytes) -> usize { data.len() }
        fn main() {
            let result: Output<_> = (true, [1, 2, 3]);
            let call: Callback<u8> = transform;
            let integer: Numeric = 10;
            println!("{result:?} {} {} {} {}", call(*pass(&3)), promoted(), pass_elided(&7), length(&result.1));
            println!("{} {integer}", total::<3>());
        }
    """, "(true, [1, 2, 3]) 4 8 7 3\n3 10\n"),
    ("aliases work inside aggregate fields and trait impl targets", """
        type Array<const N: usize> = [u8; N];
        type Payload<T> = (T,);
        trait Score { fn score(&self) -> u8; }
        impl Score for Array<3> { fn score(&self) -> u8 { self[1] } }
        struct Packet<T> { value: Payload<T> }
        fn main() {
            let packet: Packet<Array<3>> = Packet { value: ([8, 9, 10],) };
            println!("{}", packet.value.0.score());
        }
    """, "9\n"),
    ("patterns borrow through shared references and carry their lifetime", """
        struct Item(i32);
        struct Holder { field: (Item, i32) }
        fn select(holder: &Holder) -> &Item {
            match holder { Holder { field: (one, _) } => one }
        }
        fn item(value: &Option<Item>) -> i32 {
            match value { Some(x) => x.0, None => -1 }
        }
        fn main() {
            let mut values = [1, 2, 3];
            let view: &[i32] = &values[..];
            let first = match view { [n, ..] => *n, _ => 0 };
            let [left, .., right] = view else { return; };
            println!("{first} {left} {right}");
            values[0] = 9;
            println!("{values:?}");
            let record = Holder { field: (Item(5), 7) };
            println!("{}", select(&record).0);
            let maybe = Some(Item(8));
            if let Some(value) = &maybe { println!("{}", value.0); }
            println!("{}", item(&maybe));
            while let Some(value) = &maybe { println!("{}", value.0); break; }
        }
    """, "1 1 3\n[9, 2, 3]\n5\n8\n8\n8\n"),
    ("shared reference pattern alternatives retain projected storage", """
        enum Choice { Left(i32), Right(i32) }
        fn choice() -> Choice { Choice::Right(7) }
        fn main() {
            let (Choice::Left(n) | Choice::Right(n)) = &choice();
            println!("{n}");
            let pair = (1, 2);
            let (mut first, second) = &pair;
            first += 1;
            println!("{first} {second}");
        }
    """, "7\n2 2\n"),
    ("mutable patterns borrow disjoint fields and slice ends", """
        struct Item(i32);
        enum Two { Pair(Item, Item), Empty }
        fn change(values: &mut [u8]) {
            let [first, .., last] = values else { return; };
            *first += 3;
            *last += 5;
        }
        fn first(values: &mut [u8; 3]) -> &mut u8 {
            let [first, ..] = values;
            first
        }
        fn main() {
            let mut data = [1u8, 2, 3];
            change(&mut data[..]);
            *first(&mut data) += 1;
            let [left, right, _] = &mut data;
            *right = *left;
            println!("{data:?}");
            let mut value = Two::Pair(Item(1), Item(2));
            if let Two::Pair(left, right) = &mut value {
                left.0 += 3;
                right.0 += left.0;
                println!("{} {}", left.0, right.0);
            }
            let mut empty = Two::Empty;
            match &mut empty { Two::Empty => println!("none"), _ => () }
        }
    """, "[5, 5, 8]\n4 6\nnone\n"),
    ("explicit reference patterns borrow fields and extend let temporaries", """
        struct Point { left: i32, right: i32 }
        enum Duo { Pair(i32, i32) }
        fn make() -> Point { Point { left: 2, right: 3 } }
        fn promoted() -> &'static i32 { let ref r = 5; r }
        fn main() {
            let mut point = make();
            let Point { ref mut left, right: ref other } = point;
            *left += *other;
            let &mut copied = left;
            println!("{copied} {other}");
            let Point { ref left, .. } = make();
            let ref whole = make();
            let mut pair = Duo::Pair(7, 8);
            let Duo::Pair(ref mut a, ref mut b) = pair;
            *a += *b;
            *b -= 1;
            let &temporary = promoted();
            println!("{left} {} {a} {b} {temporary}", whole.right);
        }
    """, "5 3\n2 3 15 7 5\n"),
    ("at bindings preserve the whole pattern value and its projections", """
        struct Item(i32);
        enum Choice { First(i32), Second(i32) }
        fn main() {
            let value = match 2 { n @ 1..=3 => n, _ => 0 };
            let aggregate @ (header, _) = (value, Item(4));
            let ref pair @ (ref left, ref right) = (3, 7);
            let container = Some(Item(9));
            let score = match &container {
                whole @ Some(item) if whole.is_some() => item.0,
                _ => 0,
            };
            let choice = Choice::Second(8);
            let v = match choice { _c @ (Choice::First(n) | Choice::Second(n)) => n };
            println!("{} {} {} {} {} {} {}", aggregate.1.0, header, pair.1, left, right, score, v);
        }
    """, "4 2 7 3 7 9 8\n"),
    ("vectors use two-phase borrowing, runtime lengths, and borrowed slices", """
        #[derive(Clone, Debug, PartialEq)]
        struct Row(i32);
        struct Tree { children: Vec<Tree> }
        fn view(values: &Vec<u8>) -> &[u8] { values.as_slice() }
        fn sum(values: &[u8]) -> u8 { values[0] + values[1] }
        fn main() {
            let mut a = Vec::new();
            a.push(1usize);
            a.push(a.len() + 1);
            println!("{a:?} {:?}", a.pop());
            let mut rows = vec![Row(4); a.len() + 1];
            rows[0].0 = 3;
            println!("{rows:?}");
            let mut values: std::vec::Vec<u8> = vec![7, 8, 9,];
            let old = values.clone();
            values.as_mut_slice()[1] = 4;
            println!("{} {} {old:?}", sum(view(&values)), sum(&values));
            values.truncate(2);
            values.clear();
            println!("{} {:?}", values.is_empty(), values.pop());
            let node = Tree { children: vec![] };
            println!("{}", node.children.len());
        }
    """, "[1] Some(2)\n[Row(3), Row(4)]\n11 11 [7, 8, 9]\ntrue None\n0\n"),
    ("local macro definitions can shadow the prelude vector macro", """
        macro_rules! vec { () => { 7i32 }; }
        fn main() { println!("{}", vec!()); }
    """, "7\n"),
    ("for loops destructure owned sequences and borrow collection elements", """
        struct Item(i32);
        fn first(values: &mut [i32]) -> &mut i32 {
            for value in values { return value; }
            panic!("empty");
        }
        fn main() {
            let mut v = vec![3, 4, 5];
            for value in &mut v { *value += 1; }
            *first(&mut v) += 1;
            let mut sum = 0;
            'outer: for &value in &v {
                for (_, extra) in [(0, 1), (0, 2)] {
                    if value == 6 { break 'outer; }
                    sum += value + extra;
                }
            }
            let mut tag = 0;
            for &byte in b"AB" { tag += byte as i32; }
            for _ in 0..2 { tag += 1; }
            let mut tally = 0;
            let items = vec![Item(2), Item(4)];
            for Item(value) in items { tally += value; }
            println!("{v:?} {sum} {tag} {tally}");
        }
    """, "[5, 5, 6] 26 133 6\n"),
    ("for loops visit successful Option and Result values once", """
        fn main() {
            let mut option = Some(2);
            for value in &mut option { *value *= 2; }
            let success: Result<i32, bool> = Ok(3);
            let error: Result<i32, bool> = Err(false);
            let mut sum = 0;
            for v in option { sum += v; }
            for &v in &success { sum += v; }
            for v in error { sum += v; }
            println!("{sum}");
        }
    """, "7\n"),
    ("collection iterators can be stored, cloned, moved, and reversed", """
        fn main() {
            let mut data = vec![1, 2, 3];
            let reverse = data.iter().rev();
            let another = reverse.clone();
            for &n in reverse { println!("{n}"); }
            for n in another.rev() { println!("{n}"); }
            data.push(4);
            let iter = data.iter_mut().rev();
            for n in iter { *n *= 2; }
            for value in data.into_iter().rev() { println!("{value}"); }
            for x in [0, 1].into_iter() { println!("{x}"); }
            let mut selected = Some(7);
            for value in selected.iter_mut() { *value = 8; }
            for n in selected.iter() { println!("{n}"); }
        }
    """, "3\n2\n1\n1\n2\n3\n8\n6\n4\n2\n0\n1\n8\n"),
    ("iterator map is lazy and reverses the order of mapper calls", """
        struct Item(i32);
        fn show(n: i32) -> Item { println!("map {n}"); Item(n + 4) }
        fn main() {
            let v = vec![1, 2, 3];
            let values = v.iter().map(|n| *n * 2).map(show).rev();
            println!("start");
            for Item(n) in values { println!("{n}"); break; }
            let scores = [3, 5].into_iter().map(show);
            for Item(n) in scores.clone() { println!("{n}"); }
            for Item(n) in scores { println!("{n}"); }
        }
    """, "start\nmap 6\n10\nmap 3\n7\nmap 5\n9\nmap 3\n7\nmap 5\n9\n"),
    ("map accepts mutable closures and clones owned capture state", """
        fn main() {
            let mut count = 0;
            for value in vec![4, 5].into_iter().map(|n| { count += 1; count * n }) {
                println!("{value}");
            }
            println!("{count}");
            let mut count = 0;
            let mut mapper = move |n: i32| { count += 1; n + count };
            let mut copied = mapper;
            println!("{} {}", mapper(0), copied(0));
            let mapped = vec![1, 3].into_iter().map(mapper);
            let cloned = mapped.clone();
            for value in mapped { println!("{value}"); }
            for value in cloned { println!("{value}"); }
            println!("{}", mapper(0));
        }
    """, "4\n10\n2\n1 1\n3\n6\n3\n6\n2\n"),
    ("owned strings preserve UTF-8 lengths and character boundaries", """
        use std::string::String as Text;
        fn add_mark(text: &mut Text) { text.push('!'); }
        fn main() {
            let mut text = Text::from("Rust");
            text.push('🦀');
            add_mark(&mut text);
            println!("{:?} {} {}", text, text.len(), "🦀".len());
            assert_eq!(text, "Rust🦀!");
            println!("{:?} {:?}", text.pop(), text.pop());
            text += " in ";
            let mut result = text + "Python";
            result.truncate(7);
            println!("{result}");
            let mut empty = String::new();
            empty.push_str("lost");
            empty.clear();
            println!("{:?} {} {:?} {:?}", empty, empty.is_empty(), true.to_string(), 'Ω'.to_string());
            let x: String = "five".to_owned();
            let y: String = Default::default();
            println!("{x:?} {y:?}");
        }
    """, '"Rust🦀!" 9 4\nSome(\'!\') Some(\'🦀\')\nRust in\n"" true "true" "Ω"\n"five" ""\n'),
    ("string views preserve explicit and elided lifetimes", """
        fn view<'a>(text: &'a String) -> &'a str { text.as_str() }
        fn echo(text: &str) -> &str { text }
        fn static_text() -> &'static str { "static" }
        fn main() {
            let mut s = String::from("Rust🦀");
            let a: &str = echo(&s);
            let b = view(&s);
            println!("{a} {b:?} {}", a.len());
            s.push('!');
            let mut t = String::from("abcΩ");
            t.as_mut_str().make_ascii_uppercase();
            let appended = s + &t;
            match appended.as_str() { "Rust🦀!ABCΩ" => println!("matched"), _ => panic!("unexpected") }
            println!("{appended} {}", static_text());
        }
    """, 'Rust🦀 "Rust🦀" 8\nmatched\nRust🦀!ABCΩ static\n'),
    ("UTF-8 substrings alias the original data", """
        fn middle<'a>(text: &'a str) -> &'a str { &text[1..3] }
        fn main() {
            let mut text = String::from("aΩbc");
            let s = middle(&text);
            println!("{s} {}", s.len());
            text[3..].make_ascii_uppercase();
            let alt = &text[1..];
            println!("{} {} {}", text, &alt[..2], &alt[2..=3]);
            let fixed: &'static str = &"XY"[..1];
            println!("{fixed}");
            let frozen = ["a", "b"];
            let item = frozen[1];
            println!("{item}");
            let mut text: &'static str = "before";
            println!("{text}");
            text = "after";
            println!("{text}");
        }
    """, "Ω 2\naΩBC Ω BC\nX\nb\nbefore\nafter\n"),
    ("string iterators distinguish Unicode scalars and UTF-8 bytes", """
        fn main() {
            let mut s = String::from("aΩ");
            let letters = s.chars().rev();
            for c in letters { println!("{c}"); }
            s.push('!');
            for b in s[1..3].bytes() { println!("{b}"); }
        }
    """, "Ω\na\n206\n169\n"),
    ("iterator consumers collect strings and vectors and drive mapper effects", """
        fn main() {
            let mut s = String::from("aΩ");
            let ascii: String = s.chars().rev().collect();
            s.push('!');
            let bytes = s.bytes().map(|x| x as u16).collect::<Vec<_>>();
            let sum: u16 = bytes.iter().sum();
            let count = bytes.iter().map(|n| { println!("{n}"); *n }).count();
            let parts = ["one", " two"].into_iter().collect::<String>();
            println!("{ascii} {bytes:?} {sum} {count} {parts} {s}");
            let words = ["first", "second"].into_iter().rev().collect::<Vec<_>>();
            println!("{words:?}");
        }
    """, '97\n206\n169\n33\nΩa [97, 206, 169, 33] 505 4 one two aΩ!\n["second", "first"]\n'),
    ("floating-point arithmetic, NaN, constants, and saturating casts", """
        const C: f32 = 0.1 + 0.2;
        const COUNT: u8 = 1000.5 as u8;
        #[derive(PartialEq, PartialOrd, Debug)]
        struct Score { n: f64 }
        fn main() {
            let mut v: f32 = 0.1;
            v += 0.2;
            let items = [0.1, 0.2f32];
            let t = (0, (v, 1.0));
            println!("{v} {C:?} {items:?} {} {:?}", t.1.0, -0.0f64);
            let nan = 0.0f64 / 0.0;
            let a = Score { n: nan };
            let b = Score { n: 1.0 };
            println!("{} {} {} {} {:?} {} {COUNT}", a == b, a < b, a != b, nan > 0.0, nan, nan as i64);
            println!("{} {} {:?} {} {}", 1.0f32 / -0.0, (-5.5f64) % 2.0, 1e-7f32, 1e-7f32, 3.0f64 as i32);
            println!("{:?} {:?} {:?}", 1e8f32, 1e16f32, 1e-4f32);
        }
    """, "0.3 0.3 [0.1, 0.2] 0.3 -0.0\nfalse false true false NaN 0 255\n"
         "-inf -1.5 1e-7 0.0000001 3\n100000000.0 1e16 0.0001\n"),
    ("float methods and range patterns use binary32 and binary64 values", """
        fn classify(x: f32) -> i32 {
            match x { 0.0..=1.5 => 1, -5.0..0.0 => 2, _ => 3 }
        }
        fn show<const N: usize>() { println!("{N}"); }
        fn main() {
            let negative_zero = -0.0f32;
            let root = 2f32.sqrt();
            let invalid = (-4.0f64).sqrt();
            let zero: f64 = Default::default();
            println!("{root:?} {} {} {} {} {} {} {:?}", negative_zero.is_sign_negative(), invalid.is_nan(),
                     f64::INFINITY.is_infinite(), f32::MAX.is_finite(), classify(1.5), classify(-0.2), zero.abs());
            show::<5>();
        }
    """, "1.4142135 true true true true 1 2 0.0\n5\n"),
    ("format macros handle alignment, numeric bases, and pretty debug", """
        #[derive(Debug)]
        struct Data(i32, Vec<(i32, bool)>);
        fn next() -> i32 { println!("next"); 7 }
        fn main() {
            let n = 1.234f64;
            println!("{n:.2} [{:+07.2}] [{:#06X}] [{:_^9.2}]", -3.125f32, -1i8, "Ωab");
            println!("[{0:e}] [{1:.2E}] [{0:#x}] [{name:>6}] {2:b}", 1200u64, 0.002f32, 3, name = "some");
            let built = format!("{{{w}}}-{1}-{0}-{0}", next(), 8, w = "value");
            print!("{built} ");
            println!("{:#?}", Data(1, vec![(2, false)]));
            println!("[{:.0}] [{:.2}] [{:>4}] [{:+}]", 'Ω', true, 'x', f64::NAN);
        }
    """, "1.23 [-003.12] [0x00FF] [___Ωa____]\n[1.2e3] [2.00E-3] [0x4b0] [  some] 11\n"
         "next\n{value}-8-7-7 Data(\n    1,\n    [\n        (\n            2,\n            false,\n"
         "        ),\n    ],\n)\n[] [tr] [   x] [NaN]\n"),
    ("format width and precision can use arguments and constants", """
        const WID: usize = 7;
        fn emit<const PLACES: usize>() {
            let width: usize = 8;
            println!("[{0:width$.PLACES$}] [{0:WID$.2}]", 1.23456);
        }
        fn main() {
            emit::<3>();
            println!("[{:.*}] [{:>width$}] [{2}:{width}]", 2, 1.234, "a", width = 6);
            println!("[{:1$}]", 42, 6);
            println!("{0}-{named}-{named}", 3, named = 7);
            println!("[{1:.*}] [{0:0$}]", 3, 0.00125);
        }
    """, "[   1.235] [   1.23]\n[1.23] [     a] [a:6]\n[    42]\n3-7-7\n[0.001] [  3]\n"),
]

INVALID = [
    ("selected references still reject overlapping mutable field borrows", """
        struct Pair { x: i32, y: i32 }
        fn exercise(pick: bool) -> i32 {
            let mut left = Pair { x: 1, y: 2 };
            let mut right = Pair { x: 3, y: 4 };
            let selected = if pick { &mut left } else { &mut right };
            let x = &mut selected.x;
            selected.x = 10;
            *x
        }
        fn main() { println!("{}", exercise(true)); }
    """, "mutably borrowed"),
    ("string search rejects a mutable reference as a pattern", """
        fn main() {
            let mut text = String::from("test");
            let pattern = text.as_mut_str();
            println!("{}", "test".contains(pattern));
        }
    """, "supports character and shared string patterns|does not implement std§str§Pattern"),
    ("uninitialized tuple bindings must each be assigned before use", """
        fn main() {
            let (left, right): (i32, i32);
            left = 1;
            println!("{left} {right}");
        }
    """, "uninitialized"),
    ("struct update cannot inherit a moved field", """
        struct Pair { a: String, b: String }
        fn main() {
            let old = Pair { a: String::new(), b: String::new() };
            let _moved = old.b;
            let _new = Pair { a: String::new(), ..old };
        }
    """, "use of moved value 'old.b'"),
    ("struct update requires inherited fields to be visible", """
        mod private {
            pub struct Pair { pub a: u8, b: u8 }
            pub fn base() -> Pair { Pair { a: 1, b: 2 } }
        }
        fn main() { let _new = private::Pair { a: 3, ..private::base() }; }
    """, "field 'b' is private"),
    ("destructuring assignment still checks destination mutability", """
        fn main() { let a = 0; (a,) = (1,); }
    """, "cannot assign to immutable binding"),
    ("destructuring assignment requires an irrefutable constructor", """
        fn main() { let mut a = 0; Some(a) = Some(1); }
    """, "refutable let pattern"),
    ("destructuring assignment rejects duplicate tuple rests", """
        fn main() { let a; (.., a, ..) = (1, 2); }
    """, "can contain only one"),
    ("concat rejects byte literals", """
        fn main() { println!("{}", concat!(b'a')); }
    """, "cannot concatenate a byte literal"),
    ("concat requires literal operands", """
        fn main() { println!("{}", concat!(("parenthesized"))); }
    """, "requires a literal"),
    ("macro_use does not import macros before the module declaration", """
        fn too_soon() { helper!(); }
        #[macro_use]
        mod helpers { macro_rules! helper { () => {}; } }
        fn main() {}
    """, "unknown macro 'helper'"),
    ("main requires the termination trait", """
        fn main() -> i32 { 1 }
    """, "does not implement .*Termination"),
    ("result-returning main requires a debug error type", """
        struct Failure;
        fn main() -> Result<(), Failure> { Ok(()) }
    """, "does not implement .*Termination"),
    ("foreign termination implementations require local types", """
        use std::process::{ExitCode, Termination};
        impl Termination for i32 {
            fn report(self) -> ExitCode { ExitCode::SUCCESS }
        }
        fn main() {}
    """, "Termination can only be implemented for a type defined in this crate"),
    ("raw identifiers still reject special path keywords", """
        fn main() { let r#self = 1; }
    """, "is reserved"),
    ("bare keywords cannot name lifetimes", """
        fn reference<'type>(value: &'type i32) -> &'type i32 { value }
        fn main() {}
    """, "lifetimes cannot use keyword names"),
    ("boxed self receivers move their owners", """
        trait Drain { fn take(self: Box<Self>) -> i32; }
        struct Unit;
        impl Drain for Unit { fn take(self: Box<Self>) -> i32 { 3 } }
        fn main() {
            let value: Box<dyn Drain> = Box::new(Unit);
            let first = value.take();
            println!("{first} {}", value.take());
        }
    """, "moved"),
    ("boxing a value cannot invent a trait implementation", """
        struct Unit;
        trait Marker {}
        fn make() -> Box<dyn Marker> { Box::new(Unit) }
        fn main() {}
    """, "Unit does not implement Marker"),
    ("borrowing box contents prevents moving their owner", """
        fn main() {
            let owned = Box::new(7);
            let borrow = owned.as_ref();
            let moved = owned;
            println!("{borrow} {moved}");
        }
    """, "borrowed"),
    ("dynamic box contents cannot move into a by-value receiver", """
        trait Read { fn consume(self); }
        struct Unit;
        impl Read for Unit { fn consume(self) {} }
        fn main() {
            let value: Box<dyn Read> = Box::new(Unit);
            value.consume();
        }
    """, "cannot pass an unsized trait object by value"),
    ("a static associated reference does not satisfy a higher-ranked equality", """
        trait Lending {
            type Item<'a> where Self: 'a;
        }
        struct Global;
        impl Lending for Global { type Item<'a> = &'static i32; }
        fn checked(_: impl for<'a> Lending<Item<'a> = &'a i32>) {}
        fn main() { checked(Global); }
    """, "associated type requires lifetime 'a, found 'static"),
    ("lending projections keep the argument borrowed through the returned reference", """
        trait Lending {
            type Item<'a> where Self: 'a;
            fn get<'a>(&'a self) -> Self::Item<'a>;
        }
        struct Packet(i32);
        impl Lending for Packet {
            type Item<'a> = &'a i32;
            fn get<'a>(&'a self) -> &'a i32 { &self.0 }
        }
        fn borrow<'s, R>(r: &'s R) -> &'s i32 where R: for<'a> Lending<Item<'a> = &'a i32> {
            r.get()
        }
        fn main() {
            let mut packet = Packet(17);
            let borrowed = borrow(&packet);
            packet.0 = 32;
            println!("{borrowed}");
        }
    """, "borrowed"),
    ("generic associated equality applies only to the named specialization", """
        trait Read {
            type Item<const N: usize>;
            fn read<const N: usize>(&self) -> Self::Item<N>;
        }
        fn get(source: impl Read<Item<2> = u8>) -> u8 { source.read::<3>() }
        fn main() {}
    """, "expected u8, found .*::Item<3>"),
    ("associated const arguments use the declared type", """
        trait Read { type Item<const N: usize>; }
        fn check(_: impl Read<Item<true> = u8>) {}
        fn main() {}
    """, "expected a usize constant"),
    ("argument position impl trait enforces associated equality", """
        trait Read { type Item; }
        struct Source;
        impl Read for Source { type Item = u16; }
        fn byte(_: impl Read<Item = u8>) {}
        fn main() { byte(Source); }
    """, "expected u8, found u16"),
    ("associated binding controls blanket implementation selection", """
        trait Read { type Item; }
        trait Marker {}
        struct Source;
        impl Read for Source { type Item = u16; }
        impl<T: Read<Item = u8>> Marker for T {}
        fn byte(_: impl Marker) {}
        fn main() { byte(Source); }
    """, "Source does not implement Marker"),
    ("explicit type arguments enforce a struct's associated bindings", """
        trait Read { type Item; }
        struct Source;
        impl Read for Source { type Item = u16; }
        struct Envelope<T: Read<Item = u8>>(T);
        fn main() { let _envelope: Envelope<Source>; }
    """, "expected u8, found u16"),
    ("opaque associated bindings enforce the return type's projection", """
        trait Read { type Item; }
        struct Unit;
        impl Read for Unit { type Item = u8; }
        fn packet() -> impl Read<Item = u16> { Unit }
        fn main() {}
    """, "expected u16, found u8"),
    ("opaque associated types stay hidden unless constrained", """
        trait Read { type Item; }
        struct Unit;
        impl Read for Unit { type Item = u8; }
        fn hidden() -> impl Read { Unit }
        fn exposed() -> impl Read<Item = u8> { hidden() }
        fn main() {}
    """, "expected u8, found impl Read returned by hidden::Item"),
    ("opaque return branches must use a single concrete type", """
        fn different(flag: bool) -> impl Copy {
            if flag { 2u8 } else { 2u16 }
        }
        fn main() {}
    """, "expected u8, found u16"),
    ("opaque types from distinct functions remain distinct", """
        fn first() -> impl Copy { 2u8 }
        fn second() -> impl Copy { 3u8 }
        fn same<T: Copy>(_: T, _: T) {}
        fn main() { same(first(), second()); }
    """, "expected impl Copy returned by first"),
    ("opaque returns expose only their declared bounds", """
        fn first() -> impl Copy { 2u8 }
        fn main() { println!("{:?}", first()); }
    """, "does not implement Debug"),
    ("opaque return values must satisfy declared bounds", """
        struct Unit;
        fn value() -> impl Copy { Unit }
        fn main() {}
    """, "Unit does not implement Copy"),
    ("an opaque return requires a resolvable concrete type", """
        fn again() -> impl Copy { (again(),) }
        fn main() {}
    """, "cannot resolve recursive opaque return type"),
    ("immutable statics cannot be assigned", """
        static NUMBER: i32 = 4;
        fn main() { NUMBER = 5; }
    """, "cannot mutate an immutable static item"),
    ("immutable statics cannot be mutably borrowed", """
        static NUMBER: i32 = 4;
        fn main() { let _ = &mut NUMBER; }
    """, "cannot mutate an immutable static item"),
    ("static bindings cannot be shadowed by patterns", """
        static NUMBER: i32 = 4;
        fn main() { let NUMBER = 5; }
    """, "pattern binding cannot shadow a static item"),
    ("expression fragments require a legal following token", """
        macro_rules! next { ($value:expr [ , ]) => {}; }
        fn main() {}
    """, "cannot follow macro fragment 'expr'"),
    ("macro definitions reject unseparated empty repetitions", """
        macro_rules! definitions { ($($visibility:vis)*) => {}; }
        fn main() {}
    """, "can match an empty token sequence"),
    ("macro definitions reject duplicate metavariable binding", """
        macro_rules! same { ($name:ident, $name:ident) => {}; }
        fn main() {}
    """, "duplicate macro matcher binding"),
    ("expr_2021 excludes top-level const blocks", """
        macro_rules! pass { ($value:expr_2021) => { $value }; }
        fn main() { println!("{}", pass!(const { 3 })); }
    """, "no matching rule"),
    ("ident fragments exclude the underscore placeholder", """
        macro_rules! accept { ($value:ident) => {}; }
        fn main() { accept!(_); }
    """, "no matching rule"),
    ("range checking uses the inferred literal width", """
        fn byte(_: u8) {}
        fn main() { let value = 256; byte(value); }
    """, "u8 arithmetic overflow"),
    ("a selected constant branch reports arithmetic failure", """
        const fn quotient(n: usize) -> usize { if n == 4 { 2 } else { 8 / n } }
        const QUOTIENT: usize = quotient(0);
        fn main() {}
    """, "division by zero"),
    ("inline const cannot read a runtime local", """
        fn main() {
            let runtime = 42;
            let value: i32 = const { runtime + 1 };
        }
    """, "unsupported constant expression"),
    ("all constant branches must have the same type", """
        const fn pick(flag: bool) -> u8 {
            if flag { 3u8 } else { true }
        }
        fn main() {}
    """, "expected a u8 constant"),
    ("ordinary functions cannot be called from a const body", """
        fn regular(x: i32) -> i32 { x }
        const fn constant(x: i32) -> i32 { regular(x) }
        fn main() {}
    """, "constant evaluation requires a const function"),
    ("constant function calls enforce argument types", """
        const fn identity(x: i32) -> i32 { x }
        const BAD: i32 = identity(true);
        fn main() {}
    """, "expected a i32 constant"),
    ("function parameters must be irrefutable", """
        fn get(Some(x): Option<i32>) -> i32 { x }
        fn main() {}
    """, "refutable parameter pattern"),
    ("parameter bindings cannot repeat across arguments", """
        fn get((left, _): (i32, i32), left: i32) -> i32 { left }
        fn main() {}
    """, "duplicate parameter 'left'"),
    ("anonymous impl Trait cannot be supplied explicitly", """
        fn copy<T: Copy>(first: T, second: impl Copy) {}
        fn main() { copy::<bool, i32>(true, 3); }
    """, "expects 1 type arguments"),
    ("anonymous impl Trait in a trait method is part of its signature", """
        struct Unit;
        trait Sink { fn write(&self, v: impl Copy); }
        impl Sink for Unit { fn write<T: Copy>(&self, v: T) {} }
        fn main() {}
    """, "incompatible generic parameters"),
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
    ("abstract generic body cannot assume Copy", """
        fn reuse<T>(value: T) -> T { let _taken = value; value }
        fn main() {}
    """, "use of moved value"),
    ("abstract generic body cannot assume Clone", """
        fn duplicate<T>(value: T) -> T { value.clone() }
        fn main() {}
    """, "T does not implement Clone"),
    ("generic obligations apply to inferred concrete type arguments", """
        struct Item(i32);
        #[derive(Clone, Copy)]
        struct Wrapper<T>(T);
        fn keep<T: Copy>(value: T) -> T { value }
        fn main() { let _result = keep(Wrapper(Item(1))); }
    """, "Wrapper<Item> does not implement Copy"),
    ("generic constraints apply to named type uses", """
        struct Wrapper<T: Copy>(T);
        fn wrap<T>(value: T) -> Wrapper<T> { Wrapper(value) }
        fn main() {}
    """, "T does not implement Copy"),
    ("generic type mismatch across arguments", """
        fn choose<T>(first: T, _: T) -> T { first }
        fn main() { let _result = choose(1, false); }
    """, "expected (i32|an integer), found bool"),
    ("generic default without context needs an annotation", """
        fn blank<T: Default>() -> T { T::default() }
        fn main() { println!("{}", blank()); }
    """, "type annotations needed for T"),
    ("overloaded arithmetic alone cannot infer a generic operand", """
        fn blank<T: Default>() -> T { T::default() }
        fn main() { let _answer: i32 = 3 + blank(); }
    """, "type annotations needed for T"),
    ("missing enum type argument needs an annotation", """
        enum Maybe<T> { Absent, Present(T) }
        fn main() { let _value = Maybe::Absent; }
    """, "type annotations needed for T"),
    ("generic enum exhaustiveness specializes payload type", """
        enum Maybe<T> { Absent, Present(T) }
        fn describe(value: Maybe<bool>) -> i32 {
            match value { Maybe::Absent => 0, Maybe::Present(true) => 1 }
        }
        fn main() {}
    """, "Maybe::Present\\(false\\)"),
    ("recursive generic layout is diagnosed", """
        struct Wrapper<T>(T);
        struct Recursive<T>(Wrapper<Recursive<Wrapper<T>>>);
        fn main() {}
    """, "infinite size"),
    ("comparison by trait cannot overwrite a borrowed operand", """
        #[derive(PartialEq)]
        struct Item(i32);
        fn main() {
            let mut value = Item(1);
            let _answer = value == { value = Item(2); Item(3) };
        }
    """, "immutably borrowed"),
    ("trait implementation must provide required methods", """
        trait Read { fn read(&self) -> i32; }
        struct Data;
        impl Read for Data {}
        fn main() {}
    """, "missing method 'read'"),
    ("trait implementation signature must match", """
        trait Read { fn read(&self) -> i32; }
        struct Data;
        impl Read for Data { fn read(&self) -> bool { true } }
        fn main() {}
    """, "incompatible trait signature"),
    ("overlapping trait impls are rejected", """
        trait Read {}
        struct Data;
        impl<T> Read for T {}
        impl Read for Data {}
        fn main() {}
    """, "conflicting implementations"),
    ("trait bound is required on generic calls", """
        trait Read { fn read(&self) -> i32; }
        fn consume<T: Read>(value: T) { let _x = value.read(); }
        fn main() { consume(7); }
    """, "i32 does not implement Read"),
    ("associated type bounds are enforced", """
        trait Read { type Item: Copy; }
        struct Data;
        struct Item;
        impl Read for Data { type Item = Item; }
        fn main() {}
    """, "Item does not implement Copy"),
    ("const argument kinds are checked", """
        fn read<const N: usize>() -> usize { N }
        fn main() { let _value = read::<true>(); }
    """, "expected a usize constant"),
    ("inferred const lengths must agree", """
        fn pair<const N: usize>(_: [i32; N], _: [i32; N]) {}
        fn main() { pair([1, 2], [1]); }
    """, "expected 2, found 1"),
    ("repeat with generic const length requires Copy", """
        fn array<T, const N: usize>(value: T) -> [T; N] { [value; N] }
        fn main() {}
    """, "T does not implement Copy"),
    ("moving non-Copy array element by index is rejected", """
        struct Item(i32);
        fn main() { let items = [Item(1)]; let _moved = items[0]; }
    """, "cannot move out of array index"),
    ("declarative macro reports no matching arm", """
        macro_rules! require_name { ($name:ident) => { 0 }; }
        fn main() { let _value = require_name!(3); }
    """, "no matching rule for require_name"),
    ("trait objects reject generic associated types", """
        trait Family { type Item<T>; }
        fn object(_: &dyn Family) {}
        fn main() {}
    """, "not dyn-compatible"),
    ("trait object coercions check associated equalities", """
        trait Read { type Item; fn read(&self) -> Self::Item; }
        struct Cell;
        impl Read for Cell { type Item = bool; fn read(&self) -> bool { true } }
        fn main() { let _object: &dyn Read<Item = i32> = &Cell; }
    """, "expected i32, found bool"),
    ("generic associated type argument bounds are checked", """
        trait Family { type Item<T: Copy>; }
        struct Owner;
        struct NonCopy(i32);
        impl Family for Owner { type Item<T: Copy> = T; }
        fn bad(_: <Owner as Family>::Item<NonCopy>) {}
        fn main() {}
    """, "NonCopy does not implement Copy"),
    ("lending projection keeps receiver borrowed", """
        trait Lending {
            type Item<'a> where Self: 'a;
            fn get<'a>(&'a self) -> Self::Item<'a>;
        }
        struct Cell(i32);
        impl Lending for Cell {
            type Item<'a> = &'a i32;
            fn get<'a>(&'a self) -> Self::Item<'a> { &self.0 }
        }
        fn main() {
            let mut cell = Cell(7);
            let item = cell.get();
            cell.0 = 9;
            println!("{item}");
        }
    """, "immutably borrowed"),
    ("macro literals do not capture invocation locals", """
        macro_rules! invalid { () => { captured }; }
        fn main() { let captured = 4; println!("{}", invalid!()); }
    """, "unknown binding 'captured'"),
    ("module function is private to its declaring scope", """
        mod nested { fn secret() -> i32 { 3 } }
        fn main() { let _value = nested::secret(); }
    """, "'secret' is private"),
    ("private record field cannot be read from the parent", """
        mod nested {
            pub struct Data { value: i32 }
            pub fn make() -> Data { Data { value: 3 } }
        }
        fn main() { let _value = nested::make().value; }
    """, "field 'value' is private"),
    ("private tuple field hides constructor", """
        mod nested { pub struct Data(i32); }
        fn main() { let _value = nested::Data(3); }
    """, "has private fields"),
    ("inherent method visibility is checked", """
        mod nested { pub struct Data; impl Data { fn hidden(&self) {} } }
        fn main() { nested::Data.hidden(); }
    """, "method 'hidden' is private"),
    ("out of scope traits do not add methods", """
        mod nested {
            pub trait Read { fn get(&self) -> i32; }
            pub struct Data;
            impl Read for Data { fn get(&self) -> i32 { 3 } }
        }
        fn main() { let _value = nested::Data.get(); }
    """, "no method 'get'"),
    ("cannot reexport a private function publicly", """
        mod nested { fn secret() {} pub use self::secret as exposed; }
        fn main() {}
    """, "cannot re-export private item"),
    ("ambiguous glob imports need an explicit namespace", """
        mod a { pub fn value() {} }
        mod b { pub fn value() {} }
        use a::*;
        use b::*;
        fn main() { value(); }
    """, "ambiguous glob import 'value'"),
    ("qualified const arguments require braces", """
        mod sizes { pub const N: usize = 3; }
        fn size<const N: usize>() -> usize { N }
        fn main() { println!("{}", size::<sizes::N>()); }
    """, "expected a usize constant"),
    ("question mark requires a compatible return family", """
        fn not_an_option() -> i32 { Some(3)? }
        fn main() {}
    """, "requires a function returning the same enum family"),
    ("indirect call argument must match", """
        fn target(n: i32) -> i32 { n }
        fn main() { let f: fn(i32) -> i32 = target; let _ = f(true); }
    """, "expected i32, found bool"),
    ("shared closure capture prevents mutation until use", """
        fn main() {
            let mut base = 3;
            let call = || base;
            base = 4;
            println!("{}", call());
        }
    """, "immutably borrowed"),
    ("capturing closure cannot coerce to a function pointer", """
        fn main() { let n = 3; let _call: fn() -> i32 = || n; }
    """, "expected fn"),
    ("mutable closure invocation requires a mutable receiver", """
        fn main() { let mut n = 0; let call = || { n += 1; }; call(); }
    """, "immutable binding"),
    ("consuming a move closure makes another call invalid", """
        struct Item(i32);
        fn consume(item: Item) -> i32 { item.0 }
        fn main() { let saved = Item(6); let once = move || consume(saved); once(); once(); }
    """, "use of moved value 'once'"),
    ("moving a shared reference into a closure preserves its loan", """
        fn main() {
            let mut n = 3;
            let r = &n;
            let f = move || *r;
            n = 4;
            println!("{}", f());
        }
    """, "immutably borrowed"),
    ("mutably captured reference cannot be used before the closure's last call", """
        fn main() {
            let mut n = 3;
            let r = &mut n;
            let mut f = || { *r += 1; };
            *r = 4;
            f();
        }
    """, "mutably borrowed"),
    ("tuple move prevents reading the entire tuple", """
        #[derive(Debug)]
        struct Item(i32);
        fn main() {
            let tuple = (Item(1), false);
            let (owned, ..) = tuple;
            println!("{tuple:?} {owned:?}");
        }
    """, "use of partially moved value"),
    ("tuple patterns must cover nested elements", """
        fn main() { match (1, false) { (_, true) => () } }
    """, r"non-exhaustive match, missing \(_, false\)"),
    ("array pattern needs the right number of elements", """
        fn main() { let [a, b] = [1, 2, 3]; }
    """, "array pattern takes 3 fields"),
    ("array patterns need a fixed length", """
        fn unknown<const N: usize>(values: [bool; N]) { let [..] = values; }
        fn main() {}
    """, "array pattern requires a fixed length"),
    ("array match must cover all combinations", """
        fn main() { match [true, true] { [false, ..] | [.., false] => () } }
    """, r"non-exhaustive match, missing \[true, true\]"),
    ("array pattern cannot move an already moved element", """
        struct Item(i32);
        fn main() {
            let items = [Item(1), Item(2)];
            let [first, ..] = items;
            let [other, ..] = items;
        }
    """, "use of moved value 'items.0'"),
    ("borrowed slices preserve the array's lifetime", """
        fn view(values: &[i32; 2]) -> &[i32] { values }
        fn main() {
            let mut a = [1, 2];
            let slice = view(&a);
            a[0] = 3;
            println!("{slice:?}");
        }
    """, "immutably borrowed"),
    ("subslice of a local cannot escape", """
        fn invalid() -> &'static [i32] {
            let a = [1, 2];
            &a[..]
        }
        fn main() {}
    """, "outlives its block"),
    ("slice values cannot be bound without indirection", """
        fn main() {
            let a = [1, 2];
            let unsized_value = a[..];
        }
    """, "cannot bind an unsized slice value"),
    ("formatting a slice requires a reference", """
        fn main() { let a = [1, 2]; println!("{:?}", a[..]); }
    """, "formatting an unsized slice requires a reference"),
    ("empty slice missing from an otherwise exhaustive match", """
        fn incomplete(s: &[bool]) -> bool { match *s { [first, ..] => first } }
        fn main() {}
    """, r"non-exhaustive match, missing \[\]"),
    ("matching a borrowed slice cannot move non-Copy elements", """
        struct Item(i32);
        fn first(s: &[Item]) -> i32 {
            let [item, ..] = *s else { return 0; };
            item.0
        }
        fn main() {}
    """, "Item does not implement Copy"),
    ("signed integer patterns must remain in bounds", """
        fn classify(value: i8) -> bool { matches!(value, -129..=127) }
        fn main() {}
    """, "i8 arithmetic overflow"),
    ("constant operands are checked before computing the result", """
        const VALUE: u8 = 256u8 - 1;
        fn main() { println!("{VALUE}"); }
    """, "u8 arithmetic overflow"),
    ("unsigned integer negation is invalid even for zero", """
        const VALUE: u8 = -0;
        fn main() { println!("{VALUE}"); }
    """, "cannot negate u8"),
    ("arithmetic requires matching integer widths", """
        fn main() { let _ = 1i16 + 1i64; }
    """, "expected i16, found i64"),
    ("character literals cannot encode surrogates", r"""
        fn main() { let _ = '\u{D800}'; }
    """, "invalid Unicode escape"),
    ("byte literals require ASCII or a byte escape", """
        fn main() { let _ = b'é'; }
    """, "non-ASCII character"),
    ("integer to character casts require a byte", """
        fn main() { let _ = 65u16 as char; }
    """, "cannot cast u16 to char"),
    ("constant shifts are checked against the left operand width", """
        const INVALID: u8 = 1 << 8;
        fn main() { println!("{}", INVALID); }
    """, "shift amount is outside the integer width"),
    ("shift counts must be integers", """
        fn main() { let _ = 2u8 << false; }
    """, "expected an integer, found bool"),
    ("cyclic type aliases are rejected", """
        type One = Two;
        type Two = One;
        fn main() {}
    """, "cyclic or unresolved type alias"),
    ("reference aliases require a lifetime in their definition", """
        type Ref = &i32;
        fn main() {}
    """, "reference type alias requires an explicit lifetime"),
    ("alias const arguments are checked before expansion", """
        type Bytes<const N: usize> = [u8; 2];
        fn main() { let _: Bytes<true> = [1, 2]; }
    """, "expected a usize constant"),
    ("alias type parameters must be used", """
        type Missing<T> = bool;
        fn main() {}
    """, "unused type parameter T"),
    ("a pattern reference prevents mutation while it remains live", """
        fn main() {
            let mut values = [1, 2];
            let [first, _] = &values;
            values[0] = 4;
            println!("{first}");
        }
    """, "immutably borrowed"),
    ("a pattern reference cannot write through a shared loan", """
        fn main() {
            let [first, _] = &[1, 2];
            *first = 4;
        }
    """, "cannot write through a shared reference"),
    ("mutable pattern binding prevents reusing the original reference", """
        fn main() {
            let mut values = [1, 2];
            let slice = &mut values[..];
            let [first, ..] = slice else { return; };
            slice[0] = 7;
            *first = 8;
        }
    """, "mutably borrowed"),
    ("mutable pattern bindings cannot write in guards", """
        fn main() {
            let mut values = [1, 2];
            match &mut values {
                [first, _] if { *first = 3; true } => (),
                _ => (),
            }
        }
    """, "in a pattern guard"),
    ("an explicit mutable pattern needs a mutable source", """
        fn main() {
            let value = (1, 2);
            let (ref mut first, _) = value;
            *first = 3;
        }
    """, "cannot borrow immutable place"),
    ("explicit reference patterns must match mutability", """
        fn main() { let mut n = 1; let &value = &mut n; }
    """, "reference pattern must match the reference's mutability"),
    ("a reference to a match temporary cannot escape its statement", """
        struct Point(i32);
        fn make() -> Point { Point(3) }
        fn main() {
            let out = match make() { Point(ref n) => n };
            println!("{out}");
        }
    """, "temporary value dropped while borrowed"),
    ("at bindings cannot both move a non-Copy value", """
        struct Item(i32);
        fn main() { let _whole @ _part = Item(1); }
    """, "Item does not implement Copy"),
    ("at bindings cannot overlap a mutable borrow", """
        fn main() {
            let mut pair = (1, 2);
            let ref mut _whole @ (ref _left, _) = pair;
        }
    """, "overlapping mutable borrows"),
    ("at binding names cannot repeat inside the subpattern", """
        fn main() { let _x @ (_x, _) = (1, 2); }
    """, "duplicate pattern binding"),
    ("a borrowed vector slice keeps its heap storage immutable", """
        fn main() {
            let mut values = vec![1, 2];
            let slice = values.as_slice();
            values.push(3);
            println!("{slice:?}");
        }
    """, "immutably borrowed"),
    ("mutating a vector requires a mutable place", """
        fn main() { let v = vec![1]; v.push(2); }
    """, "cannot borrow immutable place"),
    ("repeating a vector element requires Clone", """
        struct Item(i32);
        fn main() { let _ = vec![Item(1); 2]; }
    """, "Item does not implement Clone"),
    ("using a moved vector is rejected", """
        fn main() { let a = vec![1]; let b = a; println!("{a:?} {b:?}"); }
    """, "use of moved value 'a'"),
    ("for loop patterns must match every element", """
        fn main() { for Some(n) in vec![Some(1), None] { println!("{n}"); } }
    """, "refutable for pattern"),
    ("borrowed iteration holds the collection loan across iterations", """
        fn main() {
            let mut v = vec![1, 2];
            for &value in &v { v.push(value); }
        }
    """, "immutably borrowed"),
    ("owned iteration consumes vectors", """
        fn main() {
            let v = vec![1, 2];
            for n in v { println!("{n}"); }
            println!("{v:?}");
        }
    """, "use of moved value 'v'"),
    ("a stored iterator retains its collection loan", """
        fn main() {
            let mut v = vec![1, 2];
            let it = v.iter().rev();
            v.push(3);
            for value in it { println!("{value}"); }
        }
    """, "immutably borrowed"),
    ("using an iterator over a dropped temporary is invalid", """
        fn main() {
            let it = vec![1, 2].iter();
            for value in it { println!("{value}"); }
        }
    """, "temporary value dropped while borrowed"),
    ("for loops consume a stored iterator", """
        fn main() {
            let a = [1, 2];
            let it = a.iter();
            for value in it { println!("{value}"); }
            for value in it { println!("{value}"); }
        }
    """, "use of moved value 'it'"),
    ("an iterator map callback must be callable more than once", """
        struct Item(i32);
        fn consume(item: Item) -> i32 { item.0 }
        fn main() {
            let value = Item(3);
            for n in vec![1, 2].into_iter().map(move |n| consume(value) + n) { println!("{n}"); }
        }
    """, "iterator map cannot use a closure that consumes its captures"),
    ("creating a mapper cannot conflict with the live source borrow", """
        fn main() {
            let mut v = vec![1usize];
            for n in v.iter_mut().map(|value| *value + v.len()) { println!("{n}"); }
        }
    """, "mutably borrowed"),
    ("adding strings consumes the owned left operand", """
        fn main() {
            let a = String::from("Rust");
            let b = a + " in Python";
            println!("{a} {b}");
        }
    """, "use of moved value 'a'"),
    ("writing a string requires its active shared reference to end", """
        fn main() {
            let mut text = String::from("rust");
            let borrowed = &text;
            text.push('!');
            println!("{borrowed}");
        }
    """, "immutably borrowed"),
    ("borrowed strings cannot outlive their owner", """
        fn invalid() -> &'static str {
            let text = String::from("rust");
            text.as_str()
        }
        fn main() {}
    """, "outlives its block"),
    ("dynamic string views cannot be stored in static string fields", """
        struct Message { name: &'static str }
        fn main() {
            let text = String::from("rust");
            let _message = Message { name: &text };
        }
    """, "reference does not live for 'static"),
    ("string iterators hold a borrow of the original text", """
        fn main() {
            let mut text = String::from("x");
            let chars = text.chars();
            text.push('y');
            for character in chars { println!("{character}"); }
        }
    """, "immutably borrowed"),
    ("collect requires an inferred or explicit destination type", """
        fn main() { let _value = [1, 2].into_iter().collect(); }
    """, "type annotations needed for collect"),
    ("iterator consumers move the iterator", """
        fn main() {
            let data = vec![1, 2];
            let it = data.into_iter();
            println!("{}", it.count());
            println!("{}", it.count());
        }
    """, "use of moved value 'it'"),
    ("overflowing float literals are rejected", """
        fn main() { let _ = 1e100f32; }
    """, "floating-point literal is out of range for f32"),
    ("floating-point types cannot derive Eq", """
        #[derive(PartialEq, Eq)]
        struct Bad(f64);
        fn main() {}
    """, "f64 does not implement Eq"),
    ("float const parameter types are not allowed", """
        fn invalid<const VALUE: f64>() {}
        fn main() {}
    """, "unsupported const parameter type"),
    ("format macro arguments must be used", """
        fn main() { println!("literal", 2); }
    """, "unused format argument"),
    ("dynamic format width must be usize", """
        fn main() { let width = 4i32; println!("{:width$}", 2); }
    """, "expected usize, found i32"),
    ("base formatting does not accept floats", """
        fn main() { println!("{:x}", 1.5f32); }
    """, "base formatting requires an integer"),
]


class InterpreterTests(unittest.TestCase):
    @unittest.skipUnless(shutil.which("rustc"), "rustc is optional")
    def test_source_location_and_file_inclusion(self):
        with TemporaryDirectory() as directory:
            folder = Path(directory)
            (folder / "nested").mkdir()
            (folder / "nested" / "mod.rs").write_text(
                'macro_rules! source { ($value:expr) => {{\n'
                '    println!("{}:{}:{} {} {}", file!(), line!(), column!(), module_path!(), $value);\n'
                '    println!("{:?}", include_str!("text.txt"));\n'
                '}} }\n'
                'pub fn show() { println!("{} {}", file!(), module_path!()); source!(line!()); }\n',
                encoding="utf-8",
            )
            (folder / "text.txt").write_bytes("root β\r\n".encode("utf-8"))
            (folder / "nested" / "text.txt").write_bytes(b"nested\n")
            (folder / "data.bin").write_bytes(b"\x00\x7f\x80\xff")
            source = folder / "demo-crate.rs"
            source.write_text(
                '#[macro_use] mod nested;\n'
                'fn main() {\n'
                '    source!(\n'
                '        line!()\n'
                '    );\n'
                '    nested::show();\n'
                '    let data: &[u8; 4] = std::include_bytes!(concat!("data", ".bin"),);\n'
                '    println!("{data:?}");\n'
                '}\n',
                encoding="utf-8",
            )
            invalid = folder / "invalid_encoding.rs"
            invalid.write_text('fn main() { println!("{}", include_str!("data.bin")); }', encoding="utf-8")
            bad_path = folder / "missing_path.rs"
            bad_path.write_text('fn main() { println!("{:?}", include_bytes!("missing")); }', encoding="utf-8")
            runner = Path(__file__).with_name("conformance.py")
            report = subprocess.run([sys.executable, runner, "--json", source, invalid, bad_path],
                                    text=True, capture_output=True, timeout=30, check=False)
            self.assertEqual(report.returncode, 0, report.stdout + report.stderr)
            cases = json.loads(report.stdout)["cases"]
            self.assertEqual([case["outcome"] for case in cases],
                             ["runtime_match", "both_rejected", "both_rejected"])

    @unittest.skipUnless(shutil.which("rustc"), "rustc is optional")
    def test_external_modules_and_conformance_report(self):
        with TemporaryDirectory() as folder:
            folder = Path(folder)
            (folder / "outer").mkdir()
            (folder / "outer" / "mod.rs").write_text(
                "mod leaf; pub fn value() -> i32 { leaf::value() }", encoding="utf-8")
            (folder / "outer" / "leaf.rs").write_text(
                "pub(super) fn value() -> i32 { 6 }", encoding="utf-8")
            (folder / "renamed.rs").write_text("pub fn value() -> i32 { 4 }", encoding="utf-8")
            root = folder / "main.rs"
            root.write_text("""
                mod outer;
                #[path = r#"renamed.rs"#] mod alias;
                fn main() { println!("{} {}", outer::value(), alias::value()); }
            """, encoding="utf-8")
            bad = folder / "invalid.rs"
            bad.write_text("fn main() { let invalid: bool = 1; }", encoding="utf-8")
            capture = folder / "capture.rs"
            capture.write_text('fn main() { let x = 1; let r = &x; let f = || *r; println!("{}", f()); }',
                               encoding="utf-8")
            slices = folder / "slice.rs"
            slices.write_text('fn main() { let numbers: &[i32] = &[1, 2]; println!("{}", numbers[0]); }',
                              encoding="utf-8")
            slice_pattern = folder / "slice_pattern.rs"
            slice_pattern.write_text('fn main() { let a = [1, 2]; assert!(matches!(a[..], [1, ..])); }',
                                     encoding="utf-8")
            shared = folder / "shared_pattern.rs"
            shared.write_text('''
                fn main() {
                    let a: &[i32] = &[1, 2];
                    let first = match a { [n, ..] => *n, _ => 0 };
                    println!("{first}");
                }
            ''', encoding="utf-8")
            mutable = folder / "mutable_pattern.rs"
            mutable.write_text('''
                fn main() {
                    let mut a = [1, 2];
                    match &mut a[..] { [n, ..] => *n = 7, _ => () }
                    println!("{a:?}");
                }
            ''', encoding="utf-8")
            explicit = folder / "explicit_ref_pattern.rs"
            explicit.write_text('''
                fn main() {
                    let tuple = (1, 2);
                    let (ref first, _) = tuple;
                    println!("{first}");
                }
            ''', encoding="utf-8")
            at_pattern = folder / "binding_subpattern.rs"
            at_pattern.write_text('''
                fn main() {
                    let value = match 2 { n @ 1..=3 => n, _ => 0 };
                    println!("{value}");
                }
            ''', encoding="utf-8")
            vector = folder / "heap_vector.rs"
            vector.write_text('''
                fn main() {
                    let mut values = vec![1, 2];
                    values.push(3);
                    println!("{values:?}");
                }
            ''', encoding="utf-8")
            for_iteration = folder / "for_iterator.rs"
            for_iteration.write_text('''
                fn main() {
                    for n in vec![1, 2, 3] { println!("{n}"); }
                }
            ''', encoding="utf-8")
            iterator = folder / "iterator_method.rs"
            iterator.write_text('''
                fn main() {
                    let v = vec![1, 2, 3];
                    for n in v.iter() { println!("{n}"); }
                }
            ''', encoding="utf-8")
            map_iterator = folder / "iterator_adapter.rs"
            map_iterator.write_text('''
                fn main() {
                    for n in vec![1, 2, 3].into_iter().map(|n| n + 1) { println!("{n}"); }
                }
            ''', encoding="utf-8")
            string = folder / "owned_string.rs"
            string.write_text('''
                fn main() {
                    let mut text = String::from("Rust");
                    text.push('!');
                    println!("{text}");
                }
            ''', encoding="utf-8")
            borrowed_string = folder / "borrowed_string.rs"
            borrowed_string.write_text('''
                fn main() {
                    let text = String::from("rust");
                    println!("{}", text.as_str());
                }
            ''', encoding="utf-8")
            collect = folder / "iterator_collect.rs"
            collect.write_text('''
                fn main() {
                    let text: String = "abc".chars().rev().collect();
                    println!("{text}");
                }
            ''', encoding="utf-8")
            floating = folder / "floating_point.rs"
            floating.write_text('''
                fn main() {
                    let n = 1.5f64;
                    println!("{}", n * 2.0);
                }
            ''', encoding="utf-8")
            formatting = folder / "format_precision.rs"
            formatting.write_text('''
                fn main() { println!("{:.2}", 1.234f64); }
            ''', encoding="utf-8")
            static_slice = folder / "static_item.rs"
            static_slice.write_text('''
                static NUMBERS: &[i32] = &[3];
                fn main() { println!("{}", NUMBERS[0]); }
            ''', encoding="utf-8")
            panics = folder / "panics.rs"
            panics.write_text('fn main() { println!("before"); assert_eq!(1, 2); }', encoding="utf-8")
            bounds = folder / "bounds.rs"
            bounds.write_text('''
                fn read(data: &[i32], at: usize) -> i32 { data[at] }
                fn main() { let a = [1, 2, 3]; println!("{}", read(&a[..1], 2)); }
            ''', encoding="utf-8")
            overflow = folder / "overflow.rs"
            overflow.write_text('''
                fn increase(n: u8) -> u8 { n + 1 }
                fn main() { println!("before"); increase(u8::MAX); }
            ''', encoding="utf-8")
            shift = folder / "shift.rs"
            shift.write_text('''
                fn rotate(n: u8, amount: i16) -> u8 { n << amount }
                fn main() { println!("before"); rotate(1, -1); }
            ''', encoding="utf-8")
            utf8 = folder / "utf8.rs"
            utf8.write_text('''
                fn main() {
                    let mut text = String::from("🦀");
                    println!("before");
                    text.truncate(1);
                }
            ''', encoding="utf-8")
            substring = folder / "substring.rs"
            substring.write_text('''
                fn main() {
                    let text = String::from("🦀");
                    println!("before");
                    println!("{}", &text[1..2]);
                }
            ''', encoding="utf-8")
            runner = Path(__file__).with_name("conformance.py")
            report = subprocess.run([sys.executable, runner, "--json", root, bad,
                                     capture, slices, slice_pattern, shared, mutable, explicit, at_pattern, vector,
                                     for_iteration, iterator, map_iterator, string, borrowed_string, collect, floating,
                                     formatting, static_slice,
                                     panics, bounds, overflow, shift, utf8, substring],
                                    text=True, capture_output=True, timeout=40, check=False)
            self.assertEqual(report.returncode, 0, report.stderr)
            cases = json.loads(report.stdout)["cases"]
            self.assertEqual([case["outcome"] for case in cases],
                             ["runtime_match", "both_rejected", "runtime_match", "runtime_match", "runtime_match", "runtime_match",
                              "runtime_match", "runtime_match", "runtime_match", "runtime_match", "runtime_match",
                              "runtime_match", "runtime_match",
                              "runtime_match", "runtime_match", "runtime_match", "runtime_match", "runtime_match",
                              "runtime_match",
                              "runtime_match", "runtime_match", "runtime_match",
                              "runtime_match", "runtime_match", "runtime_match"])
            self.assertEqual(cases[0]["run"]["python"]["stdout"], "6 4\n")
            self.assertEqual(cases[-1]["run"]["python"]["status"], 101)

    def test_program_corpus(self):
        """Each programs/*.rs matches the exit status and output rustc recorded in its .out file."""
        folder = Path(__file__).with_name("programs")
        for source in sorted(folder.glob("*.rs")):
            with self.subTest(program=source.name):
                status, _, expected = source.with_suffix(".out").read_text(encoding="utf-8").partition("\n")
                output = StringIO()
                try:
                    actual = run(source.read_text(encoding="utf-8"), output, source_path=source) or 0
                except RustPanic:
                    actual = 101
                self.assertEqual((actual, output.getvalue()), (int(status), expected))

    def test_valid_programs(self):
        for name, source, expected in VALID:
            with self.subTest(name=name):
                output = StringIO()
                run(source, output)
                self.assertEqual(output.getvalue(), expected)

    @unittest.skipUnless(shutil.which("rustc"), "rustc is optional")
    def test_edition_rules(self):
        with TemporaryDirectory() as folder:
            folder = Path(folder)
            expressions = folder / "expressions.rs"
            expressions.write_text("""
                macro_rules! select {
                    ($value:expr) => { 1 };
                    (const $body:block) => { 2 };
                    (_) => { 3 };
                }
                macro_rules! legacy {
                    ($value:expr_2021) => { 4 };
                    (const $body:block) => { 5 };
                }
                fn main() {
                    println!("{} {} {}", select!(const { 3 }), select!(_), legacy!(const { 3 }));
                }
            """, encoding="utf-8")
            patterns = folder / "patterns.rs"
            patterns.write_text("""
                macro_rules! either {
                    ($a:pat | $b:pat) => {
                        match 2 { $a | $b => true, _ => false }
                    };
                }
                fn main() { println!("{}", either!(1 | 2)); }
            """, encoding="utf-8")
            keywords = folder / "keywords.rs"
            keywords.write_text('fn gen() -> i32 { 9 } fn main() { println!("{}", gen()); }', encoding="utf-8")
            lifetimes = folder / "lifetimes.rs"
            lifetimes.write_text("""
                fn alias<'r#type>(value: &'r#type i32) -> &'r#type i32 { value }
                fn main() { println!("{}", alias(&9)); }
            """, encoding="utf-8")
            chains = folder / "chains.rs"
            chains.write_text("""
                fn noisy() -> Option<i32> { println!("should never run"); Some(1) }
                fn step(i: i32) -> Option<i32> { if i < 5 { Some(i) } else { None } }
                fn choose<'a>(value: &'a Option<i32>, fallback: &'a i32) -> &'a i32 {
                    if let Some(number) = value && *number > 2 { number } else { fallback }
                }
                fn main() {
                    let number = 7;
                    let shadowed = if let Some(number) = Some(0) && number > 0 { number } else { number };
                    if false && let Some(value) = noisy() { println!("{value}"); }
                    let mut at = 0;
                    let mut total = 0;
                    'next: while let Some(value) = step(at) && value < 4 {
                        at += 1;
                        if value == 2 { continue 'next; }
                        total += value;
                    }
                    let option = Some(6);
                    println!("{} {} {}", shadowed, total, choose(&option, &number));
                }
            """, encoding="utf-8")
            move = folder / "chain_move.rs"
            move.write_text("""
                fn main() {
                    let parcel = Some(String::from("owned"));
                    if let Some(taken) = parcel && false { println!("{taken}"); }
                    println!("{}", parcel.is_some());
                }
            """, encoding="utf-8")
            runner = Path(__file__).with_name("conformance.py")
            for edition, expected, pattern_outcome in (
                ("2018", "2 3 5\n", "runtime_match"),
                ("2021", "2 3 5\n", "both_rejected"),
                ("2024", "1 1 5\n", "both_rejected"),
            ):
                with self.subTest(edition=edition):
                    report = subprocess.run([sys.executable, runner, "--json", f"--edition={edition}",
                                             expressions, patterns, keywords, lifetimes, chains, move],
                                            text=True, capture_output=True,
                                            timeout=15, check=False)
                    self.assertEqual(report.returncode, 0, report.stderr or report.stdout)
                    exprs, pats, names, regions, chain, moved = json.loads(report.stdout)["cases"]
                    self.assertEqual(exprs["outcome"], "runtime_match")
                    self.assertEqual(exprs["run"]["python"]["stdout"], expected)
                    self.assertEqual(pats["outcome"], pattern_outcome)
                    self.assertEqual(names["outcome"], "both_rejected" if edition == "2024" else "runtime_match")
                    self.assertEqual(regions["outcome"], "both_rejected" if edition == "2018" else "runtime_match")
                    self.assertEqual(chain["outcome"], "runtime_match" if edition == "2024" else "both_rejected")
                    if edition == "2024":
                        self.assertEqual(chain["run"]["python"]["stdout"], "7 4 6\n")
                    self.assertEqual(moved["outcome"], "both_rejected")

    def test_invalid_programs(self):
        for name, source, message in INVALID:
            with self.subTest(name=name):
                with self.assertRaisesRegex(RustError, message):
                    run(source, StringIO())

    @unittest.skipUnless(shutil.which("rustc"), "rustc is optional")
    def test_main_termination(self):
        cases = [
            ("result failure", """
                #[derive(Debug)]
                struct Failure { reason: u8 }
                fn lookup() -> Result<u8, Failure> { Err(Failure { reason: 3 }) }
                fn main() -> Result<(), Failure> {
                    println!("before");
                    lookup()?;
                    Ok(())
                }
            """, 1),
            ("custom report", """
                use std::process::{ExitCode, Termination};
                struct Status(u8);
                impl Termination for Status {
                    fn report(self) -> ExitCode {
                        println!("report {}", self.0);
                        ExitCode::from(self.0)
                    }
                }
                fn main() -> impl Termination { Status(13) }
            """, 13),
            ("result exit code", """
                use std::process::ExitCode;
                fn main() -> Result<ExitCode, ()> { Ok(ExitCode::from(7)) }
            """, 7),
            ("explicit process exit", """
                use std::process::exit;
                fn main() -> ! { eprintln!("stopping"); exit(9) }
            """, 9),
        ]
        with TemporaryDirectory() as folder:
            source_file = Path(folder) / "termination.rs"
            binary = Path(folder) / "native"
            interpreter = Path(__file__).with_name("rust_in_python.py")
            for name, source, status in cases:
                with self.subTest(name=name):
                    compiled = self.compile(source_file, binary, source)
                    self.assertEqual(compiled.returncode, 0, compiled.stderr)
                    native = subprocess.run([binary], text=True, capture_output=True, timeout=10)
                    python = subprocess.run([sys.executable, interpreter, source_file],
                                            text=True, capture_output=True, timeout=10)
                    self.assertEqual(native.returncode, status)
                    self.assertEqual((python.returncode, python.stdout, python.stderr),
                                     (native.returncode, native.stdout, native.stderr))

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
            env={**os.environ, "RUSTC_BOOTSTRAP": "1"} if "feature(generic_const_exprs)" in source else None,
        )

    @unittest.skipUnless(shutil.which("rustc"), "rustc is optional")
    def test_compiled_macro_hygiene_and_original_source_spans(self):
        with TemporaryDirectory() as directory:
            folder = Path(directory)
            declarative = folder / "foreign.rs"
            declarative.write_text("""
                #[macro_export]
                macro_rules! combine {
                    ($value:expr) => {{
                        let hidden = 7;
                        hidden + $value
                    }};
                }
                #[macro_export]
                macro_rules! inner { () => { 11 }; }
                #[macro_export(local_inner_macros)]
                macro_rules! number { () => { inner!() }; }
            """, encoding="utf-8")
            rlib = folder / "libforeign.rlib"
            subprocess.run(["rustc", "--edition=2021", "--crate-type", "rlib", declarative, "-o", rlib],
                           capture_output=True, timeout=30, check=True)
            procedural = folder / "spans.rs"
            procedural.write_text("""
                extern crate proc_macro;
                use proc_macro::{Span, TokenStream};
                #[proc_macro]
                pub fn at(input: TokenStream) -> TokenStream {
                    let invocation = Span::call_site();
                    let arg = input.into_iter().next().unwrap().span();
                    format!("({:?}, {}, {}, {}, {}, {:?})", invocation.file(), invocation.line(),
                            invocation.column(), arg.line(), arg.column(), arg.source_text().unwrap())
                        .parse().unwrap()
                }
            """, encoding="utf-8")
            subprocess.run(["rustc", "--edition=2021", "--crate-type", "proc-macro",
                            "--out-dir", folder, procedural], capture_output=True, timeout=30, check=True)
            dylib = next(folder.glob("libspans.*"))
            source = folder / "main.rs"
            source.write_text(
                'use spans::at; fn first() { println!("{:?}", at!(( 3 /* keep this text */ + 2 ))); }\n'
                'use foreign::{combine, number};\n'
                'macro_rules! inner { () => { 21 }; }\n'
                'fn main() {\n'
                '    let hidden = 5;\n'
                '    first();\n'
                '    println!("{} {} {} {}", combine!(hidden), number!(), inner!(), module_path!());\n'
                '}\n', encoding="utf-8",
            )
            binary = folder / "native"
            external = (f"foreign={rlib}", f"spans={dylib}")
            flags = [part for item in external for part in ("--extern", item)]
            subprocess.run(["rustc", "--edition=2021", *flags, source, "-o", binary],
                           capture_output=True, timeout=30, check=True)
            native = subprocess.run([binary], capture_output=True, text=True, timeout=10, check=True)
            flags = [part for item in external for part in ("--macro-extern", item)]
            runner = Path(__file__).with_name("rust_in_python.py")
            python = subprocess.run([sys.executable, runner, *flags, source],
                                    capture_output=True, text=True, timeout=30, check=False)
            self.assertEqual(python.returncode, 0, python.stderr)
            self.assertEqual(python.stdout, native.stdout)

    @unittest.skipUnless(shutil.which("rustc"), "rustc is optional")
    def test_procedural_macro_host(self):
        with TemporaryDirectory() as folder:
            folder = Path(folder)
            helper = folder / "helper.rs"
            helper.write_text("""
                extern crate proc_macro;
                use proc_macro::TokenStream;
                #[proc_macro]
                pub fn answer(_: TokenStream) -> TokenStream { "6 * 7".parse().unwrap() }
                #[proc_macro]
                pub fn shifted(_: TokenStream) -> TokenStream { "8u8 >> 1".parse().unwrap() }
                #[proc_macro_attribute]
                pub fn add(_: TokenStream, item: TokenStream) -> TokenStream {
                    let mut result = item;
                    result.extend("fn from_attribute() -> i32 { 5 }".parse::<TokenStream>().unwrap());
                    result
                }
                #[proc_macro_derive(Marker)]
                pub fn marker(_: TokenStream) -> TokenStream {
                    "fn from_derive() -> i32 { 7 }".parse().unwrap()
                }
            """, encoding="utf-8")
            subprocess.run(["rustc", "--crate-type", "proc-macro", "--crate-name", "helper",
                            "--out-dir", folder, helper], check=True, capture_output=True, timeout=30)
            crate = next(path for path in folder.iterdir() if path != helper)
            external = f"helper={crate}"
            (folder / "nested.rs").write_text("""
                use helper::shifted;
                pub fn value() -> u8 { shifted!() }
            """, encoding="utf-8")
            source = """
                use helper::{answer, add, Marker};
                mod nested;
                #[derive(Marker, Debug)]
                struct Number(i32);
                #[add]
                fn main() {
                    let mask = 2u8 << 3;
                    let repeated = vec![nested::value(); 2];
                    let list = vec![1, answer!()];
                    let name = stringify!(nested::value());
                    let formatted = format!("{mask} {repeated:?} {list:?} {name}");
                    print!("{formatted}");
                    eprint!("proc");
                    eprintln!(" macros");
                    println!(" {} {} {} {:?}", answer!(), from_attribute(), from_derive(), Number(3));
                }
            """
            input_file = folder / "main.rs"
            binary = folder / "binary"
            input_file.write_text(source, encoding="utf-8")
            subprocess.run(["rustc", "--edition=2021", "--extern", external, input_file, "-o", binary],
                           check=True, capture_output=True, timeout=30)
            rust = subprocess.run([binary], capture_output=True, text=True, timeout=10, check=True)
            output = StringIO()
            errors = StringIO()
            with redirect_stderr(errors):
                run(source, output, proc_extern=(external,), source_path=input_file)
            self.assertEqual(output.getvalue(), rust.stdout)
            self.assertEqual(errors.getvalue(), rust.stderr)


if __name__ == "__main__":
    unittest.main()
