pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub fn char_n(c: char, n: usize) -> String {
    let mut out = String::new();

    for _ in 0..n {
        out.push(c);
    }

    out
}
