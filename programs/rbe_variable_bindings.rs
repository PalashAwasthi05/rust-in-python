fn main() {
    // immutable by default, `mut` when we need to change it
    let an_integer = 1u32;
    let a_boolean = true;
    let unit = ();
    let copied_integer = an_integer;
    println!("integer {:?} boolean {:?} unit {:?} copy {}", an_integer, a_boolean, unit, copied_integer);
    let _unused_variable = 3u32;

    let mut mutable_binding = 1;
    println!("before mutation: {}", mutable_binding);
    mutable_binding += 1;
    mutable_binding *= 10;
    println!("after mutation: {}", mutable_binding);

    // scope
    let long_lived_binding = 1;
    {
        let short_lived_binding = 2;
        println!("inner short: {}", short_lived_binding);
        println!("inner can see outer: {}", long_lived_binding);
    }
    println!("outer long: {}", long_lived_binding);

    // shadowing
    let shadowed_binding = 1;
    {
        println!("before being shadowed: {}", shadowed_binding);
        let shadowed_binding = "abc";
        println!("shadowed in inner block: {}", shadowed_binding);
    }
    println!("outside inner block: {}", shadowed_binding);
    let shadowed_binding = 2;
    println!("shadowed in outer block: {}", shadowed_binding);
    let spaces = "   ";
    let spaces = spaces.len();
    println!("spaces is now a number: {}", spaces);

    // declare first, initialize later
    let a_binding;
    {
        let x = 2;
        a_binding = x * x;
    }
    println!("a binding: {}", a_binding);
    let another_binding;
    another_binding = if a_binding > 3 { "big" } else { "small" };
    println!("another binding: {}", another_binding);

    let category: &str;
    let score = 73;
    if score >= 90 {
        category = "A";
    } else if score >= 70 {
        category = "C";
    } else {
        category = "F";
    }
    println!("score {} -> {}", score, category);

    // freezing: shadowing a mutable binding with an immutable one
    let mut _mutable_integer = 7i32;
    {
        let _mutable_integer = _mutable_integer;
        println!("frozen copy: {}", _mutable_integer);
    }
    _mutable_integer = 3;
    println!("unfrozen again: {}", _mutable_integer);

    // destructuring bindings
    let (mut left, right) = (10, 20);
    left += right;
    let [first, second, ..] = [1, 2, 3, 4];
    let mut total = 0;
    for value in [first, second, left, right] {
        total += value;
    }
    println!("left {} right {} first {} second {} total {}", left, right, first, second, total);
    let mut v = Vec::new();
    v.push(1);
    let v = v;
    println!("vec frozen by shadowing: {:?}", v);
}
