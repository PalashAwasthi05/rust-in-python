// Chapter 5: structs, tuple structs, methods and associated functions.
#[derive(Debug)]
struct User {
    active: bool,
    username: String,
    email: String,
    sign_in_count: u64,
}

fn build_user(email: String, username: String) -> User {
    User {
        active: true,
        username,
        email,
        sign_in_count: 1,
    }
}

struct Color(i32, i32, i32);
struct Point(i32, i32, i32);
#[derive(Debug, PartialEq)]
struct AlwaysEqual;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Rectangle {
    width: u32,
    height: u32,
}

impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }

    fn width(&self) -> bool {
        self.width > 0
    }

    fn can_hold(&self, other: &Rectangle) -> bool {
        self.width > other.width && self.height > other.height
    }
}

impl Rectangle {
    fn square(size: u32) -> Self {
        Self {
            width: size,
            height: size,
        }
    }

    fn scale(&mut self, factor: u32) {
        self.width *= factor;
        self.height *= factor;
    }

    fn into_tuple(self) -> (u32, u32) {
        (self.width, self.height)
    }
}

fn area_tuple(dimensions: (u32, u32)) -> u32 {
    dimensions.0 * dimensions.1
}

fn area_struct(rectangle: &Rectangle) -> u32 {
    rectangle.width * rectangle.height
}

fn main() {
    let mut user1 = User {
        active: true,
        username: String::from("someusername123"),
        email: String::from("someone@example.com"),
        sign_in_count: 1,
    };
    user1.email = String::from("anotheremail@example.com");
    println!("{} <{}> active={} count={}", user1.username, user1.email, user1.active, user1.sign_in_count);

    let user2 = User {
        email: String::from("another@example.com"),
        ..user1
    };
    println!("user2: {} {}", user2.username, user2.email);
    println!("user1 still has email {}", user1.email);

    let user3 = build_user(String::from("ferris@example.com"), String::from("ferris"));
    println!("{:?}", user3);
    println!("{:#?}", user3);

    let black = Color(0, 0, 0);
    let origin = Point(1, 2, 3);
    let Point(px, py, pz) = origin;
    println!("black = ({}, {}, {}), origin = ({px}, {py}, {pz})", black.0, black.1, black.2);
    let subject = AlwaysEqual;
    println!("{:?} {}", subject, subject == AlwaysEqual);

    let width1 = 30;
    let height1 = 50;
    println!("The area of the rectangle is {} square pixels.", width1 * height1);
    let rect1 = (30, 50);
    println!("The area of the rectangle is {} square pixels.", area_tuple(rect1));
    let rect1 = Rectangle { width: 30, height: 50 };
    println!("The area of the rectangle is {} square pixels.", area_struct(&rect1));
    println!("rect1 is {:?}", rect1);
    println!("rect1 is {:#?}", rect1);

    let scale = 2;
    let rect2 = Rectangle {
        width: dbg!(30 * scale),
        height: 50,
    };
    dbg!(&rect2);
    println!("rect2 area {}", rect2.area());

    if rect1.width() {
        println!("The rectangle has a nonzero width; it is {}", rect1.width);
    }

    let rect2 = Rectangle { width: 10, height: 40 };
    let rect3 = Rectangle { width: 60, height: 45 };
    println!("Can rect1 hold rect2? {}", rect1.can_hold(&rect2));
    println!("Can rect1 hold rect3? {}", rect1.can_hold(&rect3));
    println!("Can rect1 hold rect3 (explicit)? {}", Rectangle::can_hold(&rect1, &rect3));

    let mut sq = Rectangle::square(3);
    println!("square {:?} area {}", sq, sq.area());
    sq.scale(4);
    println!("scaled {:?} area {}", sq, (&sq).area());
    let copy = sq;
    println!("copy == sq: {}", copy == sq);
    let (w, h) = sq.into_tuple();
    println!("tuple ({w}, {h})");

    let mut rects = vec![
        Rectangle { width: 10, height: 1 },
        Rectangle { width: 3, height: 5 },
        Rectangle { width: 7, height: 12 },
    ];
    rects.sort_by_key(|r| r.area());
    for r in &rects {
        println!("{}x{} -> {}", r.width, r.height, r.area());
    }
    let biggest = rects.iter().max_by_key(|r| r.area()).unwrap();
    println!("biggest {:?}", biggest);
}
