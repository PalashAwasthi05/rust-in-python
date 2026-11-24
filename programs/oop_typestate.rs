// Typestate with PhantomData generic markers.
use std::marker::PhantomData;

struct Open;
struct Closed;
struct Locked;

struct Door<S> {
    name: String,
    opened_times: u32,
    _state: PhantomData<S>,
}

impl Door<Closed> {
    fn new(name: &str) -> Self {
        Door { name: name.to_string(), opened_times: 0, _state: PhantomData }
    }
    fn open(self) -> Door<Open> {
        println!("{}: opening", self.name);
        Door { name: self.name, opened_times: self.opened_times + 1, _state: PhantomData }
    }
    fn lock(self) -> Door<Locked> {
        println!("{}: locking", self.name);
        Door { name: self.name, opened_times: self.opened_times, _state: PhantomData }
    }
}

impl Door<Open> {
    fn close(self) -> Door<Closed> {
        println!("{}: closing", self.name);
        Door { name: self.name, opened_times: self.opened_times, _state: PhantomData }
    }
}

impl Door<Locked> {
    fn unlock(self, key: u32) -> Result<Door<Closed>, Door<Locked>> {
        if key == 42 {
            println!("{}: unlocked", self.name);
            Ok(Door { name: self.name, opened_times: self.opened_times, _state: PhantomData })
        } else {
            println!("{}: wrong key {}", self.name, key);
            Err(self)
        }
    }
}

trait StateName {
    const NAME: &'static str;
}
impl StateName for Open {
    const NAME: &'static str = "open";
}
impl StateName for Closed {
    const NAME: &'static str = "closed";
}
impl StateName for Locked {
    const NAME: &'static str = "locked";
}

impl<S: StateName> Door<S> {
    fn describe(&self) -> String {
        format!("door {} is {} (opened {} times)", self.name, S::NAME, self.opened_times)
    }
}

// Units of measure with PhantomData.
#[derive(Debug, Clone, Copy)]
struct Meters;
#[derive(Debug, Clone, Copy)]
struct Feet;

#[derive(Debug, Clone, Copy)]
struct Length<U> {
    value: f64,
    unit: PhantomData<U>,
}

impl<U> Length<U> {
    fn new(value: f64) -> Self {
        Length { value, unit: PhantomData }
    }
}

impl<U> std::ops::Add for Length<U> {
    type Output = Length<U>;
    fn add(self, other: Length<U>) -> Length<U> {
        Length::new(self.value + other.value)
    }
}

impl Length<Feet> {
    fn to_meters(self) -> Length<Meters> {
        Length::new(self.value * 0.3048)
    }
}

// A connection whose `send` only exists once connected.
struct Disconnected;
struct Connected;

struct Conn<State = Disconnected> {
    addr: String,
    sent: Vec<String>,
    marker: PhantomData<State>,
}

impl Conn {
    fn new(addr: &str) -> Conn<Disconnected> {
        Conn { addr: addr.into(), sent: vec![], marker: PhantomData }
    }
}

impl Conn<Disconnected> {
    fn connect(self) -> Conn<Connected> {
        Conn { addr: self.addr, sent: self.sent, marker: PhantomData }
    }
}

impl Conn<Connected> {
    fn send(&mut self, msg: &str) -> &mut Self {
        self.sent.push(msg.to_uppercase());
        self
    }
    fn disconnect(self) -> Conn<Disconnected> {
        println!("disconnect from {} after {:?}", self.addr, self.sent);
        Conn { addr: self.addr, sent: self.sent, marker: PhantomData }
    }
}

fn main() {
    let door = Door::new("front");
    println!("{}", door.describe());
    let door = door.open();
    println!("{}", door.describe());
    let door = door.close().open().close();
    println!("{}", door.describe());
    let locked = door.lock();
    println!("{}", locked.describe());
    let locked = match locked.unlock(7) {
        Ok(_) => panic!("should not unlock"),
        Err(d) => d,
    };
    let door = locked.unlock(42).ok().unwrap();
    println!("{}", door.describe());

    let a: Length<Feet> = Length::new(10.0);
    let b = Length::<Feet>::new(2.5);
    let total = a + b;
    let m = total.to_meters();
    println!("{} ft = {:.3} m", total.value, m.value);
    let m2 = m + Length::<Meters>::new(1.0);
    println!("{:.3}", m2.value);
    println!("size of Length<Feet> = {}", std::mem::size_of::<Length<Feet>>());
    println!("size of PhantomData<String> = {}", std::mem::size_of::<PhantomData<String>>());

    let mut c = Conn::new("10.0.0.1:80").connect();
    c.send("hello").send("world");
    let c = c.disconnect();
    println!("still has {} messages", c.sent.len());
}
