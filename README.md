# Rust in Python

I heard Rust had a steep learning curve, so I put Python underneath it.

This is a Rust interpreter written entirely in Python. It tokenizes, parses, type-checks, and runs a subset of Rust using only the Python standard library. Python is now checking my borrowing habits. This seemed funnier before it started saying no.

## Run it

Use Python 3.10 or newer.

```sh
python3 rust_in_python.py --demo
python3 rust_in_python.py hello.rs
```

Save this as `hello.rs` to make the second command earn its keep.

```rust
fn main() {
    let mut total = 0;
    for n in 1..=5 {
        total += n;
    }
    println!("I borrowed {total} integers.");
}
```

It prints `I borrowed 15 integers.` The loan was unsecured.

## What runs

- Functions and recursion
- Typed bindings, mutability, and block expressions
- Conditionals, labeled loops and blocks, integer ranges, and value-producing `break`
- Named, tuple, and unit structs, plus enums with payloads
- Field access and mutation, destructuring, `if let`, `while let`, and `let-else`
- Exhaustive `match` with nested patterns, alternatives, integer ranges, and guards
- Move and initialization checks across branches and loop iterations
- Partial moves, assignment after a move, and selected `Clone`, `Copy`, and `Debug` derives
- Shared and mutable references to locals, fields, and temporaries
- Borrow checks through last use, including aliases, reborrows, and function arguments
- Reference returns, lifetime elision, named lifetime bounds, and immutable `'static` references
- Generic functions, structs, and enums with inferred and explicit type arguments
- Built-in trait bounds for copying, cloning, formatting, comparing, and constructing defaults
- User-defined traits, supertraits, inherent and trait `impl` blocks, default methods, and associated items
- Const generic parameters and fixed-size arrays with length inference, indexing, and repeat literals
- Declarative `macro_rules!` with multiple arms, fragment captures, and repetitions
- Checked `i32` arithmetic, `bool`, string literals, and `()`
- `println!` with display, debug, and named placeholders

## Match something

```rust
struct Point { x: i32, y: i32 }
enum Delivery { Arrived(Point), InTransit }

fn main() {
    let parcel = Delivery::Arrived(Point { x: 3, y: 4 });
    match parcel {
        Delivery::Arrived(Point { x, y }) => println!("Left at ({x}, {y})"),
        Delivery::InTransit => println!("Probably on a truck"),
    }
}
```

## Check it

```sh
python3 -m unittest -v
```

The interpreter runs with Python alone. With `rustc` installed, the tests also check the same programs against Rust.

## Move something

```rust
struct Parcel(i32);
struct Shipment { first: Parcel, second: Parcel }

fn main() {
    let mut shipment = Shipment { first: Parcel(1), second: Parcel(2) };
    let taken = shipment.first;
    println!("Moved {}. Still have {}.", taken.0, shipment.second.0);
    shipment.first = Parcel(3);
    let rebuilt = shipment;
    println!("Rebuilt with {}.", rebuilt.first.0);
}
```

## Borrow something

```rust
fn add(value: &mut i32, delta: i32) {
    *value += delta;
}

fn main() {
    let mut balance = 3;
    let borrowed = &mut balance;
    add(borrowed, *borrowed);
    println!("{balance}");
}
```

The implicit mutable reborrow activates when `add` is called, so the second argument can read the old balance. The program prints `6`. No interest was charged.

## Return something borrowed

```rust
struct Pair { left: i32, right: i32 }

fn left(pair: &mut Pair) -> &mut i32 {
    &mut pair.left
}

fn main() {
    let mut pair = Pair { left: 3, right: 5 };
    let borrowed = left(&mut pair);
    *borrowed += 1;
    println!("{}", pair.left);
}
```

The returned reference keeps the argument borrowed until its last use. The program prints `4`.

## Generalize something

```rust
use std::fmt::Debug;

#[derive(Default)]
enum Maybe<T> {
    Present(T),
    #[default]
    Absent,
}

fn inspect<T: Debug>(item: Maybe<T>) {
    match item {
        Maybe::Present(value) => println!("Found {value:?}"),
        Maybe::Absent => println!("Still looking"),
    }
}

fn main() {
    inspect(Maybe::Present(7));
    inspect(Maybe::<bool>::default());
}
```

The first call infers the type argument from `7`. The second specifies `bool` explicitly. The program prints `Found 7` and `Still looking`.

## Count something

```rust
trait Count {
    const LENGTH: usize;
    fn count(&self) -> usize { Self::LENGTH }
}

struct Batch<T, const N: usize>([T; N]);

impl<T, const N: usize> Count for Batch<T, N> {
    const LENGTH: usize = N;
}

macro_rules! batch {
    ($($item:expr),+ $(,)?) => { Batch([$($item),+]) };
}

fn main() {
    let batch = batch!(10, 20, 30,);
    println!("{}", batch.count());
}
```

The array length infers the const argument `N`. The trait's default method reads the implementation's associated constant and prints `3`.

## Scope

This is a tree-walking interpreter for a Rust subset. The move checker tracks record and tuple-struct fields separately and treats enum payloads as a single storage location.

Function signatures can tie reference returns to input lifetimes. At calls the borrow checker uses those declared input regions, including bounds between lifetimes. Reference reassignment, reference fields, reference binding patterns, nested references, and lifetime support for string-slice signatures remain incomplete.

Generic bodies are checked against their declared trait bounds. Const parameters support `i32`, 64-bit `usize`, and `bool`. Type-level expressions cover standalone parameters and basic constant expressions. Trait support covers static dispatch, methods, and non-reference associated types. Declarative macros support selected fragments and local-name hygiene. Trait objects, specialization, generic associated types, full macro hygiene, and procedural macros remain open.

General reference types as type arguments, modules, the standard library, Cargo integration, and native code generation are also incomplete.
