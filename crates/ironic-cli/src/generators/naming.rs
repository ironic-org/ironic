use convert_case::{Case, Casing};

use crate::CliError;

#[derive(Debug)]
pub(crate) struct Names {
    pub(crate) snake: String,
    pub(crate) pascal: String,
    pub(crate) kebab: String,
}

impl Names {
    pub(crate) fn parse(value: &str) -> Result<Self, CliError> {
        if !value.chars().any(char::is_alphabetic) {
            return Err(CliError::InvalidName {
                name: value.to_owned(),
            });
        }
        let snake = value.to_case(Case::Snake);
        let pascal = value.to_case(Case::Pascal);
        let kebab = value.to_case(Case::Kebab);
        if snake.is_empty()
            || pascal.is_empty()
            || kebab.is_empty()
            || is_keyword(&snake)
            || snake.starts_with(|character: char| character.is_ascii_digit())
        {
            return Err(CliError::InvalidName {
                name: value.to_owned(),
            });
        }
        Ok(Self {
            snake,
            pascal,
            kebab,
        })
    }
}

fn is_keyword(value: &str) -> bool {
    matches!(
        value,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
    )
}
