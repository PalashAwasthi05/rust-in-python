struct Person {
    name: String,
    address: Option<Address>,
}

struct Address {
    city: Option<String>,
    zip: Option<u32>,
}

fn city_len(p: &Person) -> Option<usize> {
    let addr = p.address.as_ref()?;
    let city = addr.city.as_ref()?;
    Some(city.len())
}

fn zip_prefix(p: &Person) -> Option<u32> {
    Some(p.address.as_ref()?.zip? / 1000)
}

fn first_even_square(v: &[i32]) -> Option<i32> {
    let first = v.iter().find(|n| **n % 2 == 0)?;
    first.checked_mul(*first)
}

fn last_char_digit(s: &str) -> Option<u32> {
    s.chars().last()?.to_digit(10)
}

fn main() {
    let people = vec![
        Person { name: "a".into(), address: None },
        Person { name: "b".into(), address: Some(Address { city: None, zip: Some(94110) }) },
        Person { name: "c".into(), address: Some(Address { city: Some("Oslo".into()), zip: None }) },
    ];
    for p in &people {
        println!("{} {:?} {:?}", p.name, city_len(p), zip_prefix(p));
    }
    println!("{:?}", first_even_square(&[1, 3, 6, 8]));
    println!("{:?}", first_even_square(&[1, 3, 5]));
    println!("{:?}", first_even_square(&[65536]));
    println!("{:?} {:?} {:?}", last_char_digit("ab7"), last_char_digit("abc"), last_char_digit(""));
}
