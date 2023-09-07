mod lexer;
mod shared;
mod source_reader;

use crate::lexer::*;
use crate::source_reader::*;

fn main() {
    let reader = Box::new(StrReader::new(
        r#"
    fn sayhi() {
        print("Hi");
    }

    sayhi();
"#,
    ));
    let lex_result = Lexer::new(reader).read_any();
}
