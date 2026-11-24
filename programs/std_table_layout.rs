// Laying out a text table with column widths computed from the data.
use std::fmt::Write;

#[derive(Clone, Copy, PartialEq)]
enum Align {
    Left,
    Right,
    Center,
}

struct Table {
    headers: Vec<String>,
    aligns: Vec<Align>,
    rows: Vec<Vec<String>>,
}

impl Table {
    fn new(headers: &[&str], aligns: &[Align]) -> Self {
        Table { headers: headers.iter().map(|h| h.to_string()).collect(), aligns: aligns.to_vec(), rows: Vec::new() }
    }

    fn push<I: IntoIterator<Item = String>>(&mut self, row: I) {
        let mut row: Vec<String> = row.into_iter().collect();
        row.resize(self.headers.len(), String::from("-"));
        self.rows.push(row);
    }

    fn widths(&self) -> Vec<usize> {
        (0..self.headers.len())
            .map(|c| self.rows.iter().map(|r| r[c].chars().count()).chain(std::iter::once(self.headers[c].len())).max().unwrap_or(0))
            .collect()
    }

    fn render(&self) -> String {
        let widths = self.widths();
        let mut out = String::new();
        let line: String = widths.iter().map(|w| "-".repeat(w + 2)).collect::<Vec<_>>().join("+");
        writeln!(out, "+{}+", line).unwrap();
        for (i, h) in self.headers.iter().enumerate() {
            write!(out, "| {:^w$} ", h, w = widths[i]).unwrap();
        }
        writeln!(out, "|").unwrap();
        writeln!(out, "+{}+", line.replace('-', "=")).unwrap();
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                let w = widths[i];
                match self.aligns[i] {
                    Align::Left => write!(out, "| {:<w$} ", cell).unwrap(),
                    Align::Right => write!(out, "| {:>w$} ", cell).unwrap(),
                    Align::Center => write!(out, "| {:^w$} ", cell).unwrap(),
                }
            }
            out.push_str("|\n");
        }
        write!(out, "+{}+", line).unwrap();
        out
    }
}

fn main() {
    let mut t = Table::new(&["planet", "moons", "radius km", "gravity"], &[Align::Left, Align::Right, Align::Right, Align::Center]);
    let data: [(&str, u32, f64, f64); 5] = [
        ("Mercury", 0, 2439.7, 3.7),
        ("Earth", 1, 6371.0, 9.807),
        ("Jupiter", 95, 69911.0, 24.79),
        ("Saturn", 146, 58232.0, 10.44),
        ("Neptune", 16, 24622.0, 11.15),
    ];
    for (name, moons, radius, g) in data.iter() {
        t.push(vec![name.to_string(), moons.to_string(), format!("{:.1}", radius), format!("{:.2}", g)]);
    }
    t.push(vec!["Pluto?".to_string()]);
    println!("{}", t.render());
    println!("widths {:?}", t.widths());
    let total_moons: u32 = data.iter().map(|d| d.1).sum();
    let biggest = data.iter().max_by(|a, b| a.2.total_cmp(&b.2)).map(|d| d.0).unwrap();
    let heaviest_g = data.iter().map(|d| d.3).fold(f64::NEG_INFINITY, f64::max);
    println!("{} moons, biggest {}, max g {}", total_moons, biggest, heaviest_g);
    let names_by_len = {
        let mut names: Vec<&str> = data.iter().map(|d| d.0).collect();
        names.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| b.cmp(a)));
        names
    };
    println!("{:?}", names_by_len);
    let w = names_by_len.iter().map(|n| n.len()).max().unwrap_or(0) + 2;
    for n in &names_by_len {
        println!("[{:>w$}] [{:-^w$}] [{:.3}]", n, n, n);
    }
    let ratio = data[2].2 / data[1].2;
    println!("{:>10.4}|{:<10.2e}|{:+.0}|{:08.3}", ratio, ratio * 1e6, ratio, -ratio);
}
