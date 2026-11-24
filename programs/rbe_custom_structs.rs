#[derive(Debug, Clone, Copy, PartialEq)]
struct Point { x: f32, y: f32 }

#[derive(Debug)]
struct Rectangle { top_left: Point, bottom_right: Point }

#[derive(Debug, Clone, Copy)]
struct Pair(i32, f32);

#[derive(Debug, PartialEq)]
struct Unit;

#[derive(Debug)]
struct Person { name: String, age: u8 }

impl Rectangle {
    fn width(&self) -> f32 { self.bottom_right.x - self.top_left.x }
    fn height(&self) -> f32 { self.top_left.y - self.bottom_right.y }
    fn area(&self) -> f32 { self.width() * self.height() }
}

fn square(p: Point, side: f32) -> Rectangle {
    Rectangle { top_left: p, bottom_right: Point { x: p.x + side, y: p.y - side } }
}

fn main() {
    let name = String::from("Peter");
    let age = 27;
    let peter = Person { name, age };
    println!("{:?}", peter);
    let older = Person { age: 28, ..peter };
    println!("{} is now {}", older.name, older.age);

    let point = Point { x: 5.2, y: 0.4 };
    let another = Point { x: 10.3, ..point };
    println!("point ({}, {}) another ({}, {})", point.x, point.y, another.x, another.y);

    let Point { x: left_edge, y: top_edge } = point;
    let rect = Rectangle {
        top_left: Point { x: left_edge, y: top_edge + 5.0 },
        bottom_right: Point { x: left_edge + 3.0, y: top_edge },
    };
    println!("{:?}", rect);
    println!("w={:.1} h={:.1} area={:.2}", rect.width(), rect.height(), rect.area());
    let sq = square(Point { x: 1.0, y: 1.0 }, 2.5);
    println!("square area {:.2}, corners equal? {}", sq.area(), sq.top_left == sq.bottom_right);

    let _unit = Unit;
    println!("unit struct: {:?}, equal to itself: {}", Unit, Unit == Unit);
    println!("unit size: {}", std::mem::size_of::<Unit>());

    let pair = Pair(1, 0.1);
    println!("pair contains {:?} and {:?}", pair.0, pair.1);
    let Pair(integer, decimal) = pair;
    println!("destructured {} {}", integer, decimal);
    let pairs: Vec<Pair> = (1..=3).map(|i| Pair(i, i as f32 / 4.0)).collect();
    let total: f32 = pairs.iter().map(|p| p.0 as f32 * p.1).sum();
    println!("pairs {:?} weighted {}", pairs, total);

    let mut people = vec![
        Person { name: "Zoe".into(), age: 31 },
        Person { name: "Al".into(), age: 19 },
        Person { name: "Mo".into(), age: 31 },
    ];
    people.sort_by(|a, b| b.age.cmp(&a.age).then(a.name.cmp(&b.name)));
    for Person { name, age } in &people {
        println!("  {:<4} {}", name, age);
    }
    if let Some(p) = people.iter_mut().find(|p| p.name == "Al") {
        p.age += 1;
    }
    println!("Al after birthday: {:?}", people.iter().find(|p| p.name == "Al"));
}
