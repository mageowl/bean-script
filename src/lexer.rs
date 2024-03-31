use std::{cell::RefCell, mem};

use crate::pat_check;

const SYMBOLS: [char; 7] = [':', '(', ')', '{', '}', ',', '.'];

enum Context {
	Program,
	String,
	Name,
	LineComment,
	BlockComment,
}

fn chunk(code: String) -> Vec<String> {
	let mut chunks: Vec<String> = Vec::new();
	let current_chunk = RefCell::from(String::new());
	let mut context = Context::Program;
	let chars: Vec<char> = code.chars().collect();

	let mut split = || {
		if current_chunk.borrow().len() > 0 {
			chunks.push(mem::replace(&mut current_chunk.borrow_mut(), String::new()));
		}
	};

	let append = |char: &char| current_chunk.borrow_mut().push_str(&char.to_string());

	for (i, char) in code.chars().enumerate() {
		match context {
			Context::Program => {
				if char == ' ' || char == '\t' || char == '\n' {
					split();
				} else if char == '/' && chars[i + 1] == '/' {
					split();
					context = Context::LineComment;
				} else if char == '/' && chars[i + 1] == '*' {
					split();
					context = Context::BlockComment;
				} else if char == '"' {
					split();
					append(&char);
					context = Context::String;
				} else if SYMBOLS.contains(&char)
					&& !(char == '.'
						&& RefCell::borrow(&current_chunk).parse::<f64>().is_ok())
				{
					split();
					append(&char);
					split();
				} else if char == '<' {
					split();
					append(&char);
					context = Context::Name;
				} else {
					append(&char);
				}
			}
			Context::String => {
				if char == '"' && chars[i - 1] != '\\' {
					append(&char);
					split();
					context = Context::Program;
				} else {
					append(&char);
				}
			}
			Context::Name => {
				if char == '>' {
					append(&char);
					split();
					context = Context::Program;
				} else {
					append(&char);
				}
			}
			Context::LineComment => {
				if char == '\n' {
					context = Context::Program;
				}
			}
			Context::BlockComment => {
				if char == '*' && chars[i + 1] == '/' {
					context = Context::Program;
				}
			}
		}
	}

	split();
	return chunks;
}

#[derive(Debug)]
pub enum Token {
	FnName(String),
	FnBody,
	ArgSeparator,
	ArgOpen,
	ArgClose,
	ScopeOpen,
	ScopeClose,
	Accessor,

	Boolean(bool),
	Number(f64),
	String(String),
	Name(String),
	None,

	EOF,
}

pub fn tokenize(code: String) -> Vec<Token> {
	let chunks = chunk(code);
	let mut tokens: Vec<Token> = Vec::new();

	for chunk in chunks {
		tokens.push(if let Ok(n) = chunk.parse::<f64>() {
			if tokens
				.last()
				.is_some_and(|x| pat_check!(Token::Accessor = x))
			{
				Token::FnName(chunk)
			} else {
				Token::Number(n)
			}
		} else if chunk.starts_with('"') && chunk.ends_with('"') {
			Token::String(String::from(chunk.trim_matches('"')))
		} else if chunk == "true" || chunk == "false" {
			Token::Boolean(chunk == "true")
		} else if chunk == "none" {
			Token::None
		} else if chunk.starts_with('<') && chunk.ends_with('>') {
			Token::Name(String::from(chunk.trim_matches(['<', '>'])))
		} else if chunk == ":" {
			Token::FnBody
		} else if chunk == "," {
			Token::ArgSeparator
		} else if chunk == "(" {
			Token::ArgOpen
		} else if chunk == ")" {
			Token::ArgClose
		} else if chunk == "{" {
			Token::ScopeOpen
		} else if chunk == "}" {
			Token::ScopeClose
		} else if chunk == "." {
			Token::Accessor
		} else {
			Token::FnName(chunk)
		});
	}

	tokens.push(Token::EOF);

	return tokens;
}
