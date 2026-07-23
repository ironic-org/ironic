use convert_case::{Case, Casing};

use crate::CliError;

/// Normalized identifier forms for a generator name.
///
/// Converts user-supplied names into `snake_case`, `PascalCase`, and `kebab-case`
/// across all generator types.
#[derive(Debug)]
pub(crate) struct Names {
    pub(crate) snake: String,
    pub(crate) pascal: String,
    pub(crate) kebab: String,
}

impl Names {
    /// Parses a name from user input, rejecting keywords and digit-only names.
    ///
    /// Non-alphanumeric characters are treated as separators.
    ///
    /// # Errors
    ///
    /// Returns [`CliError::InvalidName`] when the name contains no alphabetic characters,
    /// resolves to a Rust keyword, or starts with a digit.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let names = example();
    /// assert_eq!(names.snake, "my_api");
    /// assert_eq!(names.pascal, "MyApi");
    /// assert_eq!(names.kebab, "my-api");
    /// ```
    pub(crate) fn parse(value: &str) -> Result<Self, CliError> {
        if !value.chars().any(char::is_alphabetic) {
            return Err(CliError::InvalidName {
                name: value.to_owned(),
            });
        }
        let safe_source = value
            .chars()
            .map(|character| {
                if character.is_alphanumeric() {
                    character
                } else {
                    ' '
                }
            })
            .collect::<String>();
        let snake = safe_source.to_case(Case::Snake);
        let pascal = safe_source.to_case(Case::Pascal);
        let kebab = safe_source.to_case(Case::Kebab);
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

#[cfg(test)]
mod tests {
    use crate::CliError;

    #[test]
    fn parse_simple_name() {
        let names = super::Names::parse("users").unwrap();
        assert_eq!(names.snake, "users");
        assert_eq!(names.pascal, "Users");
        assert_eq!(names.kebab, "users");
    }

    #[test]
    fn parse_multi_word() {
        let names = super::Names::parse("user_profile").unwrap();
        assert_eq!(names.snake, "user_profile");
        assert_eq!(names.pascal, "UserProfile");
        assert_eq!(names.kebab, "user-profile");
    }

    #[test]
    fn parse_camel_case_input() {
        let names = super::Names::parse("userProfile").unwrap();
        assert_eq!(names.snake, "user_profile");
        assert_eq!(names.pascal, "UserProfile");
    }

    #[test]
    fn parse_pascal_case_input() {
        let names = super::Names::parse("UserProfile").unwrap();
        assert_eq!(names.snake, "user_profile");
    }

    #[test]
    fn parse_with_special_chars() {
        let names = super::Names::parse("my.api-project").unwrap();
        assert_eq!(names.snake, "my_api_project");
        assert_eq!(names.pascal, "MyApiProject");
        assert_eq!(names.kebab, "my-api-project");
    }

    #[test]
    fn parse_with_underscores() {
        let names = super::Names::parse("my_api").unwrap();
        assert_eq!(names.snake, "my_api");
    }

    #[test]
    fn parse_rejects_digits_only() {
        let result = super::Names::parse("123");
        assert!(matches!(result, Err(CliError::InvalidName { name }) if name == "123"));
    }

    #[test]
    fn parse_rejects_empty() {
        let result = super::Names::parse("");
        assert!(matches!(result, Err(CliError::InvalidName { .. })));
    }

    #[test]
    fn parse_rejects_leading_digit() {
        let result = super::Names::parse("1user");
        assert!(matches!(result, Err(CliError::InvalidName { .. })));
    }

    #[test]
    fn parse_rejects_keywords() {
        let keywords = ["mod", "fn", "struct", "impl", "let", "match", "return", "use", "while", "for"];
        for kw in &keywords {
            let result = super::Names::parse(kw);
            assert!(
                result.is_err(),
                "keyword `{kw}` should be rejected"
            );
        }
    }

    #[test]
    fn is_keyword_true_for_all_keywords() {
        let keywords = [
            "as", "async", "await", "break", "const", "continue", "crate",
            "dyn", "else", "enum", "extern", "false", "fn", "for", "if",
            "impl", "in", "let", "loop", "match", "mod", "move", "mut",
            "pub", "ref", "return", "self", "static", "struct", "super",
            "trait", "true", "type", "unsafe", "use", "where", "while",
        ];
        for kw in &keywords {
            assert!(super::is_keyword(kw), "{kw} should be a keyword");
        }
    }

    #[test]
    fn is_keyword_false_for_non_keywords() {
        assert!(!super::is_keyword("my_variable"));
        assert!(!super::is_keyword("users"));
        assert!(!super::is_keyword("controller"));
        assert!(!super::is_keyword("MyModule"));
    }

    #[test]
    fn parse_normalizes_mixed_case() {
        let names = super::Names::parse("  spaced  name  ").unwrap();
        assert_eq!(names.snake, "__spaced__name__");
        assert_eq!(names.pascal, "SpacedName");
    }

    #[test]
    fn parse_with_numbers() {
        let names = super::Names::parse("api_v2").unwrap();
        assert_eq!(names.snake, "api_v_2");
    }

    #[test]
    fn parse_full_round_trip() {
        let input = "my-awesome-service";
        let names = super::Names::parse(input).unwrap();
        assert_eq!(names.snake, "my_awesome_service");
        assert_eq!(names.pascal, "MyAwesomeService");
        assert_eq!(names.kebab, "my-awesome-service");
    }
}
