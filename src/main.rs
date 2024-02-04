use std::{cell::RefCell, env, fs, rc::Rc};

use f_script::{evaluator, lexer, parser, scope::Scope};

fn main() {
	let args: Vec<String> = env::args().collect();
	let file = fs::read_to_string(&args[1]).expect("Failed to open file");

	let tokens = lexer::tokenize(file);
	let tree = parser::parse(tokens);

	let runtime = Rc::new(RefCell::new(Scope::runtime()));
	evaluator::evaluate(&tree, runtime);
}
