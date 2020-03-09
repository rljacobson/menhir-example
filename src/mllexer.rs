extern crate menhir_runtime;
extern crate logos;

use menhir_runtime::{IteratorLexer, Lexer as MenhirLexer, LRParser as MenhirParser};
use menhir_runtime::ParserError::{LexerError as MenhirLexerError};
use logos::{Logos, Lexer as LogosLexer};

use std::ops::Range;
use std::{fmt, error};
use std::marker::PhantomData;

// ToDo: Make `use logos` not required in client code (reexport, etc.).
// ToDo: Refine error types.

//region Error types

/// Error type for this module.
#[derive(Debug)]
pub struct MLLexerError<'a> {
  range: Range<usize>,
  kind: &'a MLLexerErrorKind<'a>,
}

/// Kinds of errors for this error type,
pub enum MLLexerErrorKind<'a> {
  UnknownError, // A catch-all that shouldn't happen.
  IllegalCharacter(char),
  IllegalEscape(&'a str, Option<&'a str>),
  ReservedSequence(&'a str, Option<&'a str>),
  UnterminatedComment,
  UnterminatedString,
  //    UnterminatedStringInComment(Location, Location),
  KeywordAsLabel(&'a str),
  InvalidLiteral(&'a str),
  InvalidDirective(&'a str, Option<&'a str>),
  CommentStartWithEnd,
  CommentEndWithoutStart,
  // EscapeError(&'a ::unescape::EscapeError),
}

impl<'a> fmt::Debug for MLLexerErrorKind<'a> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self)
  }
}

impl<'a> fmt::Display for MLLexerErrorKind<'a>{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let msg = match self {
      MLLexerErrorKind::UnknownError => "An unexpected error occurred. This should not \
            happen.".to_owned(),
      MLLexerErrorKind::IllegalCharacter(char) => format!("Illegal character: `{}`.", char),
      MLLexerErrorKind::IllegalEscape(text, opt) => {
        let extra = match opt {
          Some(reason) => reason,
          None => ""
        };
        format!("Illegal escape: `{}`. {}", text, extra)
      },
      MLLexerErrorKind::ReservedSequence(text, opt) => {
        let extra = match opt {
          Some(reason) => reason,
          None => ""
        };
        format!("This sequence is reserved: `{}`. {}", text, extra)
      },
      MLLexerErrorKind::UnterminatedComment => "Unterminated comment, missing `*)`.".to_owned(),
      MLLexerErrorKind::UnterminatedString => "Unterminated string, missing `\"`.".to_owned(),
      // LexicalErrorKind::UnterminatedStringInComment(Location, Location),
      MLLexerErrorKind::KeywordAsLabel(text) =>
        format!("The keyword {} cannot be used as a label.", text),
      MLLexerErrorKind::InvalidLiteral(literal) =>
        format!("Invalid literal: `{}`.", literal),
      MLLexerErrorKind::InvalidDirective(text, opt) => {
        let extra = match opt {
          Some(reason) => reason,
          None => ""
        };
        format!("Invalid directive: `{}`. {}", text, extra)
      },
      MLLexerErrorKind::CommentStartWithEnd =>
        format!("Comment beginning and end are combined: `(*)`."),
      MLLexerErrorKind::CommentEndWithoutStart =>
        format!("Comment ended, but no comment was begun. (Missing `(*`.)"),
      // MLLexerErrorKind::EscapeError(EscapeError) =>
      //   format!("{}", EscapeError)
    };
    write!(f, "{}", msg)
  }
}

impl<'a> MLLexerError<'a>{
  pub fn new(range: Range<usize>, kind: &'a MLLexerErrorKind<'a>) -> MLLexerError<'a>{
    MLLexerError{
      range,
      kind
    }
  }
  pub fn at(&self) -> Range<usize>{
    return self.range.clone()
  }
  pub fn kind(&self) -> &MLLexerErrorKind {
    return self.kind
  }
}

impl<'a> fmt::Display for MLLexerError<'a>{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let range = self.at();
    write!(f, "{}:{}: error : {}", self.range.start, self.range.end-1, self.kind)
  }
}

impl<'a> error::Error for MLLexerError<'a> {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    // match *self.kind(){
    //   MLLexerErrorKind::EscapeError(ref e) => Some(e),
    //   _ => None
    // }
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
  type Error = MLLexerError<'a>;

  fn input(&mut self) -> Result<(Range<usize>, PTokenType), Self::Error>{

    if self.lexer.token == LTokenType::ERROR {
      // ToDo: Do we advance on a lexer error?
      // ToDo: Figure out how to extract error states from Logos
      return Err(
        MLLexerError::new(
          self.lexer.range(),
          &MLLexerErrorKind::UnknownError
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
