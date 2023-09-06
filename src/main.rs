mod lexer;
mod shared;
mod source_reader;

use crate::lexer::*;

fn main() {
    let lex_result = Lexer::parse(
        r#"
        fn sayhi() {
            print("Hi");
        }

        sayhi();
    "#,
    );
}
