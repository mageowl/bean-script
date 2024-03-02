use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use crate::{
	data::Data,
	parser::Node,
	scope::{block_scope::BlockScope, function::Function, ScopeRef},
};

pub fn evaluate_verbose(
	node: &Node,
	scope_ref: ScopeRef,
	return_scope: bool,
	access_scope_ref: Option<ScopeRef>,
) -> Data {
	let scope = RefCell::borrow(&scope_ref);

	match node {
		Node::FnCall {
			name,
			parameters,
			yield_fn,
		} => {
			let function = scope
				.get_function(&name)
				.unwrap_or_else(|| panic!("Unknown value or function '{}'.", &name));
			drop(scope);

			let mut args: Vec<Data> = Vec::new();
			for n in parameters {
				args.push(evaluate(
					n,
					Rc::clone(access_scope_ref.as_ref().unwrap_or(&scope_ref)),
				));
			}

			let return_value = function.call_from(
				args,
				if let Some(body) = yield_fn {
					Some(Function::Custom {
						body: Rc::new(*body.clone()),
						scope_ref: Rc::clone(
							access_scope_ref.as_ref().unwrap_or(&scope_ref),
						),
					})
				} else {
					None
				},
				Rc::clone(&scope_ref),
				access_scope_ref,
			);

			return return_value;
		}
		Node::Scope { body } => {
			let scope = BlockScope::new(Some(Rc::clone(&scope_ref)));
			let scope_ref = Rc::new(RefCell::new(scope));

			for n in body {
				evaluate(n, Rc::clone(&scope_ref) as ScopeRef);
				if RefCell::borrow(&scope_ref).did_break() {
					break;
				}
			}

			let scope: &RefCell<BlockScope> = scope_ref.borrow();
			let return_value = scope.borrow().return_value.clone();
			return if return_scope {
				Data::Scope(scope_ref)
			} else {
				return_value
			};
		}
		Node::ParameterBlock { body } => {
			drop(scope);
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

			if let Data::Scope(target_scope) = target {
				drop(scope);
				evaluate_verbose(
					call,
					Rc::clone(&target_scope),
					false,
					Some(Rc::clone(&scope_ref)),
				)
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

pub fn evaluate(node: &Node, scope_ref: ScopeRef) -> Data {
	evaluate_verbose(node, scope_ref, false, None)
}
