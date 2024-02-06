use std::{borrow::Borrow, cell::RefCell, ops::Deref, rc::Rc};

use crate::{
	data::Data,
	parser::Node,
	scope::{Function, Scope},
};

pub fn evaluate(node: &Node, scope_ref: Rc<RefCell<dyn Scope>>) -> Data {
	let scope: &RefCell<Scope> = scope_ref.borrow();
	let scope = scope.borrow();

	match node {
		Node::FnCall {
			name,
			parameters,
			yield_fn,
		} => {
			let function = scope
				.get_function(&name)
				.unwrap_or_else(|| panic!("Unknown value or function '{}'.", &name));

			let mut args: Vec<Data> = Vec::new();
			for n in parameters {
				args.push(evaluate(n, Rc::clone(&scope_ref)));
			}

			drop(scope);
			let return_value = function.call(
				args,
				if let Some(body) = yield_fn {
					Some(Function::Custom {
						body: Rc::new(body.deref().clone()),
						scope_ref: Rc::clone(&scope_ref),
					})
				} else {
					None
				},
				Rc::clone(&scope_ref),
			);

			return return_value;
		}
		Node::Scope { body } => {
			let scope = Scope::new(Some(Rc::clone(&scope_ref)));
			let scope_ref = Rc::new(RefCell::new(scope));

			for n in body {
				evaluate(n, Rc::clone(&scope_ref));
			}

			let scope: &RefCell<Scope> = scope_ref.borrow();
			let return_value = scope.borrow().return_value.clone();
			return if let Data::None = return_value {
				Data::Scope(scope_ref)
			} else {
				return_value
			};
		}
		Node::ParameterBlock { body } => {
			let mut return_value: Data = Data::None;
			for n in body {
				return_value = evaluate(n, Rc::clone(&scope_ref));
			}

			return return_value;
		}
		Node::Program { body } => {
			drop(scope);
			for n in body {
				evaluate(n, Rc::clone(&scope_ref));
			}
			return Data::None;
		}
		Node::FnAccess { target, call } => {
			let target = evaluate(target, Rc::clone(&scope_ref));

			if let Data::Scope(call_scope) = target {
				evaluate(call, Rc::clone(&call_scope))
			} else {
				panic!("Tried to access properties of a non-scope data type.")
			}
		}
		Node::Boolean(v) => Data::Boolean(*v),
		Node::Number(v) => Data::Number(*v),
		Node::String(v) => Data::String(v.clone()),
		Node::Memory(name) => Data::Memory {
			scope: Rc::clone(&scope_ref),
			name: name.clone(),
		},
		Node::None => Data::None,
	}
}
