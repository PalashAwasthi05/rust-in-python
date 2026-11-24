// cfg attributes and cfg! inside function bodies, struct fields, and match arms.
struct Config {
    name: &'static str,
    #[cfg(test)]
    test_only: u32,
    #[cfg(not(test))]
    level: u8,
}

#[cfg(debug_assertions)]
fn mode() -> &'static str {
    "debug"
}

#[cfg(not(debug_assertions))]
fn mode() -> &'static str {
    "release"
}

#[cfg(all(unix, not(windows)))]
fn platform() -> &'static str {
    "unix-like"
}

#[cfg(windows)]
fn platform() -> &'static str {
    "windows"
}

#[cfg(any(feature = "fancy", test))]
fn fancy() -> u32 {
    this_does_not_exist()
}

fn main() {
    let c = Config { name: "app", level: 2 };
    println!("{} {}", c.name, c.level);
    println!("{} {}", mode(), platform());
    #[cfg(test)]
    println!("never");
    #[cfg(not(test))]
    println!("always");
    #[cfg(feature = "nope")]
    let x = 1;
    #[cfg(not(feature = "nope"))]
    let x = 2;
    println!("x = {}", x);
    if cfg!(debug_assertions) {
        println!("debug assertions on");
    }
    println!("{}", cfg!(test));
    #[cfg(test)]
    {
        println!("hidden block");
    }
    let v = match x {
        #[cfg(test)]
        2 => "test two",
        2 => "two",
        _ => "other",
    };
    println!("{}", v);
}
