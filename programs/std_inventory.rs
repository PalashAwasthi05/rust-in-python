// Inventory ledger using BTreeMap entries, Option/Result chains and formatted reports.
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
struct Item {
    name: String,
    qty: u32,
    price_cents: u64,
}

#[derive(Debug)]
enum LedgerError {
    UnknownSku(String),
    Insufficient { sku: String, have: u32, want: u32 },
    Parse(String),
}

impl fmt::Display for LedgerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LedgerError::UnknownSku(s) => write!(f, "unknown sku {}", s),
            LedgerError::Insufficient { sku, have, want } => write!(f, "{}: have {}, want {}", sku, have, want),
            LedgerError::Parse(msg) => write!(f, "parse error: {}", msg),
        }
    }
}

struct Ledger {
    items: BTreeMap<String, Item>,
}

impl Ledger {
    fn apply(&mut self, line: &str) -> Result<String, LedgerError> {
        let mut parts = line.split_whitespace();
        let op = parts.next().ok_or_else(|| LedgerError::Parse("empty".into()))?;
        let sku = parts.next().ok_or_else(|| LedgerError::Parse(format!("missing sku in {:?}", line)))?.to_string();
        let qty: u32 = parts
            .next()
            .unwrap_or("1")
            .parse()
            .map_err(|e: std::num::ParseIntError| LedgerError::Parse(e.to_string()))?;
        match op {
            "add" => {
                let price = parts.next().and_then(|p| p.parse::<f64>().ok()).map(|p| (p * 100.0).round() as u64);
                match self.items.entry(sku.clone()) {
                    Entry::Occupied(mut e) => {
                        e.get_mut().qty += qty;
                        Ok(format!("restocked {} to {}", sku, e.get().qty))
                    }
                    Entry::Vacant(e) => {
                        let item = e.insert(Item { name: sku.to_uppercase(), qty, price_cents: price.unwrap_or(0) });
                        Ok(format!("new item {} x{}", item.name, item.qty))
                    }
                }
            }
            "take" => {
                let item = self.items.get_mut(&sku).ok_or(LedgerError::UnknownSku(sku.clone()))?;
                item.qty = item.qty.checked_sub(qty).ok_or(LedgerError::Insufficient { sku: sku.clone(), have: item.qty, want: qty })?;
                Ok(format!("took {} {}", qty, sku))
            }
            other => Err(LedgerError::Parse(format!("bad op {}", other))),
        }
    }

    fn total_value(&self) -> u64 {
        self.items.values().map(|i| i.qty as u64 * i.price_cents).sum()
    }
}

fn dollars(cents: u64) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}

fn main() {
    let mut ledger = Ledger { items: BTreeMap::new() };
    let script = "add bolt 100 0.15\nadd nut 250 0.05\nadd bolt 20\ntake nut 30\ntake gear 1\ntake bolt 500\nsell bolt 1\nadd washer x\nadd\nadd gear 3 12.50";
    for line in script.lines() {
        match ledger.apply(line) {
            Ok(msg) => println!("ok: {}", msg),
            Err(e) => println!("err: {} ({:?})", e, e),
        }
    }
    println!("{:<8}|{:>6}|{:>8}|{:>10}", "sku", "qty", "price", "value");
    for (sku, item) in &ledger.items {
        println!("{:<8}|{:>6}|{:>8}|{:>10}", sku, item.qty, dollars(item.price_cents), dollars(item.qty as u64 * item.price_cents));
    }
    println!("total {}", dollars(ledger.total_value()));
    let cheapest = ledger.items.values().min_by_key(|i| i.price_cents).map(|i| &i.name);
    println!("cheapest {:?}", cheapest);
    let low: Vec<&String> = ledger.items.iter().filter(|(_, i)| i.qty < 50).map(|(k, _)| k).collect();
    println!("low stock {:?}", low);
    if let Entry::Occupied(e) = ledger.items.entry("gear".to_string()) {
        let removed = e.remove();
        println!("removed {:?}", removed);
    }
    match ledger.items.entry("zzz".into()) {
        Entry::Vacant(v) => println!("vacant key {}", v.key()),
        Entry::Occupied(_) => println!("occupied"),
    }
    println!("keys {:?}", ledger.items.keys().collect::<Vec<_>>());
}
