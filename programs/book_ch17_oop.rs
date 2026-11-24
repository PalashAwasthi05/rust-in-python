// Chapter 17: encapsulation, trait objects (gui Draw), and the blog Post state pattern two ways.
pub struct AveragedCollection {
    list: Vec<i32>,
    average: f64,
}

impl AveragedCollection {
    pub fn new() -> Self {
        AveragedCollection { list: vec![], average: 0.0 }
    }

    pub fn add(&mut self, value: i32) {
        self.list.push(value);
        self.update_average();
    }

    pub fn remove(&mut self) -> Option<i32> {
        let result = self.list.pop();
        match result {
            Some(value) => {
                self.update_average();
                Some(value)
            }
            None => None,
        }
    }

    pub fn average(&self) -> f64 {
        self.average
    }

    fn update_average(&mut self) {
        let total: i32 = self.list.iter().sum();
        self.average = if self.list.is_empty() { 0.0 } else { total as f64 / self.list.len() as f64 };
    }
}

pub trait Draw {
    fn draw(&self) -> String;
}

pub struct Screen {
    pub components: Vec<Box<dyn Draw>>,
}

impl Screen {
    pub fn run(&self) {
        for component in self.components.iter() {
            println!("{}", component.draw());
        }
    }
}

pub struct Button {
    pub width: u32,
    pub height: u32,
    pub label: String,
}

impl Draw for Button {
    fn draw(&self) -> String {
        format!("[Button {}x{}: {}]", self.width, self.height, self.label)
    }
}

struct SelectBox {
    width: u32,
    height: u32,
    options: Vec<String>,
}

impl Draw for SelectBox {
    fn draw(&self) -> String {
        format!("<SelectBox {}x{} options={}>", self.width, self.height, self.options.join("|"))
    }
}

pub struct GenericScreen<T: Draw> {
    pub components: Vec<T>,
}

impl<T: Draw> GenericScreen<T> {
    pub fn run(&self) -> usize {
        self.components.iter().map(|c| c.draw().len()).sum()
    }
}

mod blog {
    pub struct Post {
        state: Option<Box<dyn State>>,
        content: String,
        approvals: u32,
    }

    impl Post {
        pub fn new() -> Post {
            Post {
                state: Some(Box::new(Draft {})),
                content: String::new(),
                approvals: 0,
            }
        }

        pub fn add_text(&mut self, text: &str) {
            if self.state.as_ref().unwrap().allows_editing() {
                self.content.push_str(text);
            }
        }

        pub fn content(&self) -> &str {
            self.state.as_ref().unwrap().content(self)
        }

        pub fn status(&self) -> &'static str {
            self.state.as_ref().unwrap().name()
        }

        pub fn request_review(&mut self) {
            if let Some(s) = self.state.take() {
                self.state = Some(s.request_review())
            }
        }

        pub fn reject(&mut self) {
            if let Some(s) = self.state.take() {
                self.approvals = 0;
                self.state = Some(s.reject())
            }
        }

        pub fn approve(&mut self) {
            if let Some(s) = self.state.take() {
                self.approvals += 1;
                let needed = self.approvals;
                self.state = Some(s.approve(needed))
            }
        }
    }

    trait State {
        fn request_review(self: Box<Self>) -> Box<dyn State>;
        fn approve(self: Box<Self>, approvals: u32) -> Box<dyn State>;
        fn reject(self: Box<Self>) -> Box<dyn State>;
        fn name(&self) -> &'static str;
        fn content<'a>(&self, _post: &'a Post) -> &'a str {
            ""
        }
        fn allows_editing(&self) -> bool {
            false
        }
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
        fn name(&self) -> &'static str {
            "draft"
        }
        fn allows_editing(&self) -> bool {
            true
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
            "pending review"
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
        fn name(&self) -> &'static str {
            "published"
        }
        fn content<'a>(&self, post: &'a Post) -> &'a str {
            &post.content
        }
    }
}

mod typed_blog {
    pub struct Post {
        content: String,
    }

    pub struct DraftPost {
        content: String,
    }

    pub struct PendingReviewPost {
        content: String,
    }

    impl Post {
        pub fn new() -> DraftPost {
            DraftPost { content: String::new() }
        }

        pub fn content(&self) -> &str {
            &self.content
        }
    }

    impl DraftPost {
        pub fn add_text(&mut self, text: &str) {
            self.content.push_str(text);
        }

        pub fn request_review(self) -> PendingReviewPost {
            PendingReviewPost { content: self.content }
        }
    }

    impl PendingReviewPost {
        pub fn approve(self) -> Post {
            Post { content: self.content }
        }

        pub fn reject(self) -> DraftPost {
            DraftPost { content: self.content }
        }
    }
}

fn main() {
    let mut c = AveragedCollection::new();
    for v in [4, 8, 15, 16, 23, 42] {
        c.add(v);
    }
    println!("average {:.3}", c.average());
    println!("removed {:?}, average now {}", c.remove(), c.average());
    let mut empty = AveragedCollection::new();
    println!("empty remove {:?} average {}", empty.remove(), empty.average());

    let screen = Screen {
        components: vec![
            Box::new(SelectBox {
                width: 75,
                height: 10,
                options: vec![String::from("Yes"), String::from("Maybe"), String::from("No")],
            }),
            Box::new(Button {
                width: 50,
                height: 10,
                label: String::from("OK"),
            }),
        ],
    };
    screen.run();
    let buttons = GenericScreen {
        components: vec![
            Button { width: 1, height: 1, label: "a".into() },
            Button { width: 2, height: 2, label: "bb".into() },
        ],
    };
    println!("generic screen drew {} chars", buttons.run());

    let mut post = blog::Post::new();
    post.add_text("I ate a salad for lunch today");
    println!("[{}] content: {:?}", post.status(), post.content());
    post.request_review();
    post.add_text(" (edited during review)");
    println!("[{}] content: {:?}", post.status(), post.content());
    post.reject();
    println!("[{}] content: {:?}", post.status(), post.content());
    post.request_review();
    post.approve();
    println!("[{}] content: {:?}", post.status(), post.content());
    post.approve();
    println!("[{}] content: {:?}", post.status(), post.content());

    let mut post = typed_blog::Post::new();
    post.add_text("I ate a salad for lunch today");
    let post = post.request_review();
    let mut post = post.reject();
    post.add_text(", and soup");
    let post = post.request_review().approve();
    println!("typed post: {}", post.content());
}
