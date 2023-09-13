extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod ast;
mod interpreter;
mod lexer;
mod parser;
mod shared;
mod source_reader;

use crate::ast::AstDump;
use crate::interpreter::*;
use crate::lexer::*;
use crate::parser::*;
use crate::source_reader::*;

fn main() {
    pretty_env_logger::init();

    info!("Start LangTwo");

    let reader = Box::new(StrReader::new(
        r#"
        fn fizzbuzz(i, limit) {
            print(i);
            print(" ");
            if (i % 3 == 0) {
                print("fizz");
            } else {}
            if (i % 5 == 0) {
                print("buzz");
            } else {}
            print(" ");

            if (i < limit) {
                fizzbuzz(i + 1, limit);
            } else {
                i;
            }
        }

        fizzbuzz(1, 100);
"#,
    ));
    let lex_result = Lexer::new(reader).read_any();
    dbg!(&lex_result);

    let mut parser = Parser::new(lex_result.unwrap().into());
    let ast_root = parser.build_ast().unwrap();
    dbg!(&ast_root);

    println!("---\n\n{}\n\n---", ast_root.ast_dump(0));

    let mut interpreter = Interpreter::new();
    let interpret_result = interpreter.interpret(ast_root);
    dbg!(&interpret_result);
}
