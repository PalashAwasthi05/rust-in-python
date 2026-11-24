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
- Tuples with positional fields, annotations, destructuring, and partial moves
- Field access and mutation, destructuring, `if let`, `while let`, and `let-else`
- Exhaustive `match` with nested patterns, alternatives, integer ranges, and guards
- Shared and mutable pattern bindings, explicit `ref` and `ref mut`, and reference patterns
- `@` bindings on ranges and structural subpatterns
- Move and initialization checks across branches and loop iterations
- Partial moves, assignment after a move, and selected `Clone`, `Copy`, and `Debug` derives
- Shared and mutable references to locals, fields, and temporaries
- Borrow checks through last use, including aliases, reborrows, and function arguments
- Reference returns, lifetime elision, named lifetime bounds, and immutable `'static` references
- Generic functions, structs, and enums with inferred and explicit type arguments
- Type aliases in type positions, including lifetime, type, and const parameters
- Built-in trait bounds for copying, cloning, formatting, comparing, and constructing defaults
- User-defined traits, supertraits, inherent and trait `impl` blocks, default methods, and associated items
- Const generic parameters and fixed-size arrays with length inference, indexing, repeat literals, and patterns
- Borrowed slices, array-to-slice coercion, range indexing, and patterns on dereferenced slices
- `Vec<T>`, `vec!`, indexing, slice views, and basic vector mutation
- Destructuring `for` loops over ranges, arrays, vectors, borrowed slices, `Option`, and `Result`
- Collection iterators with `iter`, `iter_mut`, `into_iter`, `rev`, and lazy `map` with closures
- Iterator `collect` into `Vec` and `String`, plus `count` and checked integer `sum`
- Declarative `macro_rules!` with multiple arms, fragment captures, and repetitions
- Borrowed `dyn` objects with dynamic dispatch, associated-type bindings, and supertrait upcasts
- Generic associated types and const expressions evaluated at specialization
- Definition-site macro bindings and labels, plus a rustc host for compiled procedural macros
- Inline and file-backed modules, grouped and glob imports, re-exports, and module visibility
- Prelude `Option` and `Result`, `?` propagation, and selected assertion and matching macros
- Function pointers, indirect calls, and closures with shared, mutable, and explicit move captures, including references
- Signed and unsigned integer widths, casts and ranges, Unicode `char`, raw strings, byte strings, `bool`, and `()`
- Owned `String` values with concatenation and UTF-8-aware lengths and mutation
- Borrowed strings with lifetime checking, UTF-8 substring views, and `chars` and `bytes` iterators
- Bitwise operators, checked shift counts, and integer casts in const expressions
- `f32` and `f64` arithmetic, comparisons, casts, and selected float methods
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

## Pass a trait object

```rust
trait Measure {
    fn read(&self) -> i32;
}

struct Meter(i32);
impl Measure for Meter {
    fn read(&self) -> i32 { self.0 }
}

fn report(value: &dyn Measure) {
    println!("{}", value.read());
}

fn main() {
    let meter = Meter(7);
    report(&meter);
}
```

## Load procedural macros

The optional host uses the installed Rust compiler to expand function-like, attribute, and derive macros from compiled Rust crates. Pass the name and path of each crate.

```sh
python3 rust_in_python.py --proc-extern demo_macros=./libdemo_macros.so program.rs
```

On macOS, use the compiled `.dylib` path. The Python interpreter checks and runs the expanded program. The host exchanges expanded source text, so compiler span metadata is lost at that boundary.

## Compare a crate against rustc

```sh
python3 rust_in_python.py --check src/main.rs
python3 conformance.py --json src/main.rs examples/another_root.rs
```

Pass each crate's root source file. The interpreter loads sibling modules from their `.rs` files or `mod.rs` directories. The conformance runner records the reference compiler version and checks acceptance before comparing standard output and exit status. It exits unsuccessfully on a mismatch, crash, or timeout. Exact diagnostic text is outside the comparison.

## Scope

This is a tree-walking interpreter for a Rust subset. The move checker tracks record and tuple-struct fields separately and treats enum payloads as a single storage location.

Function signatures can tie reference returns to input lifetimes. At calls the borrow checker uses those declared input regions, including bounds between lifetimes. Reference reassignment, reference fields, reference binding patterns, nested references, and lifetime support for string-slice signatures remain incomplete.

Generic bodies are checked against their declared trait bounds. Const parameters support `i32`, 64-bit `usize`, and `bool`. Dependent const expressions cover checked arithmetic and comparisons with structural matching, plus Rust's unit-array bound syntax.

Trait objects currently use shared or mutable references. Generic associated types support type, const, and lifetime parameters, including lending projections on concrete receivers. Trait specialization and general borrow analysis for abstract lending projections remain open.

Declarative macros preserve definition-site local bindings and labels, caller-supplied tokens, and fragment opacity. Textual macro inheritance, `#[macro_export]`, crate-root paths, and macro re-exports work within a crate. Crate dependencies and cross-crate macro hygiene remain open.

General reference types as type arguments, complete closure capture analysis and callable trait bounds, standard library coverage, Cargo integration, and native code generation remain incomplete. `Result` propagation currently requires the same error type at the call and return sites.
