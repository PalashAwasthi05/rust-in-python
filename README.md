# Rust in Python

I heard Rust had a steep learning curve, so I put Python underneath it.

This is a small Rust interpreter written entirely in Python. It tokenizes, parses, type-checks, and runs a subset of Rust using only the Python standard library. The borrow checker saw the repo name and turned around.

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
- Conditionals, loops, integer ranges, and early returns
- Checked `i32` arithmetic, `bool`, string literals, and `()`
- `println!` with display, debug, and named placeholders

The scope is small. Ownership, borrow checking, traits, Cargo, and the Rust standard library are outside it.
