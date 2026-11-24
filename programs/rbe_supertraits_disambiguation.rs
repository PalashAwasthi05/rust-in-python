trait Person {
    fn name(&self) -> String;
}

trait Student: Person {
    fn university(&self) -> String;
}

trait Programmer {
    fn fav_language(&self) -> String;
}

trait CompSciStudent: Programmer + Student {
    fn git_username(&self) -> String;
}

fn comp_sci_student_greeting(student: &dyn CompSciStudent) -> String {
    format!(
        "My name is {} and I attend {}. My favorite language is {}. My Git username is {}",
        student.name(),
        student.university(),
        student.fav_language(),
        student.git_username()
    )
}

struct Grad;
impl Person for Grad { fn name(&self) -> String { "Ada".into() } }
impl Student for Grad { fn university(&self) -> String { "Cambridge".into() } }
impl Programmer for Grad { fn fav_language(&self) -> String { "Rust".into() } }
impl CompSciStudent for Grad { fn git_username(&self) -> String { "ada-l".into() } }

trait UsernameWidget { fn get(&self) -> String; }
trait AgeWidget { fn get(&self) -> u8; }

struct Form { username: String, age: u8 }

impl UsernameWidget for Form { fn get(&self) -> String { self.username.clone() } }
impl AgeWidget for Form { fn get(&self) -> u8 { self.age } }

trait Pilot { fn fly(&self) -> String; fn id() -> &'static str; }
trait Wizard { fn fly(&self) -> String; fn id() -> &'static str; }

struct Human;
impl Pilot for Human {
    fn fly(&self) -> String { "This is your captain speaking.".into() }
    fn id() -> &'static str { "pilot" }
}
impl Wizard for Human {
    fn fly(&self) -> String { "Up!".into() }
    fn id() -> &'static str { "wizard" }
}
impl Human {
    fn fly(&self) -> String { "*waving arms furiously*".into() }
    fn id() -> &'static str { "human" }
}

trait Shape: std::fmt::Debug {
    fn area(&self) -> f64;
    fn report(&self) -> String { format!("{:?} has area {:.2}", self, self.area()) }
}

trait Solid: Shape {
    fn depth(&self) -> f64;
    fn volume(&self) -> f64 { self.area() * self.depth() }
}

#[derive(Debug)]
struct Slab { w: f64, h: f64, d: f64 }
impl Shape for Slab { fn area(&self) -> f64 { self.w * self.h } }
impl Solid for Slab { fn depth(&self) -> f64 { self.d } }

fn as_shape(s: &dyn Solid) -> String { s.report() }

fn main() {
    println!("{}", comp_sci_student_greeting(&Grad));

    let form = Form { username: "rustacean".to_owned(), age: 28 };
    let username = <Form as UsernameWidget>::get(&form);
    let age = <Form as AgeWidget>::get(&form);
    println!("username {:?}, age {}", username, age);
    println!("via UFCS: {} {}", UsernameWidget::get(&form), AgeWidget::get(&form));

    let person = Human;
    println!("{}", person.fly());
    println!("{}", Pilot::fly(&person));
    println!("{}", Wizard::fly(&person));
    println!("{}", Human::fly(&person));
    println!("ids: {} {} {}", Human::id(), <Human as Pilot>::id(), <Human as Wizard>::id());

    let slab = Slab { w: 2.0, h: 3.0, d: 0.5 };
    println!("{}", as_shape(&slab));
    println!("volume {}", slab.volume());
    println!("string method via path: {}", <str as ToString>::to_string("abc"));
    println!("i32 via trait path: {}", <i32 as std::str::FromStr>::from_str("-12").unwrap());
    println!("default via path: {:?}", <Vec<u8> as Default>::default());
}
