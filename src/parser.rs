use std::cell::Cell;

use crate::lexer::Token;

#[derive(Debug)]
pub enum Node {
	FnCall {
		name: String,
		parameters: Vec<Box<Node>>,
		yield_fn: Option<Box<Node>>,
	},
	Scope {
		body: Vec<Box<Node>>,
	},
	ParameterBlock {
		body: Vec<Box<Node>>,
	},
	Program {
		body: Vec<Box<Node>>,
	},
	FnAccess {
		target: Box<Node>,
		call: Box<Node>,
	},
	Boolean(bool),
	Number(isize),
	String(String),
	Memory(String),
	None,
	Error(String),
}

pub fn parse(tokens: Vec<Token>) -> Node {
	let i = Cell::new(0);
	let mut body = Vec::new();

	let next = || &tokens[i.replace(i.get() + 1)];
	let peek = || &tokens[i.get()];

	fn parse_token<'a>(
		token: &Token,
		next: &dyn Fn() -> &'a Token,
		peek: &dyn Fn() -> &'a Token,
	) -> Node {
		let node: Node = match token {
			Token::FnName(name) => {
				let mut parameters = Vec::new();
				let mut yield_fn = None;

				if let Token::ArgOpen = peek() {
					next();

					loop {
						if let Token::ArgClose = peek() {
							break;
						}

						let mut body = Vec::new();

						loop {
							match peek() {
								Token::ArgSeparator | Token::ArgClose | Token::EOF => break,
								_ => (),
							}
							body.push(Box::new(parse_token(next(), &next, &peek)));
						}

						if let Token::ArgSeparator = peek() {
							next();
						}

						parameters.push(Box::new(Node::ParameterBlock { body }));
					}

					if let Token::ArgClose = peek() {
						next();
					}
				}

				if let Token::FnYield = peek() {
					next();
					yield_fn = Some(Box::new(parse_token(next(), &next, &peek)));
				}

				if let Token::Accessor = peek() {
					next();

					Node::FnAccess {
						target: Box::new(Node::FnCall {
							name: name.clone(),
							parameters,
							yield_fn,
						}),
						call: Box::new(parse_token(next(), &next, &peek)),
					}
				} else {
					Node::FnCall {
						name: name.clone(),
						parameters,
						yield_fn,
					}
				}
			}
			Token::FnYield => panic!("Unexpected yield symbol. (':')"),
			Token::ArgSeparator => panic!("Unexpected argument separator. (',')"),
			Token::ArgOpen => {
				let mut body = Vec::new();

				loop {
					if let Token::ArgClose = peek() {
						break;
					} else if let Token::EOF = peek() {
						break;
					}
					body.push(Box::new(parse_token(next(), &next, &peek)));
				}

				Node::ParameterBlock { body }
			}
			Token::ArgClose => panic!("Unexpected argument close symbol. (')')"),
			Token::ScopeOpen => {
				let mut body = Vec::new();

				loop {
					if let Token::ScopeClose = peek() {
						break;
					} else if let Token::EOF = peek() {
						break;
					}
					body.push(Box::new(parse_token(next(), &next, &peek)));
				}

				if let Token::ScopeClose = peek() {
					next();
				}

				Node::Scope { body }
			}
			Token::ScopeClose => panic!("Unexpected scope close symbol. ('}}')"),
			Token::Accessor => panic!("Unexpected access symbol. ('.')"),
			Token::Boolean(v) => Node::Boolean(*v),
			Token::Number(v) => Node::Number(*v),
			Token::String(v) => Node::String(v.clone()),
			Token::Memory(v) => Node::Memory(v.clone()),
			Token::None => Node::None,
			Token::EOF => panic!("Unexpected end of input."),
		};

		return node;
	}

	dbg!(&tokens);
	while i.get() < tokens.len() - 1 {
		body.push(Box::new(parse_token(next(), &next, &peek)));
	}

	Node::Program { body }
}
