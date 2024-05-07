use std::cell::Cell;

use crate::{
	error::{Error, ErrorSource},
	lexer::Token,
};

#[derive(Debug, Clone)]
pub enum Node {
	FnCall {
		name: String,
		parameters: Vec<Box<PosNode>>,
		body_fn: Option<Box<PosNode>>,
	},
	Scope {
		body: Vec<Box<PosNode>>,
	},
	ParameterBlock {
		body: Vec<Box<PosNode>>,
	},
	Program {
		body: Vec<Box<PosNode>>,
	},
	FnAccess {
		target: Box<PosNode>,
		call: Box<PosNode>,
	},
	Boolean(bool),
	Number(f64),
	String(String),
	Name(String),
	None,
}

#[derive(Debug, Clone)]
pub struct PosNode {
	pub node: Node,
	pub ln: usize,
}

pub fn parse(tokens: Vec<Token>) -> Result<PosNode, Error> {
	let i = Cell::new(0usize);
	let line = Cell::new(1usize);
	let mut body = Vec::new();

	let next = || &tokens[i.replace(i.get() + 1)];
	let peek = || &tokens[i.get()];
	let get_ln = || line.get();
	let new_ln = || line.replace(line.get() + 1);

	fn parse_token<'a>(
		mut token: &'a Token,
		next: &dyn Fn() -> &'a Token,
		peek: &dyn Fn() -> &'a Token,
		get_ln: &dyn Fn() -> usize,
		new_ln: &dyn Fn() -> usize,
	) -> Result<PosNode, Error> {
		while let Token::LineBreak = token {
			new_ln();
			token = next();
		}

		let node: Node = match token {
			Token::FnName(name) => {
				let mut parameters = Vec::new();
				let mut body_fn = None;

				if let Token::ArgOpen = peek() {
					next();

					loop {
						while let Token::LineBreak = peek() {
							new_ln();
							next();
						}

						if let Token::ArgClose = peek() {
							break;
						} else if let Token::EOF = peek() {
							return Err(Error::new(
								"Unexpected end of file. (expected ')')",
								ErrorSource::Line(get_ln()),
							));
						}

						let mut body = Vec::new();

						loop {
							match peek() {
								Token::ArgSeparator | Token::ArgClose | Token::EOF => {
									break;
								}
								_ => (),
							}
							body.push(Box::new(parse_token(
								next(),
								&next,
								&peek,
								&get_ln,
								&new_ln,
							)?));
						}

						if let Token::ArgSeparator = peek() {
							next();
						}

						parameters.push(Box::new(PosNode {
							node: Node::ParameterBlock { body },
							ln: get_ln(),
						}));
					}

					if let Token::ArgClose = peek() {
						next();
					}
				}

				while let Token::LineBreak = peek() {
					new_ln();
					next();
				}

				if let Token::FnBody = peek() {
					next();
					body_fn = Some(Box::new(parse_token(
						next(),
						&next,
						&peek,
						&get_ln,
						&new_ln,
					)?));
				}

				while let Token::LineBreak = peek() {
					new_ln();
					next();
				}

				if let Token::Accessor = peek() {
					next();

					Node::FnAccess {
						target: Box::new(PosNode {
							node: Node::FnCall {
								name: name.clone(),
								parameters,
								body_fn,
							},
							ln: get_ln(),
						}),
						call: Box::new(parse_token(
							next(),
							&next,
							&peek,
							&get_ln,
							&new_ln,
						)?),
					}
				} else {
					Node::FnCall {
						name: name.clone(),
						parameters,
						body_fn,
					}
				}
			}
			Token::FnBody => {
				return Err(Error::new(
					"Unexpected function body.",
					ErrorSource::Line(get_ln()),
				))
			}
			Token::ArgSeparator => {
				return Err(Error::new("Unexpected comma.", ErrorSource::Line(get_ln())));
			}
			Token::ArgOpen => {
				let mut body = Vec::new();

				loop {
					while let Token::LineBreak = peek() {
						new_ln();
						next();
					}

					if let Token::ArgClose = peek() {
						break;
					} else if let Token::EOF = peek() {
						return Err(Error::new(
							"Unexpected end of file. (expected ')')",
							ErrorSource::Line(get_ln()),
						));
					}
					body.push(Box::new(parse_token(
						next(),
						&next,
						&peek,
						&get_ln,
						&new_ln,
					)?));
				}

				while let Token::LineBreak = peek() {
					new_ln();
					next();
				}
				if let Token::ArgClose = peek() {
					next();
				}

				Node::ParameterBlock { body }
			}
			Token::ArgClose => {
				return Err(Error::new(
					"Unexpected closing parentheses.",
					ErrorSource::Line(get_ln()),
				))
			}
			Token::ScopeOpen => {
				let mut body = Vec::new();

				loop {
					while let Token::LineBreak = peek() {
						new_ln();
						next();
					}

					if let Token::ScopeClose = peek() {
						break;
					} else if let Token::EOF = peek() {
						return Err(Error::new(
							"Unexpected end of file. (expected '}')",
							ErrorSource::Line(get_ln()),
						));
					}
					body.push(Box::new(parse_token(
						next(),
						&next,
						&peek,
						&get_ln,
						&new_ln,
					)?));
				}

				if let Token::ScopeClose = peek() {
					next();
				}

				Node::Scope { body }
			}
			Token::ScopeClose => {
				return Err(Error::new(
					"Unexpected closing brace.",
					ErrorSource::Line(get_ln()),
				))
			}
			Token::Accessor => {
				return Err(Error::new(
					"Unexpected dot operator.",
					ErrorSource::Line(get_ln()),
				))
			}
			Token::Boolean(v) => Node::Boolean(*v),
			Token::Number(v) => Node::Number(*v),
			Token::String(v) => Node::String(v.clone()),
			Token::Name(v) => Node::Name(v.clone()),
			Token::None => Node::None,
			Token::EOF => {
				return Err(Error::new(
					"Unexpected end of input. (How did this happen?)",
					ErrorSource::Line(get_ln()),
				))
			}
			Token::LineBreak => {
				return Err(Error::new(
					"Unexpected line break. (How did this happen?)",
					ErrorSource::Line(get_ln()),
				))
			}
		};

		return Ok(PosNode { node, ln: get_ln() });
	}

	while i.get() < tokens.len() - 1 {
		body.push(Box::new(parse_token(
			next(),
			&next,
			&peek,
			&get_ln,
			&new_ln,
		)?));
	}

	Ok(PosNode {
		node: Node::Program { body },
		ln: 0,
	})
}
