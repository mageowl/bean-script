use std::{
	cell::RefCell,
	env::{self, Args},
	fs,
	rc::Rc,
};

use f_script::{
	evaluator, lexer,
	modules::{runtime, Module},
	parser,
	scope::{block_scope::BlockScope, ScopeRef},
	scope_ref,
};

const HELP_MSG: &str = "Function-based language interpreter.
Usage: f-script [OPTIONS] [PATH]

Options:
	-p, --parse     Parse file without evaluating it.
	-l, --tokenize  Tokenize file without parsing it.
	-h, --help      Print this message and exit.";

struct CliArgs {
	no_args: bool,
	f_help: bool,
	f_parse: bool,
	f_tokenize: bool,
	path: Option<String>,
}

fn main() {
	let args = parse_args(env::args());
	if args.no_args || args.f_help {
		println!("{}", HELP_MSG);
	} else {
		let file = fs::read_to_string(args.path.expect("Expected path to file."))
			.expect("Failed to open file");

		let tokens = lexer::tokenize(file);
		if args.f_tokenize {
			dbg!(tokens);
			return;
		}

		let tree = parser::parse(tokens);
		if args.f_parse {
			dbg!(tree);
			return;
		}

		let runtime: ScopeRef = scope_ref!(Module::new(runtime::construct));
		let program_scope = BlockScope::new(Some(runtime));
		evaluator::evaluate(&tree, scope_ref!(program_scope));
	}
}

fn parse_args(mut args: Args) -> CliArgs {
	let no_args = args.len() == 1;
	let mut flags: Vec<String> = Vec::new();
	let mut path: Option<String> = None;

	args.next();
	for arg in args {
		if arg.starts_with("--") || arg.starts_with("-") {
			flags.push(arg);
		} else if path.is_none() {
			path = Some(arg);
		}
	}

	CliArgs {
		no_args,
		path,
		f_help: flags.contains(&String::from("--help"))
			|| flags.contains(&String::from("-h")),
		f_parse: flags.contains(&String::from("--parse"))
			|| flags.contains(&String::from("-p")),
		f_tokenize: flags.contains(&String::from("--tokenize"))
			|| flags.contains(&String::from("-l")),
	}
}
