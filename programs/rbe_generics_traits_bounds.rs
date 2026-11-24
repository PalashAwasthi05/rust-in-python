use std::fmt::{Debug, Display};

struct Val { val: f64 }
struct GenVal<T> { gen_val: T }

impl Val {
    fn value(&self) -> &f64 { &self.val }
}

impl<T> GenVal<T> {
    fn value(&self) -> &T { &self.gen_val }
}

impl GenVal<f64> {
    fn rounded(&self) -> i64 { self.gen_val.round() as i64 }
}

trait HasArea { fn area(&self) -> f64; }

#[derive(Debug)]
struct Rectangle { length: f64, height: f64 }
#[derive(Debug)]
struct Triangle { base: f64, height: f64 }

impl HasArea for Rectangle { fn area(&self) -> f64 { self.length * self.height } }
impl HasArea for Triangle { fn area(&self) -> f64 { 0.5 * self.base * self.height } }

fn print_debug<T: Debug>(t: &T) { println!("{:?}", t); }

fn area<T: HasArea>(t: &T) -> f64 { t.area() }

fn compare_prints<T: Debug + Display>(t: &T) {
    println!("Debug: `{:?}`", t);
    println!("Display: `{}`", t);
}

fn compare_types<T: Debug, U: Debug>(t: &T, u: &U) {
    println!("t: `{:?}`", t);
    println!("u: `{:?}`", u);
}

trait PrintInOption { fn print_in_option(self); }

impl<T> PrintInOption for T where Option<T>: Debug {
    fn print_in_option(self) { println!("{:?}", Some(self)); }
}

fn total_area<I, S>(shapes: I) -> f64
where
    I: IntoIterator<Item = S>,
    S: HasArea + Debug,
{
    shapes.into_iter().map(|s| s.area()).sum()
}

fn largest_by_area<'a, T>(items: &'a [T]) -> Option<&'a T>
where
    T: HasArea,
{
    items.iter().max_by(|a, b| a.area().partial_cmp(&b.area()).unwrap())
}

struct Cardinal;
struct BlueJay;
trait Red {}
trait Blue {}
impl Red for Cardinal {}
impl Blue for BlueJay {}
fn red<T: Red>(_: &T) -> &'static str { "red" }
fn blue<T: Blue>(_: &T) -> &'static str { "blue" }

fn main() {
    let x = Val { val: 3.0 };
    let y = GenVal { gen_val: 3i32 };
    let z = GenVal { gen_val: 2.6f64 };
    println!("{}, {}, {} rounded {}", x.value(), y.value(), z.value(), z.rounded());

    let rectangle = Rectangle { length: 3.0, height: 4.0 };
    let triangle = Triangle { base: 3.0, height: 4.0 };
    print_debug(&rectangle);
    println!("rectangle area: {}, triangle area: {}", area(&rectangle), area(&triangle));
    println!("a cardinal is {}, a blue jay is {}", red(&Cardinal), blue(&BlueJay));

    let string = "words";
    let array = [1, 2, 3];
    let vec = vec![1, 2, 3];
    compare_prints(&string);
    compare_types(&array, &vec);

    let vec = vec![1, 2, 3];
    vec.print_in_option();
    "text".print_in_option();

    let rects = vec![Rectangle { length: 1.0, height: 2.0 }, Rectangle { length: 3.0, height: 3.0 }, Rectangle { length: 0.5, height: 8.0 }];
    println!("largest {:?}", largest_by_area(&rects));
    println!("total area {}", total_area(rects));
    println!("triangles total {}", total_area(vec![Triangle { base: 2.0, height: 2.0 }, triangle]));
}
