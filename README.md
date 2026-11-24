# Rust in Python

I heard Rust had a steep learning curve, so I put Python underneath it.

This is a Rust interpreter written entirely in Python. It tokenizes, parses, type-checks, and runs a subset of Rust using only the Python standard library. The borrow checker saw the repo name and sent the move checker ahead.

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

## Scope

This is a tree-walking interpreter for a Rust subset. The move checker tracks record and tuple-struct fields separately and treats enum payloads as a single storage location. The remaining work includes general reference expressions, borrow checking, lifetime inference, generics, user-defined traits, modules, general macros, the standard library, Cargo integration, and native code generation.
