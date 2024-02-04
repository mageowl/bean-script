use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use crate::{
	data::Data,
	parser::Node,
	scope::{Function, Scope},
};

pub fn evaluate(node: Node, scope_ref: Rc<RefCell<Scope>>) -> Data {
	let scope: &RefCell<Scope> = scope_ref.borrow();
	let scope = scope.borrow_mut();

	match node {
		Node::FnCall {
			name,
			parameters,
			yield_fn,
		} => {
			let function = scope
				.get_function(&name)
				.unwrap_or_else(|| panic!("Unknown value or function '{}'.", &name));

			let return_value = function.call(
				Vec::new(),
				if let Some(body) = yield_fn {
					Some(Function::Custom {
						body: Rc::new(vec![*body]),
					})
				} else {
					None
				},
				Rc::clone(&scope_ref),
			);

			return return_value;
		}
		Node::Scope { body } => {
			let new_scope =
				Rc::new(RefCell::new(Scope::new(Some(Rc::clone(&scope_ref)))));

			for n in body {
				evaluate(*n, Rc::clone(&new_scope));
			}

			return Data::Scope(new_scope);
		}
		Node::ParameterBlock { body } => todo!(),
		Node::Program { body } => todo!(),
		Node::FnAccess { target, call } => todo!(),
		Node::Boolean(_) => todo!(),
		Node::Number(_) => todo!(),
		Node::String(_) => todo!(),
		Node::Memory(_) => todo!(),
		Node::None => todo!(),
		Node::Error(_) => todo!(),
	}
}
