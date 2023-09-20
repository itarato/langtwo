extern crate pretty_env_logger;
#[macro_use]
extern crate log;

mod ast;
mod interpreter;
mod ir;
mod ir_builder;
mod lexer;
mod parser;
mod shared;
mod source_reader;
mod vm;

use crate::ast::AstDump;
use crate::interpreter::*;
use crate::ir_builder::*;
use crate::lexer::*;
use crate::parser::*;
use crate::source_reader::*;

fn print_help_and_exit() {
    panic!("Call: `./bin interpret` or `./bin ir`");
}

fn interpret_example() {
    let reader = Box::new(StrReader::new(
        r#"
fn fizzbuzz(i, limit) {
    print(i);
    print(" ");
    if (i % 3 == 0) {
        print("fizz");
    }
    if (i % 5 == 0) {
        print("buzz");
    }
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

fn ir_example() {
    let reader = Box::new(StrReader::new(
        r#"
        1 + 2;
"#,
    ));
    let lex_result = Lexer::new(reader).read_any();
    dbg!(&lex_result);

    let mut parser = Parser::new(lex_result.unwrap().into());
    let ast_root = parser.build_ast().unwrap();
    dbg!(&ast_root);

    println!("---\n\n{}\n\n---", ast_root.ast_dump(0));

    let mut ir = IRBuilder::new();
    let ir_result = ir.build(ast_root);
    dbg!(&ir_result);
}

fn main() {
    pretty_env_logger::init();
    info!("Start LangTwo");

    let args = std::env::args();
    if args.len() != 2 {
        eprintln!("Expected 1 argument.");
        print_help_and_exit();
    }

    match args.into_iter().last().unwrap().as_str() {
        "interpret" => interpret_example(),
        "ir" => ir_example(),
        _ => print_help_and_exit(),
    };
}
