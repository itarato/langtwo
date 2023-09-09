extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod ast;
mod interpreter;
mod lexer;
mod parser;
mod shared;
mod source_reader;

use crate::interpreter::*;
use crate::lexer::*;
use crate::parser::*;
use crate::source_reader::*;

fn main() {
    pretty_env_logger::init();

    info!("Start LangTwo");

    let reader = Box::new(StrReader::new(
        r#"
    fn sayhello(x) {
        print("Hello ");
        say(x);
    }

    fn say(word) {
        print(word);
    }

    fn printnum() {
        print(42);
    }

    sayhello("world");
    printnum();
"#,
    ));
    let lex_result = Lexer::new(reader).read_any();
    dbg!(&lex_result);

    let mut parser = Parser::new(lex_result.unwrap().into());
    let ast_root = parser.build_ast();
    dbg!(&ast_root);

    let mut interpreter = Interpreter::new();
    let interpret_result = interpreter.interpret(ast_root.unwrap());
    dbg!(&interpret_result);
}
