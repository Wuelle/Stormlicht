//! A runner for [html5 tokenizer tests](https://github.com/html5lib/html5lib-tests)

mod escape;

use clap::Parser;
use web::html::tokenization::{
    IgnoreParseErrors, ParseErrorHandler, Token, Tokenizer, TokenizerState,
};

use crate::escape::{unescape_str, unicode_escape};

#[derive(Debug, Default, Parser)]
#[command(version, about, long_about = None)]
struct Arguments {
    /// Initial tokenizer state
    #[arg(short = 's', long = "state")]
    initial_state: String,

    /// HTML source that should be tokenized
    #[arg(short = 'i', long = "input")]
    input: String,

    /// Name of the current test case
    #[arg(short = 'l', long = "last-start-tag")]
    last_start_tag: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Error {
    InvalidInitialState,
}

fn main() -> Result<(), Error> {
    let args = Arguments::parse();

    let last_start_tag = args
        .last_start_tag
        .clone()
        .map(|t| t[1..t.len() - 1].to_string());

    let source = unescape_str(&args.input[1..args.input.len() - 1]).expect("Invalid input text");

    // our commandline parser doesnt handle quotes very well...
    let initial_state = parse_initial_state(&args.initial_state[1..args.initial_state.len() - 1])?;

    let mut tokenizer: Tokenizer<IgnoreParseErrors> = Tokenizer::new(&source);
    tokenizer.switch_to(initial_state);
    tokenizer.set_last_start_tag(last_start_tag);

    let mut serialized_tokens = vec![];
    while let Some(token) = tokenizer.next() {
        if serialize_token(token, &mut tokenizer, &mut serialized_tokens) {
            break;
        }
    }

    let result = format!("[{}]", serialized_tokens.join(","));
    println!("{result}");

    Ok(())
}

fn parse_initial_state(initial_state: &str) -> Result<TokenizerState, Error> {
    match initial_state {
        "Data state" => Ok(TokenizerState::Data),
        "PLAINTEXT state" => Ok(TokenizerState::PLAINTEXT),
        "RCDATA state" => Ok(TokenizerState::RCDATA),
        "RAWTEXT state" => Ok(TokenizerState::RAWTEXT),
        "Script data state" => Ok(TokenizerState::ScriptData),
        "CDATA section state" => Ok(TokenizerState::CDATASection),
        _ => Err(Error::InvalidInitialState),
    }
}

fn serialize_token<P: ParseErrorHandler>(
    token: Token,
    tokenizer: &mut Tokenizer<P>,
    serialized_tokens: &mut Vec<String>,
) -> bool {
    match token {
        Token::DOCTYPE(doctype) => {
            let name = doctype
                .name
                .map(|s| format!("\"{s}\""))
                .unwrap_or("null".to_string());
            let public_id = doctype
                .public_ident
                .map(|s| format!("\"{s}\""))
                .unwrap_or("null".to_string());
            let system_id = doctype
                .system_ident
                .map(|s| format!("\"{s}\""))
                .unwrap_or("null".to_string());
            let force_quirks = doctype.force_quirks;

            serialized_tokens.push(format!(
                "[\"DOCTYPE\", {}, {}, {}, {:?}]",
                unicode_escape(&name),
                unicode_escape(&public_id),
                unicode_escape(&system_id),
                !force_quirks,
            ));
        },
        Token::StartTag(tagdata) => {
            let attributes = tagdata
                .attributes
                .iter()
                .map(|(key, value)| {
                    format!(
                        "\"{}\": \"{}\"",
                        unicode_escape(&key.to_string()),
                        unicode_escape(&value.to_string())
                    )
                })
                .collect::<Vec<String>>()
                .join(",");
            let serialized_token = if tagdata.self_closing {
                format!(
                    "[\"StartTag\", \"{}\", {{{attributes}}}, true]",
                    unicode_escape(&tagdata.name.to_string()),
                )
            } else {
                format!(
                    "[\"StartTag\", \"{}\", {{{attributes}}}]",
                    unicode_escape(&tagdata.name.to_string()),
                )
            };
            serialized_tokens.push(serialized_token);
        },
        Token::EndTag(tagdata) => {
            serialized_tokens.push(format!(
                "[\"EndTag\", \"{}\"]",
                unicode_escape(&tagdata.name.to_string()),
            ));
        },
        Token::Comment(comment) => {
            serialized_tokens.push(format!("[\"Comment\", \"{}\"]", unicode_escape(&comment)));
        },
        Token::EOF => {
            return true;
        },
        Token::Character(c) => {
            // Collect all adjacent character tokens
            let mut data = c.to_string();
            loop {
                match tokenizer.next() {
                    Some(Token::Character(c)) => data.push(c),
                    Some(other) => {
                        serialized_tokens
                            .push(format!("[\"Character\", \"{}\"]", unicode_escape(&data)));
                        return serialize_token(other, tokenizer, serialized_tokens);
                    },
                    None => {
                        return true;
                    },
                }
            }
        },
    }
    false
}
