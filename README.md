# Rust in Python

I heard Rust had a steep learning curve, so I put Python underneath it.

This is a Rust interpreter written entirely in Python. It tokenizes, parses, type-checks, borrow-checks, and runs Rust using only the Python standard library. It also ships a standard library, mostly written in Rust and running on the interpreter, with Python underneath for the parts that touch the machine. Python is now checking my borrowing habits. This seemed funnier before it started saying no.

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

- Functions and recursion, including destructured function and closure parameters
- Typed bindings, deferred initialization in `let` patterns, mutability, and block expressions
- Conditionals, labeled loops and blocks, integer ranges, and value-producing `break`
- Named, tuple, and unit structs, plus enums with payloads
- Struct update syntax with partial moves from the base value
- Tuples with positional fields, annotations, destructuring, and partial moves
- Destructuring assignment for tuples, arrays, and structs, including ignored fields and initialization of existing bindings
- Field access and mutation, destructuring, `if let`, `while let`, `let-else`, and Rust 2024 let chains
- Exhaustive `match` with nested patterns, alternatives, integer ranges, and guards
- Shared and mutable pattern bindings, explicit `ref` and `ref mut`, and reference patterns
- `@` bindings on ranges and structural subpatterns
- Move and initialization checks across branches and loop iterations
- Partial moves, assignment after a move, and selected `Clone`, `Copy`, and `Debug` derives
- Shared and mutable references to locals, fields, and temporaries
- Borrow checks through last use, including aliases, reborrows, and function arguments
- Disjoint field reborrows through references selected by control flow
- Reference returns, lifetime elision, named lifetime bounds, and immutable `'static` references
- Generic functions, structs, and enums with inferred and explicit type arguments, associated-type constraints, and argument-position `impl Trait`
- Opaque owned return types in free functions and inherent methods with `-> impl Trait` and associated-type bindings
- Type aliases in type positions, including lifetime, type, and const parameters
- Built-in trait bounds for copying, cloning, formatting, comparing, and constructing defaults
- User-defined traits, supertraits, inherent and trait `impl` blocks, default methods, and associated items
- Const generic parameters and fixed-size arrays with length inference, indexing, repeat literals, and patterns
- Scalar `const fn` calls, inline const blocks, and conditional constant expressions
- Immutable scalar `static` items with shared borrows and macro-expanded initializers
- Borrowed slices, array-to-slice coercion, range indexing, and patterns on dereferenced slices
- `Vec<T>`, `vec!`, indexing, slice views, and basic vector mutation
- Destructuring `for` loops over ranges, arrays, vectors, borrowed slices, `Option`, and `Result`
- Collection iterators with `iter`, `iter_mut`, `into_iter`, `rev`, and lazy `map` with closures
- Iterator `collect` into `Vec` and `String`, plus `count` and checked integer `sum`
- Declarative `macro_rules!` with multiple arms, fragment forwarding, repetitions, and matcher follow-set checks
- Built-in `concat!` and `stringify!` with literal text and token spacing
- Source-location macros `file!`, `line!`, `column!`, and `module_path!`, plus `include_str!` and `include_bytes!`
- Shared, mutable, and boxed `dyn` objects with dynamic dispatch, associated-type bindings, supertrait upcasts, and consuming `Box<Self>` receivers
- Generic associated types with specialized and higher-ranked equality constraints
- Const expressions evaluated at specialization
- Definition-site macro bindings and labels, plus a rustc host for compiled procedural macros
- Inline and file-backed modules, grouped and glob imports, re-exports, and module visibility
- Unicode and raw identifiers, NFC normalization, and edition-aware raw lifetimes
- Prelude `Option` and `Result`, `?` propagation, and selected assertion and matching macros
- Program termination with `Result`, `ExitCode`, custom `Termination` implementations, and `process::exit`
- Function pointers, indirect calls, and closures with shared, mutable, and explicit move captures, including references
- Signed and unsigned integer widths, inferred numeric bindings, casts and ranges, Unicode `char`, raw strings, byte strings, `bool`, and `()`
- Owned `String` values with concatenation and UTF-8-aware lengths and mutation
- String searches with character and shared string patterns and UTF-8 byte offsets
- Borrowed strings with lifetime checking, UTF-8 substring views, and `chars` and `bytes` iterators
- Bitwise operators, checked shift counts, and integer casts in const expressions
- `f32` and `f64` arithmetic, comparisons, casts, and selected float methods
- `format!`, printing macros, named and positional arguments, and numeric width and precision
- `Fn`, `FnMut`, and `FnOnce` bounds, `impl Fn` arguments and returns, `Box<dyn Fn>`, and paths like `String::from` or `Some` passed as functions
- Generic traits with defaulted parameters, such as `Add<Rhs = Self>` and `From<T>`, matched on their arguments
- Operator overloading through `std::ops` for arithmetic, bitwise, unary, compound assignment, `Index`, and `IndexMut`
- User `Display`, `Debug`, `Clone`, `Default`, `PartialEq`, `PartialOrd`, `Ord`, `Hash`, and `Drop` impls, used by formatting, cloning, comparison, sorting, and scope exit
- `Deref` auto-deref for method calls, fields, `*`, and indexing
- `From`, `Into`, `TryFrom`, `TryInto`, `FromStr` with `parse`, and `?` that converts errors, including into `Box<dyn Error>`
- Lifetime parameters on structs and impls, and references stored in structs, tuples, vectors, and maps
- Items nested inside function bodies, non-scalar `const` items, `static mut`, and `unsafe` blocks
- Drops in Rust order at scope exit, on reassignment, for discarded temporaries, and through `mem::drop`

