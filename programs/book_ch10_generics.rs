// Chapter 10.1: generic functions, structs, enums, and methods.
fn largest_i32(list: &[i32]) -> &i32 {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

fn largest_copy<T: PartialOrd + Copy>(list: &[T]) -> T {
    let mut largest = list[0];
    for &item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

#[derive(Debug)]
struct Point<T> {
    x: T,
    y: T,
}

impl<T> Point<T> {
    fn x(&self) -> &T {
        &self.x
    }
}

impl Point<f32> {
    fn distance_from_origin(&self) -> f32 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}

#[derive(Debug, Clone, Copy)]
struct Mixed<X1, Y1> {
    x: X1,
    y: Y1,
}

impl<X1, Y1> Mixed<X1, Y1> {
    fn mixup<X2, Y2>(self, other: Mixed<X2, Y2>) -> Mixed<X1, Y2> {
        Mixed {
            x: self.x,
            y: other.y,
        }
    }
}

#[derive(Debug)]
enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L: std::fmt::Debug, R: std::fmt::Debug> Either<L, R> {
    fn describe(&self) -> String {
        match self {
            Either::Left(l) => format!("left {:?}", l),
            Either::Right(r) => format!("right {:?}", r),
        }
    }
}

struct Stack<T> {
    items: Vec<T>,
}

impl<T: Clone + std::fmt::Display> Stack<T> {
    fn new() -> Self {
        Stack { items: Vec::new() }
    }
    fn push(&mut self, item: T) -> &mut Self {
        self.items.push(item);
        self
    }
    fn peek(&self) -> Option<&T> {
        self.items.last()
    }
    fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }
    fn render(&self) -> String {
        let parts: Vec<String> = self.items.iter().map(|i| i.to_string()).collect();
        format!("[{}]", parts.join(" | "))
    }
}

fn pair_up<A: Clone, B: Clone>(a: &A, b: &B) -> (A, B) {
    (a.clone(), b.clone())
}

fn count_matching<T: PartialEq>(items: &[T], needle: &T) -> usize {
    items.iter().filter(|item| *item == needle).count()
}

fn main() {
    let number_list = vec![34, 50, 25, 100, 65];
    println!("The largest number is {}", largest_i32(&number_list));
    let number_list = vec![102, 34, 6000, 89, 54, 2, 43, 8];
    println!("The largest number is {}", largest(&number_list));
    let char_list = vec!['y', 'm', 'a', 'q'];
    println!("The largest char is {}", largest(&char_list));
    let float_list = [1.5, -2.0, 9.25, 3.0];
    println!("The largest float is {}", largest_copy(&float_list));
    let words = vec![String::from("pear"), String::from("zucchini"), String::from("apple")];
    println!("The largest word is {}", largest(&words));
    let pairs = [(1, 'b'), (3, 'a'), (3, 'c'), (2, 'z')];
    println!("The largest pair is {:?}", largest(&pairs));

    let integer = Point { x: 5, y: 10 };
    let float = Point { x: 1.0, y: 4.0 };
    println!("{:?} {:?}", integer, float);
    println!("p.x = {}", integer.x());
    let p: Point<f32> = Point { x: 3.0, y: 4.0 };
    println!("distance = {}", p.distance_from_origin());

    let p1 = Mixed { x: 5, y: 10.4 };
    let p2 = Mixed { x: "Hello", y: 'c' };
    let p3 = p1.mixup(p2);
    println!("p3.x = {}, p3.y = {}", p3.x, p3.y);
    println!("{:?}", p2.mixup(p1));

    let values: Vec<Either<i32, &str>> = vec![Either::Left(1), Either::Right("two"), Either::Left(3)];
    for v in &values {
        println!("{}", v.describe());
    }
    let lefts: i32 = values
        .iter()
        .map(|v| match v {
            Either::Left(n) => *n,
            Either::Right(_) => 0,
        })
        .sum();
    println!("sum of lefts = {lefts}");

    let some: Option<i32> = Some(5);
    let ok: Result<f64, String> = Ok(2.5);
    let err: Result<f64, String> = Err("bad".to_string());
    println!("{:?} {:?} {:?}", some, ok, err);

    let mut stack: Stack<String> = Stack::new();
    stack.push("a".to_string()).push("b".to_string()).push("c".to_string());
    println!("{} peek {:?}", stack.render(), stack.peek());
    let top = stack.pop();
    println!("popped {:?} now {}", top, stack.render());
    let mut nums = Stack::new();
    nums.push(1.5).push(2.25);
    println!("{}", nums.render());

    let (a, b) = pair_up(&String::from("left"), &vec![1, 2]);
    println!("{a} {:?}", b);
    println!("{} {}", count_matching(&[1, 2, 1, 3, 1], &1), count_matching(&["a", "b"], &"c"));
}
