#[derive(Debug, Clone, Copy, PartialEq)]
enum Food { CordonBleu, Steak, Sushi, Apple, Carrot, Potato }

#[derive(Debug)]
struct Peeled(Food);
#[derive(Debug)]
struct Chopped(Food);
#[derive(Debug)]
struct Cooked(Food);

fn peel(food: Option<Food>) -> Option<Peeled> {
    match food {
        Some(food) => Some(Peeled(food)),
        None => None,
    }
}

fn chop(peeled: Option<Peeled>) -> Option<Chopped> {
    peeled.map(|Peeled(food)| Chopped(food))
}

fn cook(chopped: Option<Chopped>) -> Option<Cooked> {
    chopped.map(|Chopped(food)| Cooked(food))
}

fn process(food: Option<Food>) -> Option<Cooked> {
    food.map(|f| Peeled(f)).map(|Peeled(f)| Chopped(f)).map(|Chopped(f)| Cooked(f))
}

fn have_ingredients(food: Food) -> Option<Food> {
    match food { Food::Sushi => None, _ => Some(food) }
}

fn have_recipe(food: Food) -> Option<Food> {
    match food { Food::CordonBleu => None, _ => Some(food) }
}

fn cookable(food: Food) -> Option<Food> {
    have_recipe(food).and_then(have_ingredients)
}

struct Person { job: Option<Job> }
#[derive(Clone, Copy)]
struct Job { phone_number: Option<PhoneNumber> }
#[derive(Clone, Copy)]
struct PhoneNumber { area_code: Option<u8>, number: u32 }

impl Person {
    fn work_phone_area_code(&self) -> Option<u8> {
        self.job?.phone_number?.area_code
    }
}

fn eat(food: Option<Cooked>) -> String {
    match food {
        Some(food) => format!("Mmm. I love {:?}", food),
        None => "Oh no! It wasn't edible.".to_string(),
    }
}

fn main() {
    let apple = Some(Food::Apple);
    let carrot = Some(Food::Carrot);
    let potato = None;
    let cooked_apple = cook(chop(peel(apple)));
    let cooked_carrot = cook(chop(peel(carrot)));
    let cooked_potato = process(potato);
    println!("{}", eat(cooked_apple));
    println!("{}", eat(cooked_carrot));
    println!("{}", eat(cooked_potato));
    println!("{}", eat(process(Some(Food::Potato))));

    for food in [Food::CordonBleu, Food::Steak, Food::Sushi] {
        match cookable(food) {
            Some(f) => println!("Yay! We get to eat {:?}.", f),
            None => println!("Oh no. We don't get to eat {:?}.", food),
        }
    }

    let p = Person { job: Some(Job { phone_number: Some(PhoneNumber { area_code: Some(61), number: 439222222 }) }) };
    println!("area code: {:?}", p.work_phone_area_code());
    let jobless = Person { job: None };
    println!("jobless area code: {:?}", jobless.work_phone_area_code());
    println!("number: {:?}", p.job.and_then(|j| j.phone_number).map(|n| n.number));

    // or, or_else, get_or_insert, unwrap_or variants, filter, zip, xor
    let apple = Some(Food::Apple);
    let no_fruit: Option<Food> = None;
    println!("or: {:?}", no_fruit.or(apple));
    println!("or_else: {:?}", no_fruit.or_else(|| Some(Food::Carrot)));
    let mut basket: Option<Food> = None;
    let got = *basket.get_or_insert(Food::Steak);
    println!("get_or_insert: {:?}, basket now {:?}", got, basket);
    let mut lazy: Option<u32> = None;
    *lazy.get_or_insert_with(|| 41) += 1;
    println!("get_or_insert_with then mutate: {:?}", lazy);
    println!("unwrap_or {} unwrap_or_default {} unwrap_or_else {}", None.unwrap_or(5), None::<i32>.unwrap_or_default(), None.unwrap_or_else(|| 2 + 2));
    println!("filter: {:?} {:?}", Some(4).filter(|x| x % 2 == 0), Some(3).filter(|x| x % 2 == 0));
    println!("zip: {:?} xor: {:?} {:?}", Some(1).zip(Some('a')), Some(1).xor(None), Some(1).xor(Some(2)));
    println!("ok_or: {:?} {:?}", Some(3).ok_or("none"), None::<i32>.ok_or("none"));
    println!("map_or: {} {}", Some(5).map_or(0, |v| v * 10), None::<i32>.map_or(0, |v| v * 10));
    let words = ["3", "x", "7"];
    let parsed: Vec<Option<i32>> = words.iter().map(|w| w.parse().ok()).collect();
    let all: Option<Vec<i32>> = parsed.iter().cloned().collect();
    let some_only: Vec<i32> = parsed.iter().flatten().copied().collect();
    println!("parsed {:?} all {:?} flattened {:?}", parsed, all, some_only);
    println!("as_ref/as_deref: {:?} {:?}", Some(String::from("hi")).as_ref().map(|s| s.len()), Some(String::from("yo")).as_deref());
}
