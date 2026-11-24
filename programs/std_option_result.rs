// Option and Result combinators.
use std::collections::HashMap;

#[derive(Debug, Default, Clone, PartialEq)]
struct Config {
    name: String,
    retries: u32,
}

fn first_even_square(xs: &[i32]) -> Option<i32> {
    let first = xs.iter().find(|x| *x % 2 == 0)?;
    Some(first * first)
}

fn lookup_chain(map: &HashMap<&str, &str>, start: &str) -> Option<usize> {
    let a = map.get(start)?;
    let b = map.get(a)?;
    Some(b.len())
}

fn parse_pair(s: &str) -> Result<(i32, i32), String> {
    let (a, b) = s.split_once(',').ok_or_else(|| format!("no comma in {:?}", s))?;
    let a = a.trim().parse::<i32>().map_err(|e| format!("bad left: {}", e))?;
    let b = b.trim().parse::<i32>().map_err(|e| format!("bad right: {}", e))?;
    Ok((a, b))
}

fn main() {
    let some: Option<i32> = Some(4);
    let none: Option<i32> = None;
    println!("zip {:?} {:?}", some.zip(Some('x')), some.zip(None::<char>));
    println!("xor {:?} {:?} {:?}", some.xor(none), some.xor(Some(1)), none.xor(none));
    println!("or {:?} or_else {:?} and {:?}", none.or(Some(9)), none.or_else(|| Some(10)), some.and(Some("y")));
    println!("and_then {:?} {:?}", some.and_then(|x| if x > 3 { Some(x * 2) } else { None }), Some(1).and_then(|x| if x > 3 { Some(x) } else { None }));
    println!("map_or {} {} map_or_else {}", some.map_or(0, |x| x + 1), none.map_or(0, |x| x + 1), none.map_or_else(|| -1, |x| x));
    println!("filter {:?} {:?}", some.filter(|x| x % 2 == 0), some.filter(|x| *x > 10));
    println!("ok_or {:?} {:?}", some.ok_or("missing"), none.ok_or("missing"));
    println!("ok_or_else {:?}", none.ok_or_else(|| format!("err {}", 42)));
    println!("unwrap_or_default {} {:?}", none.unwrap_or_default(), None::<Config>.unwrap_or_default());
    println!("unwrap_or_else {}", none.unwrap_or_else(|| 7 * 6));
    println!("is_some_and {} is_none {}", some.is_some_and(|x| x > 2), none.is_none());
    println!("flatten {:?} {:?}", Some(Some(3)).flatten(), Some(None::<i32>).flatten());
    let t: Option<Result<i32, String>> = Some(Ok(5));
    println!("transpose {:?}", t.transpose());
    let r: Result<Option<i32>, String> = Ok(None);
    println!("transpose back {:?}", r.transpose());
    println!("iter over option {:?}", some.iter().chain(none.iter()).collect::<Vec<_>>());
    let mut total = 0;
    for x in some {
        total += x;
    }
    println!("for over option {}", total);
    let mixed = vec![Some(1), None, Some(3)];
    println!("flatten options {:?}", mixed.iter().flatten().collect::<Vec<_>>());
    let all: Option<Vec<i32>> = mixed.iter().cloned().collect();
    println!("collect options {:?}", all);
    let sum: Option<i32> = [Some(1), Some(2)].into_iter().sum();
    println!("sum options {:?}", sum);
    let mut slot = Some(3);
    println!("take {:?} {:?}", slot.take(), slot);
    println!("replace {:?} {:?}", slot.replace(8), slot);
    println!("get_or_insert {}", *slot.get_or_insert(1));
    let mut empty: Option<Vec<i32>> = None;
    empty.get_or_insert_with(Vec::new).push(5);
    println!("get_or_insert_with {:?}", empty);
    if let Some(v) = slot.as_mut() {
        *v += 1;
    }
    println!("as_mut {:?} as_ref map {:?}", slot, slot.as_ref().map(|v| v * 2));
    let name: Option<String> = Some("ferris".to_string());
    println!("as_deref {:?} len {:?}", name.as_deref(), name.as_ref().map(String::len));
    println!("copied {:?} cloned {:?}", [1, 2].first().copied(), name.iter().next().cloned());
    println!("first_even_square {:?} {:?}", first_even_square(&[1, 3, 6, 8]), first_even_square(&[1]));
    let map = HashMap::from([("a", "b"), ("b", "hello")]);
    println!("lookup_chain {:?} {:?}", lookup_chain(&map, "a"), lookup_chain(&map, "b"));

    for input in ["3, 4", "3 4", "x,1", "1,y"] {
        match parse_pair(input) {
            Ok((a, b)) => println!("pair {} {}", a, b),
            Err(e) => println!("error: {}", e),
        }
    }
    let good: Result<i32, String> = Ok(10);
    let bad: Result<i32, String> = Err("boom".into());
    println!("map {:?} map_err {:?}", good.clone().map(|x| x / 2), bad.clone().map_err(|e| e.len()));
    println!("and_then {:?}", good.clone().and_then(|x| if x > 5 { Ok(x) } else { Err("small".to_string()) }));
    println!("or_else {:?}", bad.clone().or_else(|e| if e == "boom" { Ok::<i32, String>(0) } else { Err(e) }));
    println!("ok {:?} err {:?}", good.clone().ok(), bad.clone().err());
    println!("unwrap_or {} unwrap_or_else {}", bad.clone().unwrap_or(-1), bad.clone().unwrap_or_else(|e| e.len() as i32));
    println!("unwrap_or_default {}", bad.clone().unwrap_or_default());
    println!("map_or {} is_ok_and {} is_err_and {}", good.clone().map_or(0, |x| x * 3), good.clone().is_ok_and(|x| x == 10), bad.clone().is_err_and(|e| e.is_empty()));
    println!("iter {:?} {:?}", good.iter().next(), bad.iter().next());
    println!("as_ref {:?}", bad.as_ref().map_err(|e| e.to_uppercase()));
    let results = vec![Ok(1), Err("a"), Ok(3), Err("b")];
    let (oks, errs): (Vec<_>, Vec<_>) = results.into_iter().partition(|r| r.is_ok());
    let oks: Vec<i32> = oks.into_iter().map(Result::unwrap).collect();
    let errs: Vec<&str> = errs.into_iter().filter_map(Result::err).collect();
    println!("partitioned {:?} {:?}", oks, errs);
    let collected: Result<Vec<i32>, &str> = vec![Ok(1), Err("first"), Err("second")].into_iter().collect();
    println!("collect result {:?}", collected);
    let cfg = Config { name: "x".into(), ..Default::default() };
    println!("default {:?} eq {}", cfg, cfg == Config::default());
    println!("copied err {:?}", Err::<&i32, ()>(()).copied());
    println!("expect {}", Some("works").expect("never"));
    println!("contains-like {}", Some(3) == Some(3));
    println!("cmp {} {}", Some(1) < None.or(Some(2)), None < Some(0));
    let bools = [true, false];
    println!("then {:?} {:?} then_some {:?}", bools[0].then(|| 1), bools[1].then(|| 1), true.then_some('k'));
}
