use std::{cell::RefCell, env, fs, rc::Rc};

use f_script::{
	evaluator, lexer,
	modules::{runtime, Module},
	parser,
};

fn main() {
	let args: Vec<String> = env::args().collect();
	let file = fs::read_to_string(&args[1]).expect("Failed to open file");

	let tokens = lexer::tokenize(file);
	let tree = parser::parse(tokens);

	let runtime = Module::new(runtime::construct);
	let runtime_scope = Rc::new(RefCell::new(runtime.as_scope()));
	evaluator::evaluate(&tree, runtime_scope);
}