## What the standard library has

`std.rs` is the standard library. It is ordinary Rust that the interpreter parses and checks with every program. Functions without bodies are intrinsics implemented in Python.

- `Iterator` and `IntoIterator` as real traits, so user iterators get every adapter: `map`, `filter`, `filter_map`, `flat_map`, `flatten`, `enumerate`, `zip`, `chain`, `skip`, `take`, `step_by`, `peekable`, `scan`, `take_while`, `skip_while`, `inspect`, `rev`, `cycle`, `by_ref`, and the rest
- Terminal iterator methods such as `fold`, `sum`, `product`, `count`, `min`, `max`, `min_by_key`, `position`, `find`, `any`, `all`, `partition`, `unzip`, `try_fold`, and `collect` into vectors, strings, maps, sets, deques, heaps, `Option`, and `Result`
- `Option` and `Result` combinators like `map`, `and_then`, `ok_or`, `unwrap_or_else`, `take`, `as_ref`, and `transpose`
- Slice and `Vec` methods including sorting, binary search, `windows`, `chunks`, `dedup`, `retain`, `drain`, `split_at_mut`, and `join`
- `str`, `String`, and `char` methods with Rust's UTF-8 byte offsets and patterns that can be a char, a string, a char set, or a closure
- Integer and float methods with Rust's overflow, wrapping, saturating, and rounding rules
- `HashMap`, `HashSet`, `BTreeMap`, `BTreeSet`, `VecDeque`, and `BinaryHeap`, with the entry API
- `Rc`, `Weak`, `RefCell`, `Cell`, `Mutex`, `RwLock`, and atomics
- `thread::spawn` and `mpsc` channels, `std::io` for stdin and stdout, `std::env`, and `std::time`

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

The `programs` folder holds Rust programs whose expected output and exit status came from rustc. The tests run each one through the interpreter and compare. To record rustc's output for a new program, add it to the folder and run:

```sh
python3 programs/bless.py programs/new_program.rs
```

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

## Load a source dependency

```sh
python3 rust_in_python.py --extern-source utility=../utility/src/lib.rs program.rs
```

Source dependencies have separate crate roots, module trees, and visibility rules. Import supported types and values with paths such as `utility::Report` and `utility::make`. Source dependency loading and compiled macro expansion currently run in separate modes.

## Load compiled macros

The optional host uses the installed Rust compiler to expand declarative macros from compiled `.rlib` crates and procedural macros from dynamic libraries. Pass the name and path of each crate.

```sh
python3 rust_in_python.py --macro-extern rules=./librules.rlib --macro-extern demo_macros=./libdemo_macros.so program.rs
```

On macOS, use the compiled `.dylib` path. `--proc-extern` is an alias for `--macro-extern`. Rustc supplies source spans to macro code in unmodified file-backed inputs. The host reads syntax contexts from rustc's expanded output to distinguish generated locals from caller locals. The Python checker still uses expanded-source coordinates for its own diagnostics, and user input that requires a rewrite can have different source positions.

## Compare a crate against rustc

```sh
python3 rust_in_python.py --check src/main.rs
python3 conformance.py --json src/main.rs examples/another_root.rs
```

Pass each crate's root source file. The interpreter loads sibling modules from their `.rs` files or `mod.rs` directories. The conformance runner records the reference compiler version and checks acceptance before comparing standard output and exit status. It exits unsuccessfully on a mismatch, crash, or timeout. Exact diagnostic text is outside the comparison.

Both commands accept `--edition`, which defaults to 2021. Edition 2024 enables let chains and adds top-level const blocks and underscores to `expr` macro fragments.

## Scope

This is a tree-walking interpreter, so it is slow. A program that does a few million operations takes a while. Startup parses and checks the whole standard library each run, which costs about a third of a second.

The move checker tracks record and tuple-struct fields separately and treats enum payloads as a single storage location. Function signatures can tie reference returns, including string slices, to input lifetimes, and at calls the borrow checker uses those declared input regions, including bounds between lifetimes.

The borrow checker is strict where it can trace a reference and permissive where it cannot. References stored inside tuples, vectors, structs, and maps, references returned without a traceable input, references produced by closures, and the results of interior mutability like `RefCell::borrow_mut` are not tracked. Programs that rustc accepts run. Some programs that rustc rejects for those reasons also run.

Generic bodies are checked against their declared trait bounds. Const parameters support integer widths, `bool`, and `char`. Dependent const expressions cover checked arithmetic and comparisons with structural matching, plus Rust's unit-array bound syntax. Recursion and mutation in constant evaluation remain open.

Trait objects support shared and mutable references, `Box`, and `Rc`. Generic associated types support type, const, and lifetime parameters. Higher-ranked equalities can tie a projected reference to its receiver's borrow. Trait specialization and inference for unconstrained lending projections remain open.

Declarative macros preserve definition-site local bindings and labels, caller-supplied tokens, and fragment opacity. Textual inheritance, `#[macro_use]` modules, `#[macro_export(local_inner_macros)]`, crate-root paths, and macro re-exports work within a crate. Python diagnostics for source files point to the original file and use the invocation location for generated macro tokens. The optional host supports compiled cross-crate macro expansion and generated local bindings.

Threads run one at a time: `thread::spawn` runs its closure to completion before returning, which is one valid schedule but deadlocks on programs that need two threads to make progress together. `HashMap` and `HashSet` iterate in insertion order, which rustc never promises, so output that prints them unsorted can differ from a compiled run. `RefCell` does not panic on conflicting borrows. File and network IO, processes, `async`, procedural derive macros other than the built-in derives, Cargo integration, and native code generation are not implemented.
