use std::collections::HashMap;
use std::collections::HashSet;

use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Pipe,      // > then
    Union,     // | or
    Cat,       // + cat
    Intersect, // & and
    Diff,      // - minus
}

impl Op {

    // higher is better
    fn precedence(self) -> u8 {
        match self {
            Op::Pipe => 1,
            Op::Union | Op::Cat | Op::Diff => 2,
            Op::Intersect => 3,
        }
    }

    fn from_word(word: &str) -> Option<Op> {
        match word {
            "then" => Some(Op::Pipe),
            "or" => Some(Op::Union),
            "cat" => Some(Op::Cat),
            "and" => Some(Op::Intersect),
            "minus" | "-" => Some(Op::Diff),
            _ => None,
        }
    }

     // - is only an operator as a token by itself
    fn from_symbol(ch: char) -> Option<Op> {
        match ch {
            '>' => Some(Op::Pipe),
            '|' => Some(Op::Union),
            '+' => Some(Op::Cat),
            '&' => Some(Op::Intersect),
            _ => None,
        }
    }
}

impl std::fmt::Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            Op::Pipe => ">",
            Op::Union => "|",
            Op::Cat => "+",
            Op::Intersect => "&",
            Op::Diff => "-",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arg {
    pub text: String,
    pub quoted: bool,
}

impl Arg {
    pub fn is_flag(&self) -> bool {
        return !self.quoted && self.text.len() > 1 && self.text.starts_with('-');
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandNode {
    pub name: String,
    pub args: Vec<Arg>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Command(CommandNode),
    Binary(Op, Box<Expr>, Box<Expr>),
}

#[derive(Debug)]
pub enum ParseError {
    MismatchedParentheses,
    UnexpectedEndOfInput,
    UnknownCommand(String),
    General(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParseError::MismatchedParentheses => write!(f, "Mismatched parentheses"),
            ParseError::UnexpectedEndOfInput => write!(f, "No command given"),
            ParseError::UnknownCommand(name) => write!(f, "Unknown command `{}`", name),
            ParseError::General(msg) => write!(f, "{}", msg),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Word(Arg),
    Op(Op),
    LParen,
    RParen,
}

pub fn verify_flags(
    expr: &Expr,
    allowed_flags: &HashMap<String, Vec<String>>,
) -> Result<(), ParseError> {
    match expr {
        Expr::Binary(_, lhs, rhs) => {
            verify_flags(lhs, allowed_flags)?;
            verify_flags(rhs, allowed_flags)
        }
        Expr::Command(cmd) => {
            for arg in cmd.args.iter().filter(|a| a.is_flag()) {
                if !allowed_flags
                    .get(&cmd.name)
                    .is_some_and(|v| v.contains(&arg.text))
                {
                    return Err(ParseError::General(format!(
                        "Flag {} cannot be used as an argument for {}",
                        arg.text, cmd.name
                    )));
                }
            }
            Ok(())
        }
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    let mut result = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            c if c.is_whitespace() => {
                // Skip whitespace
                chars.next();
            }
            '(' => {
                result.push(Token::LParen);
                chars.next();
            }
            ')' => {
                result.push(Token::RParen);
                chars.next();
            }
            '"' => {
                // Read a quoted string
                result.push(Token::Word(Arg {
                    text: read_quoted(&mut chars)?,
                    quoted: true,
                }));
            }
            _ => {
                if let Some(op) = Op::from_symbol(ch) {
                    result.push(Token::Op(op));
                    chars.next();
                    continue;
                }
                // unquoted word or word operator
                let text = read_unquoted(&mut chars);
                match Op::from_word(&text) {
                    Some(op) => result.push(Token::Op(op)),
                    None => result.push(Token::Word(Arg {
                        text,
                        quoted: false,
                    })),
                }
            }
        }
    }

    Ok(result)
}

fn read_quoted(chars: &mut Peekable<Chars>) -> Result<String, ParseError> {
    chars.next();

    let mut result = String::new();
    for ch in chars.by_ref() {
        match ch {
            '"' => return Ok(result),
            _ => result.push(ch),
        }
    }

    Err(ParseError::General("Unclosed double quote".to_string()))
}

fn read_unquoted(chars: &mut Peekable<Chars>) -> String {
    let mut result = String::new();
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace()
            || ch == '('
            || ch == ')'
            || ch == '"'
            || Op::from_symbol(ch).is_some()
        {
            break;
        }
        result.push(ch);
        chars.next();
    }
    return result;
}

pub fn parse(tokens: &[Token], command_list: &HashSet<String>) -> Result<Expr, ParseError> {
    if tokens.is_empty() {
        return Err(ParseError::UnexpectedEndOfInput);
    }

    let mut parser = Parser {
        tokens,
        pos: 0,
        command_list,
    };
    let expr = parser.parse_expr(1)?;

    if parser.pos < tokens.len() {
        return Err(ParseError::MismatchedParentheses);
    }

    return Ok(expr);
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    command_list: &'a HashSet<String>,
}

impl Parser<'_> {
    fn parse_expr(&mut self, min_prec: u8) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_primary()?;

