// process::exit with a custom code after validating input records.
use std::process;

#[derive(Debug)]
enum Record {
    User { id: u32, name: String },
    Group(u32, Vec<u32>),
    Comment(String),
}

fn validate(records: &[Record]) -> Result<usize, (usize, String)> {
    let mut users = Vec::new();
    for (i, rec) in records.iter().enumerate() {
        match rec {
            Record::User { id, name } if name.is_empty() => {
                return Err((i, format!("user {} has no name", id)));
            }
            Record::User { id, .. } => users.push(*id),
            Record::Group(gid, members) => {
                if let Some(missing) = members.iter().find(|m| !users.contains(m)) {
                    return Err((i, format!("group {} references unknown user {}", gid, missing)));
                }
            }
            Record::Comment(_) => {}
        }
    }
    Ok(users.len())
}

fn main() {
    let good = vec![
        Record::User { id: 1, name: "ann".into() },
        Record::User { id: 2, name: "bob".into() },
        Record::Comment("admins".into()),
        Record::Group(10, vec![1, 2]),
    ];
    match validate(&good) {
        Ok(n) => println!("valid: {} users", n),
        Err((i, e)) => println!("record {}: {}", i, e),
    }
    let bad = vec![
        Record::User { id: 1, name: "ann".into() },
        Record::Group(11, vec![1, 3]),
    ];
    if let Err((index, msg)) = validate(&bad) {
        println!("record {}: {}", index, msg);
        println!("{:?}", bad[index]);
        process::exit(3);
    }
    println!("unreachable");
}
