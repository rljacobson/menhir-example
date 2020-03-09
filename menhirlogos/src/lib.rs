extern crate menhir_runtime;
extern crate logos;

use self::menhir_runtime::{IteratorLexer, Lexer as MenhirLexer, LRParser as MenhirParser};
pub use self::menhir_runtime::ParserError::{SyntaxError, LexerError};
pub use self::menhir_runtime::{EntryPoint};

pub use self::logos::{Logos, Lexer as LogosLexer};


use std::ops::Range;
use std::{fmt, error};
use std::marker::PhantomData;
use std::fmt::{Debug, Formatter, Error};


//region Error types
pub struct MLLexerError{
  range: Range<usize>
}

impl MLLexerError {
  pub fn new(range: Range<usize>) -> MLLexerError{
    MLLexerError{
      range
    }
  }
  pub fn at(&self) -> Range<usize>{
    return self.range.clone()
  }
}

impl fmt::Display for MLLexerError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let range = self.at();
    write!(f, "Lexer Error at {}:{}. Probably an IO error.", self.range.start, self.range.end-1)
  }
}

impl Debug for MLLexerError {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f,
           "MLLexerError{{start={},end={}}}",
           self.range.start,
           self.range.end-1
    )
  }
}

impl error::Error for MLLexerError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    None
  }
}
//endregion Error types

/// The Logos token enum implements this trait. This is how the Logos
/// token gets mapped back to the parser's `YYType`.
pub trait LogosTokenType<PTokenType>
  where Self: logos::Logos
{
  /// Takes a Logos token and gives the equivalen MenhirParser::Token.
  fn map_to_parser_token(lex: &logos::Lexer<Self, &str>) -> PTokenType;
}

/// The main struct of the module, MLLexer represents a lexer of a single source text.
// We could also parameterize the type of text with logos::source::Source.
// See https://docs.rs/logos/0.10.0-rc2/logos/source/trait.Source.html.
pub struct MLLexer<'a, LTokenType, PTokenType>
  where LTokenType: Logos + PartialEq
{
  lexer: LogosLexer<LTokenType, &'a str>,
  phantom: std::marker::PhantomData<PTokenType>
}

impl<'a, LTokenType, PTokenType> MLLexer<'a, LTokenType, PTokenType>
  where LTokenType: Logos + PartialEq
{
  pub fn new(text: &'a str) -> MLLexer<'a, LTokenType, PTokenType>
  {
    println!("New lexer for text `{}`.", text);
    MLLexer{
      lexer: LTokenType::lexer(text),
      phantom: PhantomData
    }
  }
}

impl<'a, LTokenType, PTokenType> MenhirLexer for MLLexer<'a, LTokenType, PTokenType>
  where LTokenType: Logos + PartialEq + LogosTokenType<PTokenType>
{
  // Location is not a (row,col)-coordinate but rather a [start,stop) range given as byte indices.
  type Location = Range<usize>;   // The location type of parser::Token.
  type Token = PTokenType; // The parser::Token type from the generated parser
  type Error = MLLexerError;

  fn input(&mut self) -> Result<(Range<usize>, PTokenType), Self::Error>{

    if self.lexer.token == LTokenType::ERROR {
      // ToDo: Do we advance on a lexer error?
      // ToDo: Figure out how to extract error states from Logos and make more detailed errors.
      return Err(
        MLLexerError::new(
          self.lexer.range()
        )
      )
    }
    let result = Ok(
                      ( // tuple
                        self.lexer.range() ,
                        LTokenType::map_to_parser_token(&self.lexer)
                      ) // end tuple
                    );
    // Advance to next token before returning.
    self.lexer.advance();
    result
  }
}
