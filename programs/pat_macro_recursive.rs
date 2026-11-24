// Recursive macros: counting, tt-munchers, nested repetitions, hygiene.
macro_rules! count {
    () => { 0usize };
    ($head:tt $($tail:tt)*) => { 1usize + count!($($tail)*) };
}

macro_rules! reverse_list {
    ([] $($rev:expr),*) => { vec![$($rev),*] };
    ([$first:expr $(, $rest:expr)*] $($rev:expr),*) => {
        reverse_list!([$($rest),*] $first $(, $rev)*)
    };
}

// A tiny calculator DSL parsed by a tt-muncher: `calc!(push 3 push 4 add push 2 mul)`.
// The stack is kept top-first.
macro_rules! calc {
    (@run [$($stack:expr),*]) => { [$($stack),*] };
    (@run [$($stack:expr),*] push $n:literal $($rest:tt)*) => {
        calc!(@run [$n $(, $stack)*] $($rest)*)
    };
    (@run [$b:expr, $a:expr $(, $stack:expr)*] add $($rest:tt)*) => {
        calc!(@run [$a + $b $(, $stack)*] $($rest)*)
    };
    (@run [$b:expr, $a:expr $(, $stack:expr)*] mul $($rest:tt)*) => {
        calc!(@run [($a) * ($b) $(, $stack)*] $($rest)*)
    };
    ($($tokens:tt)*) => { calc!(@run [] $($tokens)*) };
}

macro_rules! matrix {
    ($([$($x:expr),*]),* $(,)?) => {
        vec![$(vec![$($x),*]),*]
    };
}

macro_rules! table {
    ($($row:ident: $($cell:expr),+;)+) => {{
        let mut lines = Vec::new();
        $(
            let cells: Vec<String> = vec![$($cell.to_string()),+];
            lines.push(format!("{:>6} | {}", stringify!($row), cells.join(", ")));
        )+
        lines
    }};
}

macro_rules! swap_hygiene {
    ($a:ident, $b:ident) => {
        let tmp = $a;
        $a = $b;
        $b = tmp;
    };
}

macro_rules! make_counter_fn {
    ($fname:ident, $($step:expr),+) => {
        fn $fname() -> Vec<i32> {
            let mut total = 0;
            let mut seen = Vec::new();
            $(
                total += $step;
                seen.push(total);
            )+
            seen
        }
    };
}

make_counter_fn!(running, 1, 2, 3, 4);

fn main() {
    println!("{}", count!());
    println!("{}", count!(a b c));
    println!("{}", count!(x (y z) [1 2 3] {} ,));
    const N: usize = count!(1 2 3 4 5);
    let arr = [0i32; N];
    println!("arr len {}", arr.len());
    println!("{:?}", reverse_list!([1, 2, 3, 4]));
    println!("{:?}", calc!(push 3 push 4 add push 2 mul));
    println!("{:?}", calc!(push 1 push 2 push 3 add));
    let m = matrix![[1, 2, 3], [4, 5, 6]];
    for row in &m {
        println!("{:?}", row);
    }
    for line in table! {
        alpha: 1, 2, 3;
        beta: "x", 'y';
        gamma: 2.5;
    } {
        println!("{}", line);
    }
    let tmp = 100;
    let mut x = 1;
    let mut y = 2;
    swap_hygiene!(x, y);
    println!("x={} y={} tmp={}", x, y, tmp);
    println!("{:?}", running());
}
