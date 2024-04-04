use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use crate::{
	data::Data,
	error::{BeanResult, Error, ErrorSource},
	parser::{Node, PosNode},
	scope::{block_scope::BlockScope, function::Function, ScopeRef},
};

pub fn evaluate_verbose(
	pos_node: &PosNode,
	scope_ref: ScopeRef,
	return_scope: bool,
	access_scope_ref: Option<ScopeRef>,
) -> Result<Data, Error> {
	let scope = RefCell::borrow(&scope_ref);

	match &pos_node.node {
		Node::FnCall {
			name,
			parameters,
			body_fn,
		} => {
			let function = scope.get_function(&name).ok_or_else(|| {
				Error::new(
					&format!("Unknown value or function {}.", name),
					ErrorSource::Line(pos_node.ln),
				)
			})?;
			drop(scope);

			let mut args: Vec<Data> = Vec::new();
			for n in parameters {
				args.push(evaluate(
					n,
					Rc::clone(access_scope_ref.as_ref().unwrap_or(&scope_ref)),
				)?);
			}

			let return_value = function
				.call_from(
					args,
					if let Some(body) = body_fn {
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
				)
				.trace(ErrorSource::Line(pos_node.ln));

			return return_value;
		}
		Node::Scope { body } => {
			let scope = BlockScope::new(Some(Rc::clone(&scope_ref)));
			let scope_ref = Rc::new(RefCell::new(scope));

			for n in body {
				evaluate(n, Rc::clone(&scope_ref) as ScopeRef)?;
				if RefCell::borrow(&scope_ref).did_break() {
					break;
				}
			}

			let scope: &RefCell<BlockScope> = scope_ref.borrow();
			let return_value = scope.borrow().return_value.clone();
			return if return_scope {
				Ok(Data::Scope(scope_ref))
			} else {
				Ok(return_value)
			};
		}
		Node::ParameterBlock { body } => {
			drop(scope);
			let mut return_value: Data = Data::None;
			for n in body {
				return_value = evaluate(n, Rc::clone(&scope_ref))?;
			}

			return Ok(return_value);
		}
		Node::Program { body } => {
			drop(scope);
			for n in body {
				evaluate(n, Rc::clone(&scope_ref))?;
			}
			return Ok(Data::None);
		}
		Node::FnAccess { target, call } => {
			let target = evaluate(target, Rc::clone(&scope_ref))?;

			if let Data::Scope(target_scope) = target {
				drop(scope);
				evaluate_verbose(
					&call,
					Rc::clone(&target_scope),
					false,
					Some(Rc::clone(&scope_ref)),
				)
			} else {
				return Err(Error::new(
					&format!(
						"Expected scope for dot operator, but got {}.",
						target.get_type().to_string()
					),
					ErrorSource::Line(pos_node.ln),
				));
			}
		}
		Node::Boolean(v) => Ok(Data::Boolean(*v)),
		Node::Number(v) => Ok(Data::Number(*v)),
		Node::String(v) => Ok(Data::String(v.clone())),
		Node::Name(name) => Ok(Data::Name {
			scope: Rc::clone(&scope_ref),
			name: name.clone(),
		}),
		Node::None => Ok(Data::None),
	}
}

pub fn evaluate(node: &PosNode, scope_ref: ScopeRef) -> Result<Data, Error> {
	evaluate_verbose(node, scope_ref, false, None)
}
