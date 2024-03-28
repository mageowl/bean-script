use std::{ panic::catch_unwind, rc::Rc, sync::Mutex };

use bean_script::{
    data::Data,
    evaluator,
    lexer,
    modules::{ runtime, Module },
    parser::{ self, Node },
    scope::{ block_scope::BlockScope, ScopeRef },
    util::make_ref,
};
use rustyline::{ error::ReadlineError, DefaultEditor };

pub fn open() -> rustyline::Result<()> {
    let runtime: ScopeRef = make_ref(Module::new(runtime::construct));
    let program_scope: ScopeRef = make_ref(BlockScope::new(Some(runtime)));
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

                let result = catch_unwind(|| {
                    let scope_ref = Rc::clone(&mutex.lock().unwrap());

                    let mut tree = parser::parse(lexer::tokenize(line));

                    if let Node::Program { body } = tree {
                        if body.len() == 1 {
                            tree = *body[0].clone();
                        } else {
                            tree = Node::Program { body };
                        }
                    }

                    evaluator::evaluate(&tree, scope_ref)
                });

                if let Ok(data) = result {
                    match data {
                        Data::None => (),
                        _ => println!("{:?}", data),
                    }
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                break;
            }
            Err(_) => {
                panic!("Error reading input.");
            }
        }
    }

    Ok(())
}
