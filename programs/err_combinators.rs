#[derive(Debug, PartialEq, Clone)]
enum LookupError {
    NoUser(String),
    NoEmail(u32),
    BadDomain(String),
}

struct User {
    id: u32,
    name: &'static str,
    email: Option<&'static str>,
}

fn users() -> Vec<User> {
    vec![
        User { id: 1, name: "ada", email: Some("ada@example.com") },
        User { id: 2, name: "bob", email: None },
        User { id: 3, name: "cy", email: Some("cy-at-nowhere") },
    ]
}

fn find_user<'a>(all: &'a [User], name: &str) -> Result<&'a User, LookupError> {
    all.iter().find(|u| u.name == name).ok_or(LookupError::NoUser(name.to_string()))
}

fn domain_of(all: &[User], name: &str) -> Result<String, LookupError> {
    find_user(all, name)
        .and_then(|u| u.email.ok_or(LookupError::NoEmail(u.id)))
        .and_then(|e| {
            e.split_once('@')
                .map(|(_, d)| d.to_string())
                .ok_or_else(|| LookupError::BadDomain(e.to_string()))
        })
}

fn main() {
    let all = users();
    for name in ["ada", "bob", "cy", "dee"] {
        let r = domain_of(&all, name);
        println!("{:>4}: {:?}", name, r);
        let shown = r.clone().map(|d| d.to_uppercase()).unwrap_or_else(|e| format!("<{:?}>", e));
        println!("      {}", shown);
        println!("      is_ok={} is_err={}", r.is_ok(), r.is_err());
    }

    let maybe: Option<i32> = "17".parse().ok();
    println!("{:?}", maybe.map(|n| n + 1).filter(|n| n % 2 == 0));
    let r: Result<i32, String> = Err("boom".into());
    println!("{:?}", r.as_ref().map_err(|e| e.len()));
    println!("{}", r.clone().unwrap_or(-1));
    println!("{:?}", r.clone().or_else(|e| if e == "boom" { Ok::<i32, String>(0) } else { Err(e) }));
    println!("{:?}", r.clone().err());
    let ok: Result<i32, String> = Ok(3);
    println!("{:?}", ok.and_then(|n| if n > 2 { Err(format!("{} too big", n)) } else { Ok(n) }));
    let opt: Option<&str> = None;
    println!("{:?}", opt.ok_or("nothing"));
    println!("{:?}", Some(4).ok_or("nothing").map(|n| n * n));
    let nested: Option<Result<i32, String>> = Some(Ok(5));
    println!("{:?}", nested.transpose());
    let res_opt: Result<Option<i32>, String> = Ok(None);
    println!("{:?}", res_opt.transpose());
    println!("{}", Err::<i32, &str>("e").is_err_and(|e| e.is_empty()));
    println!("{}", Ok::<i32, &str>(8).is_ok_and(|n| n > 5));
    println!("{:?}", Ok::<i32, &str>(8).map_or(0, |n| n * 10));
    println!("{:?}", Err::<i32, &str>("bad").map_or_else(|e| e.len() as i32, |n| n * 10));
    println!("{:?}", Some(2).xor(None::<i32>));
    println!("{:?}", Ok::<i32, &str>(1).iter().next());
}
