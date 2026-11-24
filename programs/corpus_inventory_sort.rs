#[derive(Debug, Clone)]
struct Item {
    sku: u32,
    name: String,
    category: String,
    price_cents: u64,
    qty: u32,
}

impl Item {
    fn new(sku: u32, name: &str, category: &str, price_cents: u64, qty: u32) -> Self {
        Item { sku, name: name.into(), category: category.into(), price_cents, qty }
    }
    fn value(&self) -> u64 { self.price_cents * self.qty as u64 }
    fn in_stock(&self) -> bool { self.qty > 0 }
}

struct Inventory { items: Vec<Item> }

impl Inventory {
    fn restock(&mut self, sku: u32, amount: u32) -> bool {
        match self.items.iter_mut().find(|i| i.sku == sku) {
            Some(item) => { item.qty += amount; true }
            None => false,
        }
    }
    fn total_value(&self) -> u64 { self.items.iter().map(Item::value).sum() }
    fn print(&self, title: &str) {
        println!("-- {} --", title);
        for i in &self.items {
            println!("{:>4} {:<12} {:<8} ${:>3}.{:02} x{:<3} {}", i.sku, i.name, i.category,
                i.price_cents / 100, i.price_cents % 100, i.qty, if i.in_stock() { "" } else { "(out)" });
        }
    }
}

fn main() {
    let mut inv = Inventory { items: vec![
        Item::new(101, "Hammer", "tools", 1599, 12),
        Item::new(205, "Nails", "hardware", 499, 0),
        Item::new(102, "Wrench", "tools", 1599, 4),
        Item::new(310, "Paint", "supplies", 2250, 7),
        Item::new(206, "Screws", "hardware", 650, 40),
        Item::new(103, "Saw", "tools", 2899, 2),
        Item::new(311, "Brush", "supplies", 350, 0),
    ]};
    inv.print("as entered");

    inv.items.sort_by(|a, b| a.category.cmp(&b.category)
        .then(b.price_cents.cmp(&a.price_cents))
        .then_with(|| a.name.cmp(&b.name)));
    inv.print("by category, price desc, name");

    inv.items.sort_by_key(|i| (!i.in_stock(), std::cmp::Reverse(i.value()), i.sku));
    inv.print("in stock first, by value desc");

    println!("restock 205: {}, restock 999: {}", inv.restock(205, 100), inv.restock(999, 1));
    let total = inv.total_value();
    println!("total value: ${}.{:02}", total / 100, total % 100);
    let out: Vec<&str> = inv.items.iter().filter(|i| !i.in_stock()).map(|i| i.name.as_str()).collect();
    println!("out of stock: {:?}", out);
    let cheapest_tool = inv.items.iter().filter(|i| i.category == "tools").min_by_key(|i| (i.price_cents, i.sku));
    println!("cheapest tool: {:?}", cheapest_tool.map(|i| &i.name));
}
