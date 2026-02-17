
use error::Error;
use pest::Parser;
use pest_derive::Parser;

use crate::ast::{builder::AstBuilder, AstProgram};
pub mod ast;
pub mod error;

#[derive(Parser)]
#[grammar = "../grammar/grammar.pest"]
pub struct QuantaParser;

pub fn parse_ast(source : &str) -> Result<AstProgram, Error> {
    let parsed_doc = QuantaParser::parse(Rule::document, source);
    let mut builder = AstBuilder::new();
    match parsed_doc {
        Ok(doc) => builder.build_ast_from_doc(doc),
        Err(err) => Err(Error::from_pest_error(err))
    }
}


#[cfg(test)]
mod tests {
    use std::{fs};

    use super::*;

    #[test]
    fn it_works() {
        let text = "func mouse(int x, int y) {
    setFigureColor(Color::Red);
    rectangle(x, y, x+100, y+100);
}

func keyboard(int key) {
    if (key == Key::Space) {
        setFigureColor(Color::Blue);
    } else {
      if (key == Key::As) {
          setFigureColor(Color::Black);
      } else {
          setFigureColor(Color::Yellow);
      }
    }
}

func main() {
   setLineColor(Color::Green);
   for i in (0..10000) {
      circle(320, 240, i % 100);
   }
   rectangle(0, 0, 100, 100);
}
";
        let wrong_text = "circle(320q, 240, 100);";
        assert!(parse_ast(text).is_ok());
        assert!(parse_ast(wrong_text).is_err());
    }

    #[test]
    fn test_file() {
        let file_path = "../grammar/test.txt";

        let contents = fs::read_to_string(file_path)
            .expect("Should have been able to read the file");
        assert!(contents.len() > 0);
        let res = parse_ast(contents.as_str());
        match &res {
            Ok(_ast) => {},
            Err(Error::ParseError{message}) => {println!("{}", message.to_string())}
            Err(Error::LogicError{message}) => {println!("{}", message.to_string())}
            Err(Error::TypeError{message}) => {println!("{}", message.to_string())}
            Err(Error::RuntimeError{message}) => {println!("{}", message.to_string())}
        }
        assert!(res.is_ok());
    }
}
