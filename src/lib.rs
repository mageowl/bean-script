use error::Error;
use modules::CustomModule;
use util::MutRc;

pub mod data;
pub mod error;
pub mod logger;
pub mod modules;
pub mod scope;
pub mod util;

pub mod evaluator;
pub mod lexer;
pub mod parser;

pub fn interpret(code: String, program_scope: MutRc<CustomModule>) -> Result<(), Error> {
	let tokens = lexer::tokenize(code);
	let tree = parser::parse(tokens)?;
	evaluator::evaluate(&tree, program_scope)?;
	Ok(())
}
