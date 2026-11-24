use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Category {
    Food,
    Tools,
    Books,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Category::Food => "food",
            Category::Tools => "tools",
            Category::Books => "books",
        };
        f.pad(name)
    }
}

struct Row {
    name: &'static str,
    category: Category,
    qty: u32,
    price: f64,
}

fn main() {
    let rows = vec![
        Row { name: "apples", category: Category::Food, qty: 12, price: 0.35 },
        Row { name: "hammer", category: Category::Tools, qty: 1, price: 12.5 },
        Row { name: "rust book", category: Category::Books, qty: 2, price: 39.995 },
        Row { name: "bread", category: Category::Food, qty: 3, price: 2.25 },
    ];
    println!("{:<12}|{:^8}|{:>5}|{:>10}", "item", "cat", "qty", "total");
    println!("{:-<12}+{:-^8}+{:->5}+{:->10}", "", "", "", "");
    let mut sums: BTreeMap<String, f64> = BTreeMap::new();
    let mut grand = 0.0;
    for row in &rows {
        let total = row.qty as f64 * row.price;
        grand += total;
        *sums.entry(row.category.to_string()).or_insert(0.0) += total;
        println!("{:<12}|{:^8}|{:>5}|{:>10.2}", row.name, row.category, row.qty, total);
    }
    println!("{:=>38}", "");
    println!("{:<12} {:>25.3}", "grand total", grand);
    for (cat, sum) in &sums {
        let pct = sum / grand * 100.0;
        println!("{cat:>8}: {sum:8.2} ({pct:5.1}%) {bar}", bar = "#".repeat((pct / 5.0) as usize));
    }
    println!("{:?}", sums);
    let cats: Vec<Category> = rows.iter().map(|r| r.category).collect();
    println!("{:?}", cats);
    println!("[{:>7}] [{:<7}] [{:^7}] [{:.2}]", Category::Food, Category::Tools, Category::Books, Category::Books);
    let ids = [7u32, 42, 1001];
    for id in ids.iter() {
        println!("#{:04} 0x{:04X} {:>#8b}", id, id, id % 16);
    }
    let mean = grand / rows.len() as f64;
    println!("mean={mean:.4} mean_e={mean:e} mean_dbg={mean:?}");
    let ratio = 2.0 / 3.0;
    println!("{:.0}% {:.1}% {:.15}", ratio * 100.0, ratio * 100.0, ratio);
}
