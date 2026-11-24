// The Rust book's blog post state pattern with Option<Box<dyn State>> and self: Box<Self>.
pub struct Post {
    state: Option<Box<dyn State>>,
    content: String,
    approvals: u32,
}

impl Post {
    pub fn new() -> Post {
        Post { state: Some(Box::new(Draft {})), content: String::new(), approvals: 0 }
    }

    pub fn add_text(&mut self, text: &str) {
        if self.state.as_ref().unwrap().can_edit() {
            self.content.push_str(text);
        }
    }

    pub fn content(&self) -> &str {
        self.state.as_ref().unwrap().content(self)
    }

    pub fn status(&self) -> String {
        self.state.as_ref().map(|s| s.name()).unwrap_or("none").to_string()
    }

    pub fn request_review(&mut self) {
        if let Some(s) = self.state.take() {
            self.state = Some(s.request_review())
        }
    }

    pub fn reject(&mut self) {
        if let Some(s) = self.state.take() {
            self.state = Some(s.reject())
        }
    }

    pub fn approve(&mut self) {
        self.approvals += 1;
        let approvals = self.approvals;
        if let Some(s) = self.state.take() {
            self.state = Some(s.approve(approvals))
        }
    }
}

trait State {
    fn request_review(self: Box<Self>) -> Box<dyn State>;
    fn approve(self: Box<Self>, approvals: u32) -> Box<dyn State>;
    fn reject(self: Box<Self>) -> Box<dyn State>;
    fn content<'a>(&self, _post: &'a Post) -> &'a str {
        ""
    }
    fn can_edit(&self) -> bool {
        false
    }
    fn name(&self) -> &'static str;
}

struct Draft {}

impl State for Draft {
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        Box::new(PendingReview {})
    }
    fn approve(self: Box<Self>, _approvals: u32) -> Box<dyn State> {
        self
    }
    fn reject(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn can_edit(&self) -> bool {
        true
    }
    fn name(&self) -> &'static str {
        "draft"
    }
}

struct PendingReview {}

impl State for PendingReview {
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn approve(self: Box<Self>, approvals: u32) -> Box<dyn State> {
        if approvals >= 2 {
            Box::new(Published {})
        } else {
            self
        }
    }
    fn reject(self: Box<Self>) -> Box<dyn State> {
        Box::new(Draft {})
    }
    fn name(&self) -> &'static str {
        "pending"
    }
}

struct Published {}

impl State for Published {
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn approve(self: Box<Self>, _approvals: u32) -> Box<dyn State> {
        self
    }
    fn reject(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn content<'a>(&self, post: &'a Post) -> &'a str {
        &post.content
    }
    fn name(&self) -> &'static str {
        "published"
    }
}

// The type-encoded alternative from the same chapter.
struct DraftPost {
    content: String,
}
struct PendingReviewPost {
    content: String,
}
struct PublishedPost {
    content: String,
}

impl DraftPost {
    fn new() -> DraftPost {
        DraftPost { content: String::new() }
    }
    fn add_text(&mut self, text: &str) {
        self.content.push_str(text);
    }
    fn request_review(self) -> PendingReviewPost {
        PendingReviewPost { content: self.content }
    }
}

impl PendingReviewPost {
    fn approve(self) -> PublishedPost {
        PublishedPost { content: self.content }
    }
}

impl PublishedPost {
    fn content(&self) -> &str {
        &self.content
    }
}

fn main() {
    let mut post = Post::new();
    post.add_text("I ate a salad for lunch today");
    println!("[{}] {:?}", post.status(), post.content());

    post.request_review();
    post.add_text(" -- edited while pending");
    println!("[{}] {:?}", post.status(), post.content());

    post.reject();
    println!("[{}] {:?}", post.status(), post.content());
    post.add_text("!");

    post.request_review();
    post.approve();
    println!("[{}] {:?} approvals={}", post.status(), post.content(), post.approvals);
    post.approve();
    println!("[{}] {:?} approvals={}", post.status(), post.content(), post.approvals);
    post.reject();
    println!("[{}] {:?}", post.status(), post.content());

    let mut d = DraftPost::new();
    d.add_text("typed states");
    let p = d.request_review().approve();
    println!("typed: {}", p.content());

    let mut posts: Vec<Post> = (0..3).map(|_| Post::new()).collect();
    for (i, p) in posts.iter_mut().enumerate() {
        p.add_text(&format!("post {}", i));
        for _ in 0..i {
            p.request_review();
            p.approve();
        }
    }
    for p in &posts {
        println!("{} -> {:?}", p.status(), p.content());
    }
}
