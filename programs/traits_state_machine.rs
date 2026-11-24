// A Rust-book style blog post workflow with trait objects that consume Box<Self>.
trait State {
    fn request_review(self: Box<Self>) -> Box<dyn State>;
    fn approve(self: Box<Self>) -> Box<dyn State>;
    fn reject(self: Box<Self>) -> Box<dyn State>;
    fn content<'a>(&self, _post: &'a Post) -> &'a str {
        ""
    }
    fn label(&self) -> String;
}

struct Draft;
struct Pending {
    approvals: u32,
}
struct Published;

impl State for Draft {
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        Box::new(Pending { approvals: 0 })
    }
    fn approve(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn reject(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn label(&self) -> String {
        "draft".into()
    }
}

impl State for Pending {
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn approve(self: Box<Self>) -> Box<dyn State> {
        if self.approvals + 1 >= 2 {
            Box::new(Published)
        } else {
            Box::new(Pending { approvals: self.approvals + 1 })
        }
    }
    fn reject(self: Box<Self>) -> Box<dyn State> {
        Box::new(Draft)
    }
    fn label(&self) -> String {
        format!("pending({})", self.approvals)
    }
}

impl State for Published {
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn approve(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn reject(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn content<'a>(&self, post: &'a Post) -> &'a str {
        &post.text
    }
    fn label(&self) -> String {
        String::from("published")
    }
}

pub struct Post {
    state: Option<Box<dyn State>>,
    text: String,
}

impl Post {
    fn new() -> Post {
        Post { state: Some(Box::new(Draft)), text: String::new() }
    }
    fn add_text(&mut self, t: &str) {
        self.text.push_str(t);
    }
    fn content(&self) -> &str {
        self.state.as_ref().unwrap().content(self)
    }
    fn step(&mut self, action: &str) {
        if let Some(s) = self.state.take() {
            self.state = Some(match action {
                "review" => s.request_review(),
                "approve" => s.approve(),
                "reject" => s.reject(),
                _ => s,
            });
        }
        println!("{:>8} -> {:<12} content={:?}", action, self.state.as_ref().unwrap().label(), self.content());
    }
}

fn main() {
    let mut post = Post::new();
    post.add_text("I ate a salad for lunch today");
    for a in ["approve", "review", "approve", "reject", "review", "approve", "approve", "reject"] {
        post.step(a);
    }
    let states: Vec<Box<dyn State>> = vec![Box::new(Draft), Box::new(Pending { approvals: 1 }), Box::new(Published)];
    let labels: Vec<String> = states.into_iter().map(|s| s.approve().label()).collect();
    println!("{:?}", labels);
}
