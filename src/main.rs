#![feature(plugin,main)]
#![plugin(rustlex)]
#[allow(plugin_as_library)]
extern crate rustlex;
extern crate menhir_runtime;

use std::io::BufReader;

pub type Expr = Box<ExprNode>;

#[derive(Debug)]
pub enum ExprNode {
  Var(String),
  App(Expr, Expr),
  Abs(String, Expr)
}

mod parser {
  include!(concat!(env!("OUT_DIR"), "/parser.rs"));
  include!(concat!(env!("OUT_DIR"), "/parseerror.rs"));
}

use parser::Token;
use parser::Token::*;
use menhir_runtime::IteratorLexer;
use menhir_runtime::ParserError::{SyntaxError, LexerError};

rustlex! Lexer {
    let ID = ['a'-'z''A'-'Z''_']['a'-'z''A'-'Z''_''0'-'9']*;

    .        => |_:&mut Lexer<R>| None
    ID       => |lexer:&mut Lexer<R>| Some(ID(lexer.yystr()))
    "lambda" => |_:&mut Lexer<R>| Some(LAMBDA)
    "("      => |_:&mut Lexer<R>| Some(OPEN)
    ")"      => |_:&mut Lexer<R>| Some(CLOSE)
    "."      => |_:&mut Lexer<R>| Some(DOT)
}

fn main() {
  let text = "(lambda x.x) (lambda x.x)";
  let input = Lexer::new(BufReader::new(text.as_bytes()));

  let mut lexer = input.chain(::std::iter::once(EOF)).enumerate();

  let adapter = IteratorLexer::new(&mut lexer);

  match parser::main::run(adapter) {
    Ok(term) => println!("succesfully parsed: {:?}", term),
    Err(SyntaxError(loc, opt)) => {
      let msg = match opt {
        Some(t) => t,
        _ => "No message for this error."
      };
      println!("Syntax error at {}: {}", loc, msg)
    }
    Err(LexerError(err)) => println!("Lexer error: {:?}", err),
  }
}
