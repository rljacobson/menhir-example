extern crate menhir_runtime;
extern crate logos;

mod mllexer;

use menhir_runtime::ParserError::{SyntaxError, LexerError};
use menhir_runtime::{EntryPoint};
use mllexer::{MLLexer, LogosTokenType};

use logos::Logos;


use std::io::BufReader;
use std::fmt::{Debug, Formatter, Error, Display};
use std::fmt;

pub type Expr = Box<ExprNode>;

#[derive(Debug)]
pub enum ExprNode {
  Var(String),
  App(Expr, Expr),
  Abs(String, Expr)
}

mod parser {
  include!(concat!(env!("OUT_DIR"), "/parser.rs"));
  include!(concat!(env!("OUT_DIR"), "/parseerrors.rs"));
}

// Need a macro to create LogosToken to parser::Token at the same time it creates the lexer tokens.
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

impl LogosTokenType<parser::Token> for LogosToken{
  /// Convert from Logos::token to parser::YYType. This function does
  /// not call `lexer.advance()`.
  fn map_to_parser_token(lex: &logos::Lexer<LogosToken, &str>)
                   -> parser::Token{
    let tok = lex.token;
    let emitted = match tok {
      LogosToken::Lambda => parser::Token::LAMBDA,
      LogosToken::Open => parser::Token::OPEN,
      LogosToken::Close => parser::Token::CLOSE,
      LogosToken::Dot => parser::Token::DOT,
      LogosToken::ID => {
        parser::Token::ID(lex.slice().to_owned())
      },
      _ => parser::Token::EOF
    };
    emitted
  }
}

// Don't need this. Use crate derive-more on LogosToken.
impl Display for parser::Token{
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    let msg = match self {
      parser::Token::OPEN => "Token::OPEN",
      parser::Token::LAMBDA => "Token::LAMBDA",
      parser::Token::ID(data) => "Token::ID",
      parser::Token::EOF => "Token::EOF",
      parser::Token::DOT => "Token::DOT",
      parser::Token::CLOSE => "Token::CLOSE",
      _ => "UNKNOWN TOKEN"
    };
    write!(f, "{}", msg)
  }
}

impl Debug for parser::Token{
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self)
  }
}


fn main() {
  let text = "(lambda x.x) (lambda x.x)";
  let input = MLLexer::<LogosToken, parser::Token>::new(text);

  // let mut lexer = input.chain(::std::iter::once(EOF)).enumerate();
  // let adapter = IteratorLexer::new(&mut lexer);

  match parser::main::run(input) {
    Ok(term) => println!("Succesfully parsed: {:?}", term),
    Err(SyntaxError(loc, opt)) => {
      let msg = match opt {
        Some(t) => t,
        _ => "No message for this error."
      };
      println!("Syntax error at {}:{}:: {}", loc.start+1, loc.end, msg)
    }
    Err(LexerError(err)) => println!("Lexer error: {}", err),
  }
}