        while let Some(Token::Op(op)) = self.tokens.get(self.pos) {
            let op = *op;
            if op.precedence() < min_prec {
                break;
            }
            self.pos += 1;
            if self.pos >= self.tokens.len() {
                return Err(ParseError::General(format!(
                    "Expected a command after `{}`",
                    op
                )));
            }
            let rhs = self.parse_expr(op.precedence() + 1)?;
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs));
        }

        return Ok(lhs);
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.tokens.get(self.pos) {
            None => Err(ParseError::UnexpectedEndOfInput),
            Some(Token::LParen) => {
                self.pos += 1;
                let expr = self.parse_expr(1)?;
                if self.tokens.get(self.pos) != Some(&Token::RParen) {
                    return Err(ParseError::MismatchedParentheses);
                }
                self.pos += 1;
                Ok(expr)
            }
            Some(Token::RParen) => Err(ParseError::MismatchedParentheses),
            Some(Token::Op(op)) => Err(ParseError::General(format!(
                "Expected a command before `{}`",
                op
            ))),
            Some(Token::Word(word)) => {
                if word.quoted || !self.command_list.contains(&word.text) {
                    return Err(ParseError::UnknownCommand(word.text.clone()));
                }
                self.pos += 1;

                let mut args = Vec::new();
                while let Some(Token::Word(arg)) = self.tokens.get(self.pos) {
                    args.push(arg.clone());
                    self.pos += 1;
                }
                if self.tokens.get(self.pos) == Some(&Token::LParen) {
                    return Err(ParseError::General(format!(
                        "Unexpected `(` in the arguments of {}",
                        word.text
                    )));
                }

                Ok(Expr::Command(CommandNode {
                    name: word.text.clone(),
                    args,
                }))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flags() -> HashMap<String, Vec<String>> {
        let f = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        return HashMap::from([
            ("list".to_string(), f(&["-i", "-h"])),
            ("sort".to_string(), f(&["-i", "-h", "--desc", "--by"])),
            ("unique".to_string(), f(&["-i", "-h"])),
            ("filter".to_string(), f(&["-i", "-h", "--tempo"])),
            ("now".to_string(), f(&["-i", "-h"])),
            ("queue_add".to_string(), f(&[])),
            ("create_playlist".to_string(), f(&["--name"])),
            ("device".to_string(), f(&[])),
        ]);
    }

    // s-expression form
    fn sexpr(expr: &Expr) -> String {
        match expr {
            Expr::Command(cmd) => {
                let mut parts = vec![cmd.name.clone()];
                parts.extend(cmd.args.iter().map(|a| match a.quoted {
                    true => format!("\"{}\"", a.text),
                    false => a.text.clone(),
                }));
                format!("({})", parts.join(" "))
            }
            Expr::Binary(op, lhs, rhs) => format!("({} {} {})", op, sexpr(lhs), sexpr(rhs)),
        }
    }

    fn parsed(input: &str) -> Result<String, ParseError> {
        let commands: HashSet<String> = flags().into_keys().collect();
        let expr = parse(&tokenize(input)?, &commands)?;
        verify_flags(&expr, &flags())?;
        return Ok(sexpr(&expr));
    }

    fn check(input: &str, expected: &str) {
        assert_eq!(parsed(input).unwrap(), expected, "input: {}", input);
    }

    #[test]
    fn design_examples() {
        check(
            "list P > sort --desc > unique",
            "(> (> (list P) (sort --desc)) (unique))",
        );
        check(
            "list P > list > queue_add",
            "(> (> (list P) (list)) (queue_add))",
        );
        check("list S1 S2 S3", "(list S1 S2 S3)");
        check(
            "sort T1 T2 T3 T1 --by artist > unique",
            "(> (sort T1 T2 T3 T1 --by artist) (unique))",
        );
        check("now > queue_add", "(> (now) (queue_add))");
        check("list P1 | list P2", "(| (list P1) (list P2))");
        check("list P1 + list P2", "(+ (list P1) (list P2))");
        check("list P1 - list P2", "(- (list P1) (list P2))");
        check(
            "list P1 & list P2 > create_playlist --name new_pl",
            "(> (& (list P1) (list P2)) (create_playlist --name new_pl))",
        );
        check(
            "list P > filter --tempo 50..",
            "(> (list P) (filter --tempo 50..))",
        );
        check(
            "(list P1 | list P2) & list P3",
            "(& (| (list P1) (list P2)) (list P3))",
        );
    }

    #[test]
    fn word_aliases_match_symbols() {
        let pairs = [
            (
                "list P then sort --desc then unique",
                "list P > sort --desc > unique",
            ),
            (
                "list A and list B then queue_add",
                "list A & list B > queue_add",
            ),
            ("list A or list B", "list A | list B"),
            ("list A cat list B", "list A + list B"),
            ("list A minus list B", "list A - list B"),
        ];
        for (words, symbols) in pairs {
            assert_eq!(parsed(words).unwrap(), parsed(symbols).unwrap());
        }
    }

    #[test]
    fn precedence_and_associativity() {
        check("now | now & now", "(| (now) (& (now) (now)))");
        check("now & now | now", "(| (& (now) (now)) (now))");
        check("now | now > now", "(> (| (now) (now)) (now))");
        check("now + now - now", "(- (+ (now) (now)) (now))");
        check("now - now + now", "(+ (- (now) (now)) (now))");
        check("now & (now | now)", "(& (now) (| (now) (now)))");
    }

    #[test]
    fn quotes_flags_and_symbols() {
        // quoted operators and flags are plain text
        check(
            "create_playlist --name \"and\" \"-h\" \"a > b\"",
            "(create_playlist --name \"and\" \"-h\" \"a > b\")",
        );
        check("list A -i - list B", "(- (list A -i) (list B))");
        check("list A>sort --desc", "(> (list A) (sort --desc))");
        check("device Living Room", "(device Living Room)");
    }

    #[test]
    fn parse_errors() {
        let err = |input: &str| parsed(input).unwrap_err();
        assert!(matches!(err(""), ParseError::UnexpectedEndOfInput));
        assert!(matches!(err("foo > now"), ParseError::UnknownCommand(n) if n == "foo"));
        assert!(matches!(err("\"now\""), ParseError::UnknownCommand(_)));
        assert!(matches!(
            err("(now | now"),
            ParseError::MismatchedParentheses
        ));
        assert!(matches!(err("now )"), ParseError::MismatchedParentheses));
        assert!(matches!(err("now >"), ParseError::General(m) if m.contains("after `>`")));
        assert!(matches!(err("> now"), ParseError::General(m) if m.contains("before `>`")));
        assert!(matches!(err("list A (now)"), ParseError::General(_)));
        assert!(matches!(err("device \"a b"), ParseError::General(_)));
        assert!(matches!(err("now --desc"), ParseError::General(m) if m.contains("--desc")));
    }
}
