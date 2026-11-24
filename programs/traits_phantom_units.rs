use std::marker::PhantomData;
use std::ops::Add;

#[derive(Debug, Clone, Copy)]
struct Meters;
#[derive(Debug, Clone, Copy)]
struct Seconds;

#[derive(Debug, Clone, Copy)]
struct Quantity<U> {
    value: f64,
    unit: PhantomData<U>,
}

impl<U> Quantity<U> {
    fn new(value: f64) -> Self {
        Quantity { value, unit: PhantomData }
    }
}

impl<U> Add for Quantity<U> {
    type Output = Quantity<U>;
    fn add(self, rhs: Self) -> Self {
        Quantity::new(self.value + rhs.value)
    }
}

trait UnitName {
    fn suffix() -> &'static str;
}
impl UnitName for Meters {
    fn suffix() -> &'static str {
        "m"
    }
}
impl UnitName for Seconds {
    fn suffix() -> &'static str {
        "s"
    }
}

impl<U: UnitName> Quantity<U> {
    fn show(&self) -> String {
        format!("{:.2}{}", self.value, U::suffix())
    }
}

struct Id<T> {
    raw: u32,
    _marker: PhantomData<T>,
}

impl<T> Id<T> {
    fn new(raw: u32) -> Self {
        Id { raw, _marker: PhantomData }
    }
}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

struct User {
    name: String,
}
struct Order {
    total: u32,
}

struct Store<T> {
    items: Vec<(Id<T>, T)>,
    next: u32,
}

impl<T> Store<T> {
    fn new() -> Self {
        Store { items: Vec::new(), next: 1 }
    }
    fn insert(&mut self, item: T) -> Id<T> {
        let id = self.next;
        self.next += 1;
        self.items.push((Id::new(id), item));
        Id::new(id)
    }
    fn get(&self, id: &Id<T>) -> Option<&T> {
        self.items.iter().find(|(i, _)| i == id).map(|(_, t)| t)
    }
}

fn main() {
    let d1: Quantity<Meters> = Quantity::new(3.5);
    let d2 = Quantity::<Meters>::new(1.25);
    let t = Quantity::<Seconds>::new(2.0);
    println!("{}", (d1 + d2).show());
    println!("{}", t.show());
    println!("speed = {:.3} m/s", (d1 + d2).value / t.value);
    println!("{:?}", d1.unit);
    let mut users: Store<User> = Store::new();
    let mut orders: Store<Order> = Store::new();
    let alice = users.insert(User { name: "alice".into() });
    let bob = users.insert(User { name: "bob".into() });
    let o = orders.insert(Order { total: 99 });
    println!("{}", users.get(&alice).unwrap().name);
    println!("{}", users.get(&bob).map(|u| u.name.as_str()).unwrap_or("?"));
    println!("{}", orders.get(&o).unwrap().total);
    println!("{}", users.get(&Id::new(7)).is_none());
    println!("{}", std::mem::size_of::<PhantomData<User>>());
}
