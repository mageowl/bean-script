use core::fmt::Debug;
use std::{any::Any, cell::RefCell, rc::Rc};

use crate::{data::Data, evaluator, parser::Node};

use super::Scope;

#[derive(Debug)]
struct CallScope {
	parent: Rc<RefCell<dyn Scope>>,
	arguments: Vec<Data>,
}

impl CallScope {
	pub fn new(parent: Rc<RefCell<dyn Scope>>, arguments: Vec<Data>) -> Self {
		Self { parent, arguments }
	}
}

impl Scope for CallScope {
	fn has_function(&self, name: &str) -> bool {
		self.parent.borrow().has_function(name)
	}

	fn get_function(&self, name: &str) -> Option<Function> {
		self.parent.borrow().get_function(name)
	}

	fn set_function(&mut self, name: &str, function: Function) {
		self.parent.borrow_mut().set_function(name, function)
	}

	fn delete_function(&mut self, name: &str) {
		self.parent.borrow_mut().delete_function(name)
	}

	fn parent(&self) -> Option<Rc<RefCell<dyn Scope>>> {
		Some(Rc::clone(&self.parent) as Rc<RefCell<dyn Scope>>)
	}

	fn argument(&self, index: usize) -> Option<Data> {
		self.arguments.get(index).map(|d| d.clone())
	}

	fn as_any(&self) -> &dyn Any {
		self
	}
}

#[derive(Clone)]
pub enum Function {
	Custom {
		body: Rc<Node>,
		scope_ref: Rc<RefCell<dyn Scope>>,
	},
	BuiltIn {
		callback: Rc<dyn Fn(Vec<Data>, Option<Function>, Rc<RefCell<dyn Scope>>) -> Data>,
	},
	Variable {
		value: Data,
		constant: bool,
		name: String,
	},
}

impl Function {
	fn call_verbose(
		&self,
		args: Vec<Data>,
		yield_fn: Option<Function>,
		scope: Rc<RefCell<dyn Scope>>,
		return_scope: bool,
	) -> Data {
		match self {
			Function::Custom { body, scope_ref } => {
				let call_scope = CallScope::new(Rc::clone(scope_ref), args);

				evaluator::evaluate_verbose(
					body,
					Rc::new(RefCell::new(call_scope)),
					return_scope,
				)
			}
			Function::BuiltIn { callback } => callback(args, yield_fn, scope),
			Function::Variable {
				value,
				constant,
				name,
			} => {
				if args.len() == 0 {
					value.clone()
				} else if !*constant {
					let pass = value.clone();
					scope.borrow_mut().set_function(
						name,
						Function::Variable {
							value: args[1].clone(),
							constant: false,
							name: String::from(name),
						},
					);
					pass
				} else {
					panic!("Tried to assign value to constant variable.")
				}
			}
		}
	}

	pub fn call(
		&self,
		args: Vec<Data>,
		yield_fn: Option<Function>,
		scope: Rc<RefCell<dyn Scope>>,
	) -> Data {
		self.call_verbose(args, yield_fn, scope, false)
	}

	pub fn call_scope(
		&self,
		args: Vec<Data>,
		yield_fn: Option<Function>,
		scope: Rc<RefCell<dyn Scope>>,
	) -> Data {
		self.call_verbose(args, yield_fn, scope, true)
	}
}

impl Debug for Function {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Custom { body, scope_ref } => f
				.debug_struct("Custom")
				.field("body", body)
				.field("scope_ref", scope_ref)
				.finish(),
			Self::BuiltIn { .. } => f.debug_struct("BuiltIn").finish(),
			Self::Variable {
				value,
				constant,
				name,
			} => f
				.debug_struct("Variable")
				.field("value", value)
				.field("constant", constant)
				.field("name", name)
				.finish(),
		}
	}
}
