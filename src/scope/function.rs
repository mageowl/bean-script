use core::fmt::Debug;
use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

use crate::{data::Data, evaluator, parser::Node};

use super::Scope;

#[derive(Debug, Clone)]
pub struct CallScope {
	parent: Rc<RefCell<dyn Scope>>,
	arguments: Rc<Vec<Data>>,
	yield_fn: Rc<Option<Function>>,
	from_scope: Rc<RefCell<dyn Scope>>,
}

impl CallScope {
	pub fn args(&self) -> Rc<Vec<Data>> {
		Rc::clone(&self.arguments)
	}

	pub fn yield_fn(&self) -> Rc<Option<Function>> {
		Rc::clone(&self.yield_fn)
	}

	pub fn from_scope(&self) -> Rc<RefCell<dyn Scope>> {
		Rc::clone(&self.from_scope)
	}
}

impl Scope for CallScope {
	fn has_function(&self, name: &str) -> bool {
		RefCell::borrow(&self.parent).has_function(name)
	}

	fn get_function(&self, name: &str) -> Option<Function> {
		RefCell::borrow(&self.parent).get_function(name)
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

	fn get_call_scope(&self) -> Option<Rc<RefCell<CallScope>>> {
		Some(Rc::new(RefCell::new(self.clone())))
	}

	fn set_return_value(&mut self, _value: Data) {}
	fn get_function_list(&self) -> std::collections::HashMap<String, Function> {
		HashMap::new()
	}

	fn as_any(&self) -> &dyn Any {
		self
	}
	fn as_mut(&mut self) -> &mut dyn Any {
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
		from_scope: Option<Rc<RefCell<dyn Scope>>>,
	) -> Data {
		match self {
			Function::Custom { body, scope_ref } => {
				let call_scope = CallScope {
					parent: Rc::clone(&scope_ref),
					arguments: Rc::new(args),
					yield_fn: Rc::new(yield_fn),
					from_scope: Rc::clone(from_scope.as_ref().unwrap_or(&scope)),
				};

				evaluator::evaluate_verbose(
					body,
					Rc::new(RefCell::new(call_scope)),
					return_scope,
					None,
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
							value: args[0].clone(),
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
		self.call_verbose(args, yield_fn, scope, false, None)
	}

	pub fn call_scope(
		&self,
		args: Vec<Data>,
		yield_fn: Option<Function>,
		scope: Rc<RefCell<dyn Scope>>,
	) -> Data {
		self.call_verbose(args, yield_fn, scope, true, None)
	}

	pub fn call_from(
		&self,
		args: Vec<Data>,
		yield_fn: Option<Function>,
		scope: Rc<RefCell<dyn Scope>>,
		from_scope: Option<Rc<RefCell<dyn Scope>>>,
	) -> Data {
		self.call_verbose(args, yield_fn, scope, false, from_scope)
	}
}

impl Debug for Function {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Custom { body, scope_ref: _ } => {
				f.debug_struct("Custom").field("body", body).finish()
			}
			Self::BuiltIn { .. } => f.debug_struct("BuiltIn").finish(),
			Self::Variable {
				value,
				constant,
				name: _,
			} => f
				.debug_struct("Variable")
				.field("value", value)
				.field("constant", constant)
				.finish(),
		}
	}
}
