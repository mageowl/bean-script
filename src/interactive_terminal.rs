use std::{rc::Rc, sync::Mutex};

use bean_script::{
	data::Data,
	evaluator, lexer,
	modules::registry::{ModuleRegistry, RegistryFeatures},
	parser::{self, Node, PosNode},
	scope::{block_scope::BlockScope, ScopeRef},
	util::make_ref,
};
use rustyline::{error::ReadlineError, DefaultEditor};

pub fn open() -> rustyline::Result<()> {
	let registry = make_ref(ModuleRegistry::new(RegistryFeatures {
		custom_modules: false,
		import: false,
		lang_debug: false,
	}));
	let program_scope: ScopeRef =
		make_ref(BlockScope::new(Some(registry.borrow().runtime())));
	let mutex = Mutex::new(program_scope);
	let mut rl = DefaultEditor::new()?;

	loop {
		let input = rl.readline(") ");

		match input {
			Ok(line) => {
				if line == "exit" {
					break;
				}

				let _ = rl.add_history_entry(line.as_str());

				let scope_ref = Rc::clone(&mutex.lock().unwrap());

				let tree = parser::parse(lexer::tokenize(line));
				if let Err(error) = tree {
					println!("\x1b[31;1merror\x1b[0m: {}", error);
					continue;
				}
				let mut tree = tree.unwrap();

				if let PosNode {
					node: Node::Program { body },
					..
				} = tree
				{
					if body.len() == 1 {
						tree = *body[0].clone();
					} else {
						tree = PosNode {
							node: Node::Program { body },
							ln: 0,
						};
					}
				}

				let result = evaluator::evaluate(&tree, scope_ref);

				if let Ok(data) = result {
					match data {
						Data::None => (),
						_ => println!("{:?}", data),
					}
				} else if let Err(error) = result {
					println!("\x1b[31;1merror\x1b[0m: {}", error);
				}
			}
			Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
				break;
			}
			Err(_) => return input.map(|_| ()),
		}
	}

	Ok(())
}
