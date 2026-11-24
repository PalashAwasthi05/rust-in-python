// Trie keyed by chars with HashMap children; prefix search, deletion, autocomplete.
use std::collections::HashMap;

#[derive(Default, Debug)]
struct TrieNode {
    children: HashMap<char, TrieNode>,
    terminal: bool,
    count: u32,
}

#[derive(Default)]
struct Trie {
    root: TrieNode,
    words: usize,
}

impl Trie {
    fn new() -> Self {
        Self::default()
    }

    fn insert(&mut self, word: &str) -> bool {
        let mut node = &mut self.root;
        for ch in word.chars() {
            node.count += 1;
            node = node.children.entry(ch).or_default();
        }
        node.count += 1;
        if node.terminal {
            false
        } else {
            node.terminal = true;
            self.words += 1;
            true
        }
    }

    fn find(&self, prefix: &str) -> Option<&TrieNode> {
        let mut node = &self.root;
        for ch in prefix.chars() {
            node = node.children.get(&ch)?;
        }
        Some(node)
    }

    fn contains(&self, word: &str) -> bool {
        self.find(word).map_or(false, |n| n.terminal)
    }

    fn starts_with(&self, prefix: &str) -> bool {
        self.find(prefix).is_some()
    }

    fn complete(&self, prefix: &str) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(node) = self.find(prefix) {
            let mut buf = prefix.to_string();
            Self::collect(node, &mut buf, &mut out);
        }
        out.sort();
        out
    }

    fn collect(node: &TrieNode, buf: &mut String, out: &mut Vec<String>) {
        if node.terminal {
            out.push(buf.clone());
        }
        let mut keys: Vec<&char> = node.children.keys().collect();
        keys.sort();
        for ch in keys {
            buf.push(*ch);
            Self::collect(&node.children[ch], buf, out);
            buf.pop();
        }
    }

    fn remove(&mut self, word: &str) -> bool {
        fn go(node: &mut TrieNode, chars: &[char]) -> (bool, bool) {
            // returns (removed, prune_this_node)
            match chars.split_first() {
                None => {
                    if !node.terminal {
                        return (false, false);
                    }
                    node.terminal = false;
                    (true, node.children.is_empty())
                }
                Some((first, rest)) => {
                    let (removed, prune_child) = match node.children.get_mut(first) {
                        None => return (false, false),
                        Some(child) => go(child, rest),
                    };
                    if prune_child {
                        node.children.remove(first);
                    }
                    (removed, removed && !node.terminal && node.children.is_empty())
                }
            }
        }
        let chars: Vec<char> = word.chars().collect();
        let (removed, _) = go(&mut self.root, &chars);
        if removed {
            self.words -= 1;
        }
        removed
    }

    fn node_count(&self) -> usize {
        fn count(n: &TrieNode) -> usize {
            1 + n.children.values().map(count).sum::<usize>()
        }
        count(&self.root)
    }

    fn longest_common_prefix(&self) -> String {
        let mut out = String::new();
        let mut node = &self.root;
        while node.children.len() == 1 && !node.terminal {
            let (ch, next) = node.children.iter().next().unwrap();
            out.push(*ch);
            node = next;
        }
        out
    }
}

fn main() {
    let mut t = Trie::new();
    let words = ["car", "card", "care", "careful", "cart", "cat", "dog", "dot", "do", "card"];
    for w in words.iter() {
        let fresh = t.insert(w);
        if !fresh {
            println!("duplicate: {}", w);
        }
    }
    println!("words {} nodes {}", t.words, t.node_count());
    for q in ["car", "ca", "careful", "cards", "do", "d", "x"] {
        println!(
            "{:8} contains={:5} prefix={:5} complete={:?}",
            q,
            t.contains(q),
            t.starts_with(q),
            t.complete(q)
        );
    }
    println!("passing through 'car': {}", t.find("car").map(|n| n.count).unwrap_or(0));

    for w in ["care", "ca", "careful", "dog", "dog"] {
        println!("remove {:8} -> {} (words {}, nodes {})", w, t.remove(w), t.words, t.node_count());
    }
    println!("all: {:?}", t.complete(""));

    let mut common = Trie::new();
    for w in ["interview", "internet", "interval", "internal"] {
        common.insert(w);
    }
    println!("lcp = {:?}", common.longest_common_prefix());

    let mut unicode = Trie::new();
    for w in ["héllo", "hélium", "日本", "日本語"] {
        unicode.insert(w);
    }
    println!("hé* {:?}", unicode.complete("hé"));
    println!("日* {:?}", unicode.complete("日"));
}
