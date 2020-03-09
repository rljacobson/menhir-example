extern crate menhir_logos;
extern crate logos;
use menhir_logos::{MLLexer, LogosTokenType, Logos, LogosLexer, LexerError, SyntaxError};


use std::io::BufReader;
use std::fmt::{Debug, Formatter, Error, Display};
use std::fmt;

/// Convenience type to make AST types boxed.
pub type Expr = Box<ExprNode>;

#[derive(Debug)]
/// AST types
pub enum ExprNode {
  Var(String),
  App(Expr, Expr),
  Abs(String, Expr)
}

/// Menhir parser definitions.
mod parser {
  include!(concat!(env!("OUT_DIR"), "/parser.rs"));
  include!(concat!(env!("OUT_DIR"), "/parseerrors.rs"));
}

use parser::EntryPoint; // Required for parser::main::run()

/// Logos lexer definitions.
#[derive(Logos, Debug, PartialEq, Clone, Copy)]
enum LogosToken {
  #[end]
  End,
  #[error]
  Error,
  #[token = "lambda"]
  Lambda,
  #[token = "("]
  Open,
  #[token = ")"]
  Close,
  #[token = "."]
  Dot,
  #[regex = "[a-zA-Z_][a-zA-Z_0-9]*"]
  ID,
}

// ToDo: Need a macro to create the `map_to_parser_token` function at the same
// time it creates the lexer tokens.
impl LogosTokenType<parser::Token> for LogosToken{
  /// Convert from Logos::token to parser::YYType. This function does
  /// not call `lexer.advance()`.
  fn map_to_parser_token(lex: &LogosLexer<LogosToken, &str>)
                   -> parser::Token{
    let tok = lex.token;
    match tok {
      LogosToken::Lambda => parser::Token::LAMBDA,
      LogosToken::Open => parser::Token::OPEN,
      LogosToken::Close => parser::Token::CLOSE,
      LogosToken::Dot => parser::Token::DOT,
      LogosToken::ID => {
        parser::Token::ID(lex.slice().to_owned())
      },
      _ => parser::Token::EOF
    }
  }
}

fn main() {
  let text = "(lambda x.x) (lambda x.x)";
  let input = MLLexer::<LogosToken, parser::Token>::new(text);

  match parser::main::run(input) {
    Ok(term) => println!("Succesfully parsed: {:?}", term),
    Err(SyntaxError(loc, opt)) => {
      let msg = match opt {
        Some(t) => t,
        _ => "No message for this error."
      };
      println!("Syntax error at {}:{}:: {}", loc.start + 1, loc.end, msg)
    }
    Err(LexerError(err)) => println!("Lexer error: {}", err),
  }
}
