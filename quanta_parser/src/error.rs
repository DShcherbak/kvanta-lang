use std::{fmt};

use crate::Rule;

#[derive(Debug, Clone, PartialEq)]

pub enum ErrorType {
    ParseError,
    LogicError,
    TypeError,
    RuntimeError
}

#[derive(Debug)]
pub struct Error {
    error_type: ErrorType,
    message: String,
    start: (usize, usize),
    finish: (usize, usize)
}

impl Error {
    pub fn parse(mes: String, pos: (usize, usize, usize, usize)) -> Error {
        Error {
            error_type: ErrorType::ParseError,
            message: mes,
            start: (pos.0, pos.1),
            finish: (pos.2, pos.3)
        }
    }

    pub fn typeEr(mes: String, pos: (usize, usize, usize, usize)) -> Error {
        Error {
            error_type: ErrorType::TypeError,
            message: mes,
            start: (pos.0, pos.1),
            finish: (pos.2, pos.3)
        }
    }

    pub fn logic(mes: String, pos: (usize, usize, usize, usize)) -> Error {
        Error {
            error_type: ErrorType::LogicError,
            message: mes,
            start: (pos.0, pos.1),
            finish: (pos.2, pos.3)
        }
    }
}



impl Error {
    pub fn from_pest_error(err: pest::error::Error<Rule>) -> Error {
        let message = err.variant.message();
        let (l1, c1, l2, c2) = match err.line_col {
            pest::error::LineColLocation::Pos((line, col)) => (line, col, line, col),
            pest::error::LineColLocation::Span((l1, c1), (l2, c2)) => (l1, c1, l2, c2),
        };
        Error::parse(message.to_string(), (l1, c1, l2, c2))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let coords = format!("{}:{} - {}:{}", self.start.0, self.start.1, self.finish.0, self.finish.1);
        match &self.error_type {
            ErrorType::ParseError => {
                write!(f, "{}", format!("Got error during parsing: {} at {}", self.message, coords))
            },
            ErrorType::LogicError => {
                write!(f, "{}", format!("Got logical error: {} at {}", self.message, coords))
            },
            ErrorType::TypeError => {
                write!(f, "{}", format!("Got type checker error: {} at {}", self.message, coords))
            }
            ErrorType::RuntimeError=> {
                write!(f, "{}", format!("Got runtime error: {} at {}", self.message, coords))
            }
        }
        
    }
}