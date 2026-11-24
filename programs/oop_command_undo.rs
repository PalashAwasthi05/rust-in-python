// Command pattern with an undo/redo stack of Box<dyn Command>.
use std::fmt;

struct Document {
    text: String,
}

trait Command: fmt::Debug {
    fn execute(&mut self, doc: &mut Document);
    fn undo(&mut self, doc: &mut Document);
    fn describe(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug)]
struct Insert {
    pos: usize,
    text: String,
}

#[derive(Debug)]
struct Delete {
    pos: usize,
    len: usize,
    removed: Option<String>,
}

#[derive(Debug)]
struct Upper {
    before: String,
}

impl Command for Insert {
    fn execute(&mut self, doc: &mut Document) {
        doc.text.insert_str(self.pos, &self.text);
    }
    fn undo(&mut self, doc: &mut Document) {
        doc.text.replace_range(self.pos..self.pos + self.text.len(), "");
    }
}

impl Command for Delete {
    fn execute(&mut self, doc: &mut Document) {
        let end = (self.pos + self.len).min(doc.text.len());
        let removed: String = doc.text.drain(self.pos..end).collect();
        self.removed = Some(removed);
    }
    fn undo(&mut self, doc: &mut Document) {
        if let Some(r) = self.removed.take() {
            doc.text.insert_str(self.pos, &r);
        }
    }
    fn describe(&self) -> String {
        format!("Delete {} chars at {}", self.len, self.pos)
    }
}

impl Command for Upper {
    fn execute(&mut self, doc: &mut Document) {
        self.before = doc.text.clone();
        doc.text = doc.text.to_uppercase();
    }
    fn undo(&mut self, doc: &mut Document) {
        doc.text = std::mem::take(&mut self.before);
    }
}

struct Editor {
    doc: Document,
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

impl Editor {
    fn new() -> Self {
        Editor { doc: Document { text: String::new() }, undo_stack: vec![], redo_stack: vec![] }
    }

    fn run(&mut self, mut cmd: Box<dyn Command>) {
        cmd.execute(&mut self.doc);
        println!("run  {:<28} -> {:?}", cmd.describe(), self.doc.text);
        self.undo_stack.push(cmd);
        self.redo_stack.clear();
    }

    fn undo(&mut self) -> bool {
        match self.undo_stack.pop() {
            Some(mut cmd) => {
                cmd.undo(&mut self.doc);
                println!("undo {:<28} -> {:?}", cmd.describe(), self.doc.text);
                self.redo_stack.push(cmd);
                true
            }
            None => {
                println!("nothing to undo");
                false
            }
        }
    }

    fn redo(&mut self) {
        if let Some(mut cmd) = self.redo_stack.pop() {
            cmd.execute(&mut self.doc);
            println!("redo {:<28} -> {:?}", cmd.describe(), self.doc.text);
            self.undo_stack.push(cmd);
        }
    }
}

// Closures as commands: a macro recorder.
struct Recorder {
    steps: Vec<(String, Box<dyn Fn(i64) -> i64>)>,
}

impl Recorder {
    fn record(&mut self, name: &str, f: impl Fn(i64) -> i64 + 'static) -> &mut Self {
        self.steps.push((name.to_string(), Box::new(f)));
        self
    }
    fn replay(&self, start: i64) -> i64 {
        self.steps.iter().fold(start, |acc, (name, f)| {
            let next = f(acc);
            println!("  {}: {} -> {}", name, acc, next);
            next
        })
    }
}

fn main() {
    let mut ed = Editor::new();
    ed.run(Box::new(Insert { pos: 0, text: "hello".into() }));
    ed.run(Box::new(Insert { pos: 5, text: " world".into() }));
    ed.run(Box::new(Delete { pos: 0, len: 6, removed: None }));
    ed.run(Box::new(Upper { before: String::new() }));
    ed.undo();
    ed.undo();
    ed.redo();
    ed.run(Box::new(Insert { pos: 5, text: "!".into() }));
    ed.redo();
    while ed.undo() {}
    println!("history: undo={} redo={}", ed.undo_stack.len(), ed.redo_stack.len());
    for c in ed.redo_stack.iter().rev() {
        println!("  pending {}", c.describe());
    }

    let mut rec = Recorder { steps: Vec::new() };
    let offset = 7;
    rec.record("double", |x| x * 2).record("add offset", move |x| x + offset).record("square", |x| x * x);
    println!("replay = {}", rec.replay(3));
    println!("replay = {}", rec.replay(-4));
}
