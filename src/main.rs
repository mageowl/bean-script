use std::{env, fs};

use f_script::{lexer, parser};

fn main() {
	let args: Vec<String> = env::args().collect();
	let file = fs::read_to_string(&args[1]).expect("Failed to open file");

	let tokens = lexer::tokenize(file);

	let tree = parser::parse(tokens);
	dbg!(tree);
}
