// Day 7 2022: building a directory tree from a terminal log, then recursive DFS.
use std::collections::HashMap;

const INPUT: &str = r"$ cd /
$ ls
dir a
14848514 b.txt
8504156 c.dat
dir d
$ cd a
$ ls
dir e
29116 f
2557 g
62596 h.lst
$ cd e
$ ls
584 i
$ cd ..
$ cd ..
$ cd d
$ ls
4060174 j
8033020 d.log
5626152 d.ext
7214296 k";

#[derive(Default, Debug)]
struct Dir {
    files: u64,
    children: Vec<String>,
}

fn total(path: &str, dirs: &HashMap<String, Dir>, sizes: &mut Vec<(String, u64)>) -> u64 {
    let d = &dirs[path];
    let mut sum = d.files;
    for child in &d.children {
        sum += total(child, dirs, sizes);
    }
    sizes.push((path.to_string(), sum));
    sum
}

fn main() {
    let mut dirs: HashMap<String, Dir> = HashMap::new();
    let mut cwd: Vec<&str> = Vec::new();
    dirs.insert("/".to_string(), Dir::default());
    for line in INPUT.lines() {
        let parts: Vec<&str> = line.split(' ').collect();
        let here = format!("/{}", cwd.join("/"));
        match parts.as_slice() {
            ["$", "cd", "/"] => cwd.clear(),
            ["$", "cd", ".."] => {
                cwd.pop();
            }
            ["$", "cd", name] => cwd.push(name),
            ["$", "ls"] => {}
            ["dir", name] => {
                let child = if cwd.is_empty() { format!("/{name}") } else { format!("{here}/{name}") };
                dirs.entry(child.clone()).or_default();
                dirs.get_mut(&here).unwrap().children.push(child);
            }
            [size, _name] => {
                dirs.get_mut(&here).unwrap().files += size.parse::<u64>().unwrap();
            }
            _ => panic!("unexpected line {line:?}"),
        }
    }
    let mut sizes = Vec::new();
    let used = total("/", &dirs, &mut sizes);
    sizes.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
    for (p, s) in &sizes {
        println!("{:>10} {}", s, p);
    }
    let part1: u64 = sizes.iter().map(|&(_, s)| s).filter(|&s| s <= 100_000).sum();
    println!("part 1: {part1}");
    let need = 30_000_000 - (70_000_000 - used);
    let part2 = sizes.iter().map(|(_, s)| *s).filter(|&s| s >= need).min().unwrap();
    println!("part 2: {part2}");
}
