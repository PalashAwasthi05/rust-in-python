// Chapter 10.2: traits, default implementations, trait bounds, blanket impls.
use std::fmt::{self, Debug, Display};

pub trait Summary {
    fn summarize_author(&self) -> String;

    fn summarize(&self) -> String {
        format!("(Read more from {}...)", self.summarize_author())
    }
}

pub struct NewsArticle {
    pub headline: String,
    pub location: String,
    pub author: String,
    pub content: String,
}

impl Summary for NewsArticle {
    fn summarize_author(&self) -> String {
        self.author.clone()
    }

    fn summarize(&self) -> String {
        format!("{}, by {} ({})", self.headline, self.author, self.location)
    }
}

pub struct Tweet {
    pub username: String,
    pub content: String,
    pub reply: bool,
    pub retweet: bool,
}

impl Summary for Tweet {
    fn summarize_author(&self) -> String {
        format!("@{}", self.username)
    }
}

impl Display for Tweet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.username, self.content)
    }
}

pub fn notify(item: &impl Summary) {
    println!("Breaking news! {}", item.summarize());
}

pub fn notify_both<T: Summary>(item1: &T, item2: &T) {
    println!("Both: {} / {}", item1.summarize(), item2.summarize());
}

pub fn notify_display(item: &(impl Summary + Display)) {
    println!("Displayed: {item} -- {}", item.summarize());
}

fn some_function<T, U>(t: &T, u: &U) -> String
where
    T: Display + Clone,
    U: Clone + Debug,
{
    format!("{} and {:?}", t.clone(), u.clone())
}

fn returns_summarizable() -> impl Summary {
    Tweet {
        username: String::from("horse_ebooks"),
        content: String::from("of course, as you probably already know, people"),
        reply: false,
        retweet: false,
    }
}

fn all_summaries(items: &[&dyn Summary]) -> Vec<String> {
    items.iter().map(|i| i.summarize()).collect()
}

struct Pair<T> {
    x: T,
    y: T,
}

impl<T> Pair<T> {
    fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: Display + PartialOrd> Pair<T> {
    fn cmp_display(&self) {
        if self.x >= self.y {
            println!("The largest member is x = {}", self.x);
        } else {
            println!("The largest member is y = {}", self.y);
        }
    }
}

trait Shout {
    fn shout(&self) -> String;
}

impl<T: Display> Shout for T {
    fn shout(&self) -> String {
        self.to_string().to_uppercase() + "!"
    }
}

trait Describe {
    fn describe(&self) -> String {
        String::from("an indescribable thing")
    }
}

struct Rock;
struct Cloud {
    altitude: u32,
}
impl Describe for Rock {}
impl Describe for Cloud {
    fn describe(&self) -> String {
        format!("a cloud at {}m", self.altitude)
    }
}

fn main() {
    let tweet = Tweet {
        username: String::from("horse_ebooks"),
        content: String::from("of course, as you probably already know, people"),
        reply: false,
        retweet: false,
    };
    println!("1 new tweet: {}", tweet.summarize());
    println!("reply={} retweet={}", tweet.reply, tweet.retweet);

    let article = NewsArticle {
        headline: String::from("Penguins win the Stanley Cup Championship!"),
        location: String::from("Pittsburgh, PA, USA"),
        author: String::from("Iceburgh"),
        content: String::from("The Pittsburgh Penguins once again are the best hockey team in the NHL."),
    };
    println!("New article available! {}", article.summarize());
    println!("content length {}", article.content.len());

    notify(&tweet);
    notify(&article);
    let tweet2 = Tweet {
        username: String::from("ferris"),
        content: String::from("click click"),
        reply: true,
        retweet: false,
    };
    notify_both(&tweet, &tweet2);
    notify_display(&tweet2);
    println!("{}", some_function(&"text", &vec![1, 2]));
    println!("{}", returns_summarizable().summarize());
    for s in all_summaries(&[&tweet, &article, &tweet2]) {
        println!("- {s}");
    }

    Pair::new(3, 7).cmp_display();
    Pair::new("zebra", "apple").cmp_display();
    Pair::new(2.5, 2.5).cmp_display();

    println!("{}", 42.shout());
    println!("{}", "hello".shout());
    println!("{}", tweet2.shout());
    let s = 3.to_string();
    println!("to_string via blanket Display impl: {s}");

    let things: Vec<Box<dyn Describe>> = vec![Box::new(Rock), Box::new(Cloud { altitude: 2000 })];
    for t in things.iter() {
        println!("{}", t.describe());
    }
}
