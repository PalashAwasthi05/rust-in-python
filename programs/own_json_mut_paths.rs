use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
enum Json {
    Null,
    Num(f64),
    Str(String),
    List(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    fn get_mut(&mut self, key: &str) -> Option<&mut Json> {
        match self {
            Json::Obj(pairs) => pairs.iter_mut().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    fn get(&self, key: &str) -> Option<&Json> {
        if let Json::Obj(pairs) = self {
            for (k, v) in pairs {
                if k == key {
                    return Some(v);
                }
            }
        }
        None
    }

    fn push(&mut self, item: Json) -> bool {
        if let Json::List(items) = self {
            items.push(item);
            true
        } else {
            false
        }
    }

    fn walk<'a>(&'a self, path: &mut Vec<String>, out: &mut Vec<(String, &'a Json)>) {
        match self {
            Json::Obj(pairs) => {
                for (k, v) in pairs {
                    path.push(k.clone());
                    v.walk(path, out);
                    path.pop();
                }
            }
            Json::List(items) => {
                for (i, v) in items.iter().enumerate() {
                    path.push(i.to_string());
                    v.walk(path, out);
                    path.pop();
                }
            }
            leaf => out.push((path.join("."), leaf)),
        }
    }

    fn count_nums(&mut self, counter: &mut HashMap<&'static str, u32>) {
        match self {
            Json::Num(n) => {
                *n *= 2.0;
                *counter.entry("num").or_default() += 1;
            }
            Json::List(items) => items.iter_mut().for_each(|j| j.count_nums(counter)),
            Json::Obj(pairs) => {
                for (_, v) in pairs.iter_mut() {
                    v.count_nums(counter);
                }
            }
            _ => *counter.entry("other").or_default() += 1,
        }
    }
}

fn main() {
    let mut doc = Json::Obj(vec![
        ("name".into(), Json::Str("widget".into())),
        ("sizes".into(), Json::List(vec![Json::Num(1.0), Json::Num(2.5)])),
        ("meta".into(), Json::Obj(vec![("owner".into(), Json::Null)])),
    ]);
    if let Some(sizes) = doc.get_mut("sizes") {
        sizes.push(Json::Num(4.0));
    }
    if let Some(Json::Obj(meta)) = doc.get_mut("meta") {
        meta.push(("tags".into(), Json::List(vec![])));
    }
    let pushed = doc.get_mut("meta").and_then(|m| m.get_mut("tags")).map(|t| t.push(Json::Str("x".into())));
    println!("pushed {:?}", pushed);
    if let Some(Json::Str(s)) = doc.get_mut("name") {
        s.push_str("-v2");
    }
    let mut counter = HashMap::new();
    doc.count_nums(&mut counter);
    let mut keys: Vec<_> = counter.iter().collect();
    keys.sort();
    println!("{:?}", keys);
    let mut out = Vec::new();
    doc.walk(&mut Vec::new(), &mut out);
    for (p, v) in &out {
        println!("{} = {:?}", p, v);
    }
    println!("{:?}", doc.get("name"));
    let snapshot = doc.clone();
    if let Some(m) = doc.get_mut("meta") {
        *m = Json::Null;
    }
    println!("{}", snapshot == doc);
    println!("{:?}", doc.get("meta"));
}
