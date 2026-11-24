use std::marker::PhantomData;
use std::ops::Add;

#[derive(PartialEq, Debug)]
struct PhantomTuple<A, B>(A, PhantomData<B>);

#[derive(PartialEq, Debug)]
struct PhantomStruct<A, B> { first: A, phantom: PhantomData<B> }

#[derive(Debug, Clone, Copy)]
enum Inch {}
#[derive(Debug, Clone, Copy)]
enum Mm {}

#[derive(Debug, Clone, Copy)]
struct Length<Unit>(f64, PhantomData<Unit>);

impl<Unit> Add for Length<Unit> {
    type Output = Length<Unit>;
    fn add(self, rhs: Length<Unit>) -> Length<Unit> { Length(self.0 + rhs.0, PhantomData) }
}

impl Length<Inch> {
    fn to_mm(self) -> Length<Mm> { Length(self.0 * 25.4, PhantomData) }
}

// typestate: a request can only be sent once its URL has been set
struct Empty;
struct Ready;

struct Request<State> { url: String, headers: Vec<String>, _state: PhantomData<State> }

impl Request<Empty> {
    fn new() -> Self { Request { url: String::new(), headers: vec![], _state: PhantomData } }
    fn url(self, url: &str) -> Request<Ready> {
        Request { url: url.to_string(), headers: self.headers, _state: PhantomData }
    }
}

impl<S> Request<S> {
    fn header(mut self, h: &str) -> Self { self.headers.push(h.to_string()); self }
}

impl Request<Ready> {
    fn send(&self) -> String { format!("GET {} with {} header(s)", self.url, self.headers.len()) }
}

struct Id<T> { value: u32, _marker: PhantomData<T> }
impl<T> Id<T> { fn new(value: u32) -> Self { Id { value, _marker: PhantomData } } }
struct User;
struct Order;

fn lookup_user(id: &Id<User>) -> String { format!("user #{}", id.value) }
fn lookup_order(id: &Id<Order>) -> String { format!("order #{}", id.value) }

fn main() {
    let _tuple1: PhantomTuple<char, f32> = PhantomTuple('Q', PhantomData);
    let _tuple2: PhantomTuple<char, f64> = PhantomTuple('Q', PhantomData);
    let struct1: PhantomStruct<char, f32> = PhantomStruct { first: 'Q', phantom: PhantomData };
    let struct2: PhantomStruct<char, f32> = PhantomStruct { first: 'Q', phantom: PhantomData };
    println!("same phantom types compare: {}", struct1 == struct2);
    println!("debug: {:?}", _tuple1);
    println!("size of PhantomData: {}, of PhantomTuple<char, f64>: {}", std::mem::size_of::<PhantomData<u64>>(), std::mem::size_of::<PhantomTuple<char, f64>>());

    let one_foot: Length<Inch> = Length(12.0, PhantomData);
    let one_meter: Length<Mm> = Length(1000.0, PhantomData);
    let two_feet = one_foot + one_foot;
    let two_meters = one_meter + one_meter;
    println!("one foot + one foot = {:?} in", two_feet.0);
    println!("one meter + one meter = {:?} mm", two_meters.0);
    let converted = two_feet.to_mm() + one_meter;
    println!("two feet in mm plus a meter = {:.1} mm", converted.0);

    let req = Request::new().header("Accept: */*").url("/status").header("X-Trace: 1");
    println!("{}", req.send());

    let u: Id<User> = Id::new(7);
    let o: Id<Order> = Id::new(7);
    println!("{} / {}", lookup_user(&u), lookup_order(&o));
}
