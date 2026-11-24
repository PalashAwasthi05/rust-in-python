use std::collections::BTreeMap;

#[derive(Debug)]
struct Order { item: &'static str, qty: u32 }

fn price_of(menu: &BTreeMap<&str, u32>, item: &str) -> Option<u32> {
    menu.get(item).copied()
}

fn main() {
    let mut menu: BTreeMap<&str, u32> = BTreeMap::new();
    menu.insert("apple", 40);
    menu.insert("pear", 55);
    menu.insert("plum", 30);
    let orders = vec![
        Order { item: "pear", qty: 2 },
        Order { item: "apple", qty: 3 },
        Order { item: "mango", qty: 1 },
        Order { item: "plum", qty: 5 },
    ];
    for o in &orders {
        let quoted = price_of(&menu, o.item).map(|p| p * o.qty);
        println!("quote {:?}: {:?}", o, quoted);
    }
    let cheapest = menu.iter().min_by_key(|(_, &p)| p).map(|(k, _)| *k);
    println!("cheapest: {:?}", cheapest);
    let mut total = 0;
    for o in &orders {
        let price = price_of(&menu, o.item).unwrap();
        total += price * o.qty;
        println!("charged {} x{} = {} (total {})", o.item, o.qty, price * o.qty, total);
    }
    println!("unreachable total {}", total);
}
