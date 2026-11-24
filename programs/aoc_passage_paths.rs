// Day 12 2021: counting cave paths with recursive DFS over a HashMap<&str, Vec<&str>>.
use std::collections::{HashMap, HashSet};

const INPUT: &str = r"dc-end
HN-start
start-kj
dc-start
dc-HN
LN-dc
HN-end
kj-sa
kj-HN
kj-dc";

fn is_small(cave: &str) -> bool {
    cave.chars().all(|c| c.is_lowercase())
}

fn count<'a>(
    graph: &HashMap<&'a str, Vec<&'a str>>,
    cave: &'a str,
    visited: &mut HashSet<&'a str>,
    twice_left: bool,
) -> usize {
    if cave == "end" {
        return 1;
    }
    let mut total = 0;
    for &next in &graph[cave] {
        if next == "start" {
            continue;
        }
        if is_small(next) {
            if !visited.contains(next) {
                visited.insert(next);
                total += count(graph, next, visited, twice_left);
                visited.remove(next);
            } else if twice_left {
                total += count(graph, next, visited, false);
            }
        } else {
            total += count(graph, next, visited, twice_left);
        }
    }
    total
}

fn main() {
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
    for line in INPUT.lines() {
        let (a, b) = line.split_once('-').unwrap();
        graph.entry(a).or_default().push(b);
        graph.entry(b).or_default().push(a);
    }
    for list in graph.values_mut() {
        list.sort_unstable();
    }
    let mut caves: Vec<_> = graph.iter().map(|(k, v)| (*k, v.len())).collect();
    caves.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    println!("{:?}", &caves[..4]);
    let mut visited = HashSet::new();
    println!("part 1: {}", count(&graph, "start", &mut visited, false));
    println!("part 2: {}", count(&graph, "start", &mut visited, true));
    let small = graph.keys().filter(|k| is_small(k)).count();
    println!("{} caves, {} small", graph.len(), small);
}
