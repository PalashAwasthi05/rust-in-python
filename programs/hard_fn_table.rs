use std::collections::HashMap;

type Op = Box<dyn Fn(i32) -> i32>;

fn make_ops(offset: i32) -> HashMap<&'static str, Op> {
    let mut ops: HashMap<&'static str, Op> = HashMap::new();
    ops.insert("double", Box::new(|x| x * 2));
    ops.insert("negate", Box::new(|x| -x));
    ops.insert("square", Box::new(|x| x * x));
    ops.insert("offset", Box::new(move |x| x + offset));
    ops.insert("clamp", Box::new(|x| x.clamp(-10, 10)));
    ops
}

fn run(ops: &HashMap<&'static str, Op>, program: &str, start: i32) -> Result<i32, String> {
    program.split_whitespace().try_fold(start, |acc, name| {
        ops.get(name).map(|f| f(acc)).ok_or_else(|| format!("unknown op '{}'", name))
    })
}

fn pipeline(stages: Vec<Op>) -> Op {
    Box::new(move |x| stages.iter().fold(x, |acc, f| f(acc)))
}

fn main() {
    let ops = make_ops(5);
    let mut names: Vec<&&str> = ops.keys().collect();
    names.sort();
    for name in &names {
        let f = &ops[**name];
        println!("{:<7} 3 -> {:>3}, -4 -> {:>3}", name, f(3), f(-4));
    }

    for prog in ["double square", "offset negate clamp", "square square double", "double halve"] {
        match run(&ops, prog, 3) {
            Ok(v) => println!("{:<22} => {}", prog, v),
            Err(e) => println!("{:<22} => error: {}", prog, e),
        }
    }

    let mut stack: Vec<Op> = Vec::new();
    for k in 1..=3 {
        stack.push(Box::new(move |x| x * 10 + k));
    }
    let results: Vec<i32> = stack.iter().map(|f| f(0)).collect();
    println!("stacked: {:?}", results);

    let p = pipeline(stack);
    println!("pipeline(0) = {}", p(0));
    let composed = pipeline(vec![Box::new(|x| x + 1), Box::new(|x| x * x), Box::new(|x| x - 3)]);
    let outs: Vec<i32> = (0..5).map(|x| composed(x)).collect();
    println!("composed over 0..5: {:?}", outs);
}
