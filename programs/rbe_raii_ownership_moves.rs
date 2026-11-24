struct ToDrop { name: &'static str }

impl Drop for ToDrop {
    fn drop(&mut self) {
        println!("drop {}", self.name);
    }
}

#[derive(Debug)]
struct Person { name: String, age: Box<u8> }

fn destroy_box(c: Box<i32>) {
    println!("destroying a box that contains {}", c);
}

fn take_and_give_back(mut v: Vec<i32>) -> Vec<i32> {
    v.push(99);
    v
}

fn make(name: &'static str) -> ToDrop {
    println!("make {}", name);
    ToDrop { name }
}

fn main() {
    // RAII: values are dropped at the end of their scope, in reverse order
    let _a = make("a");
    {
        let _b = make("b");
        let _c = make("c");
        println!("leaving inner scope");
    }
    let moved = make("moved");
    let new_owner = moved;
    println!("ownership moved to {}", new_owner.name);
    drop(new_owner);
    println!("dropped explicitly");
    let mut replaced = make("first");
    replaced = make("second");
    println!("replaced now holds {}", replaced.name);

    // ownership and moves
    let x = 5u32;
    let y = x;
    println!("x is {}, and y is {}", x, y);
    let a = Box::new(5i32);
    println!("a contains: {}", a);
    let b = a;
    destroy_box(b);
    let v = vec![1, 2];
    let v = take_and_give_back(v);
    println!("got back {:?}", v);

    // mutability can change on move
    let immutable_box = Box::new(5u32);
    let mut mutable_box = immutable_box;
    *mutable_box = 4;
    println!("mutable_box now contains {}", mutable_box);

    // partial moves
    let person = Person { name: String::from("Alice"), age: Box::new(20) };
    let Person { name, ref age } = person;
    println!("the person's age is {}", age);
    println!("the person's name is {}", name);
    println!("the person's age from person struct is {}", person.age);

    let pair = (String::from("left"), String::from("right"));
    let (l, _) = pair;
    println!("took {}, pair.1 still usable: {}", l, pair.1);

    let names = vec![String::from("x"), String::from("y")];
    for n in &names { print!("{} ", n); }
    println!("(borrowed loop)");
    for n in names { print!("{} ", n.to_uppercase()); }
    println!("(consuming loop)");
    let guards: Vec<ToDrop> = vec![make("v1"), make("v2")];
    println!("vec of {} guards going out of scope", guards.len());
}
