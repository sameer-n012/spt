use std::collections::HashMap;
use std::collections::HashSet;

use std::iter::Peekable;
use std::str::Chars;

// Command struct:
// name is the command word (e.g. "play", "search", etc.)
// args is a list of arguments, which may be text or other subcommands.
#[derive(Debug, Clone)]
pub struct CommandNode {
    pub name: String,
    pub args: Vec<Arg>,
}

// An argument can be either:
// - Another Command (subcommand)
// - Plain text (e.g. a string or a flag like "-h")
#[derive(Debug, Clone)]
pub enum Arg {
    Command(CommandNode),
    Text(String),
}

// Parsing/Tokenizing Errors:
#[derive(Debug)]
pub enum ParseError {
    MismatchedParentheses,
    UnexpectedEndOfInput,
    NoCommandFound,
    General(String),
}

// A token, with a flag indicating whether it was originally quoted or not.
#[derive(Debug, Clone)]
pub struct Token {
    text: String,
    is_quoted: bool,
}

pub fn verify_command(tokens: &[Token], command_list: &HashSet<String>) -> Result<(), ParseError> {
    if tokens.is_empty() {
        return Err(ParseError::UnexpectedEndOfInput);
    }

    if tokens[0].is_quoted || !is_recognized_command(&tokens[0].text, command_list) {
        return Err(ParseError::NoCommandFound);
    }

    Ok(())
}

pub fn verify_flags(
    cmd: &CommandNode,
    allowed_flags: &HashMap<String, Vec<String>>,
) -> Result<(), ParseError> {
    for arg in &cmd.args {
        match arg {
            Arg::Command(subcmd) => {
                verify_flags(subcmd, allowed_flags)?;
            }
            Arg::Text(text) => {
                if text.starts_with("-")
                    && !allowed_flags
                        .get(&cmd.name)
                        .is_some_and(|v| v.contains(text))
                {
                    return Err(ParseError::General(format!(
                        "Flag {} cannot be used as an argument for {}",
                        text, cmd.name
                    )));
                }
            }
        }
    }

    Ok(())
}

// Splits the input string into tokens, respecting quotes and parentheses.
// Store an additional boolean is_quoted to indicate if the token came from a
// quoted region.
pub fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    let mut result = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            c if c.is_whitespace() => {
                // Skip whitespace
                chars.next();
            }
            '(' | ')' => {
                // Parentheses become individual tokens
                result.push(Token {
                    text: ch.to_string(),
                    is_quoted: false,
                });
                chars.next();
            }
            '"' => {
                // Read a quoted string
                let quoted_text = read_quoted(&mut chars)?;
                result.push(Token {
                    text: quoted_text,
                    is_quoted: true, // was inside quotes
                });
            }
            _ => {
                // Read an unquoted token
                let unquoted_text = read_unquoted(&mut chars)?;
                result.push(Token {
                    text: unquoted_text,
                    is_quoted: false,
                });
            }
        }
    }

    Ok(result)
}

// Reads everything until the next unescaped quote.
fn read_quoted(chars: &mut Peekable<Chars>) -> Result<String, ParseError> {
    // Consume the leading quote
    if chars.next() != Some('"') {
        return Err(ParseError::General("Expected opening quote".to_string()));
    }

    let mut result = String::new();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                // closing quote
                return Ok(format!("\"{}\"", result));
            }
            _ => result.push(ch),
        }
    }

    // If we reach EOF without a matching quote
    Err(ParseError::General("Unclosed double quote".to_string()))
}

// Reads an unquoted token until whitespace, parentheses, or quotes.
fn read_unquoted(chars: &mut Peekable<Chars>) -> Result<String, ParseError> {
    let mut result = String::new();
    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() || ch == '(' || ch == ')' || ch == '"' {
            break;
        }
        result.push(ch);
        chars.next();
    }
    Ok(result)
}

pub fn parse(tokens: &[Token], command_list: &HashSet<String>) -> Result<CommandNode, ParseError> {
    let mut pos: usize = 0;
    let tree = parse_rec(tokens, &mut pos, command_list);

    // If there are leftover tokens, error
    if pos < tokens.len() {
        return Err(ParseError::General(format!(
            "Unconsumed tokens remain after parsing: {:?}",
            &tokens[pos..]
        )));
    }

    return tree;
}

// Parses a command from the token stream.
// Caller should ensure that the next token is a recognized command
// or an opening parenthesis containing a command.
// This function tries to parse: commandName arg1 arg2 ...
fn parse_rec(
    tokens: &[Token],
    pos: &mut usize,
    command_list: &HashSet<String>,
) -> Result<CommandNode, ParseError> {
    if *pos >= tokens.len() {
        return Err(ParseError::UnexpectedEndOfInput);
    }

    // If the next token is "(" then we parse one command inside parentheses
    if tokens[*pos].text == "(" && !tokens[*pos].is_quoted {
        *pos += 1; // consume '('
                   // parse a command inside the parentheses
        let cmd = parse_rec(tokens, pos, command_list)?;

        // The next token must be ")"
        if *pos >= tokens.len() {
            return Err(ParseError::MismatchedParentheses);
        }
        if tokens[*pos].text != ")" || tokens[*pos].is_quoted {
            return Err(ParseError::MismatchedParentheses);
        }
        *pos += 1; // consume ')'
        return Ok(cmd);
    }

    // Otherwise, it should be a recognized command (unquoted) to be valid:
    if tokens[*pos].is_quoted || !is_recognized_command(&tokens[*pos].text, command_list) {
        return Err(ParseError::NoCommandFound);
    }
    let name = tokens[*pos].text.clone();
    *pos += 1; // consume the command name

    let mut command = CommandNode {
        name,
        args: Vec::new(),
    };

    // Gather arguments while we have tokens left and until we reach a closing parenthesis
    // or the end of the token list. Each argument can be:
    // - A parenthesized subcommand (e.g. (search foo))
    // - A recognized unquoted command
    // - A text token
    while *pos < tokens.len() {
        // If the next token is ")" that means we close the current command.
        if tokens[*pos].text == ")" && !tokens[*pos].is_quoted {
            break;
        }

        // If the next token is "(" => parse subcommand inside parentheses as a single Arg
        if tokens[*pos].text == "(" && !tokens[*pos].is_quoted {
            // parse a nested command
            *pos += 1; // consume '('
            let subcmd = parse_rec(tokens, pos, command_list)?;
            // must see ")"
            if *pos >= tokens.len() {
                return Err(ParseError::MismatchedParentheses);
            }
            if tokens[*pos].text != ")" || tokens[*pos].is_quoted {
                return Err(ParseError::MismatchedParentheses);
            }
            *pos += 1; // consume ')'
            command.args.push(Arg::Command(subcmd));
            continue;
        }

        // If the next token is an unquoted recognized command => parse as subcommand
        if !tokens[*pos].is_quoted && is_recognized_command(&tokens[*pos].text, command_list) {
            let subcmd = parse_rec(tokens, pos, command_list)?;
            command.args.push(Arg::Command(subcmd));
            continue;
        }

        // Otherwise, treat it as plain text
        command.args.push(Arg::Text(tokens[*pos].text.clone()));
        *pos += 1;
    }

    Ok(command)
}

// Returns true if the given token text is in the recognized command list.
fn is_recognized_command(txt: &str, command_list: &HashSet<String>) -> bool {
    command_list.contains(txt)
}
