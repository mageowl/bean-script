use core::fmt::Debug;
use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
	data::Data,
	error::{Error, ErrorSource},
	evaluator,
	parser::PosNode,
};

use super::{Scope, ScopeRef};

#[derive(Debug, Clone)]
pub struct CallScope {
	parent: ScopeRef,
	arguments: Rc<Vec<Data>>,
	body_fn: Rc<Option<Function>>,
	from_scope: ScopeRef,
}

impl CallScope {
	pub fn args(&self) -> Rc<Vec<Data>> {
		Rc::clone(&self.arguments)
	}

	pub fn body_fn(&self) -> Rc<Option<Function>> {
		Rc::clone(&self.body_fn)
	}

	pub fn from_scope(&self) -> ScopeRef {
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

	fn parent(&self) -> Option<ScopeRef> {
		Some(Rc::clone(&self.parent) as ScopeRef)
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

	fn set_if_state(&mut self, _state: super::block_scope::IfState) {}
}

#[derive(Clone)]
pub enum Function {
	Custom {
		body: Rc<PosNode>,
		scope_ref: ScopeRef,
	},
	BuiltIn {
		callback:
			Rc<dyn Fn(Vec<Data>, Option<Function>, ScopeRef) -> Result<Data, Error>>,
	},
	Variable {
		value: Data,
		scope_ref: ScopeRef,
		name: String,
	},
	Constant {
		value: Data,
	},
}

impl Function {
	fn call_verbose(
		&self,
		args: Vec<Data>,
		body_fn: Option<Function>,
		scope: ScopeRef,
		return_scope: bool,
		abstract_call_scope: bool,
		from_scope: Option<ScopeRef>,
	) -> Result<Data, Error> {
		match self {
			Function::Custom { body, scope_ref } => {
				let call_scope = CallScope {
					parent: Rc::clone(&scope_ref),
					arguments: Rc::new(args),
					body_fn: Rc::new(body_fn),
					from_scope: Rc::clone(from_scope.as_ref().unwrap_or(&scope)),
				};

				evaluator::evaluate_verbose(
					body,
					if abstract_call_scope {
						Rc::new(RefCell::new(call_scope))
					} else {
						scope
					},
					return_scope,
					None,
				)
			}
			Function::BuiltIn { callback } => callback(args, body_fn, scope),
			Function::Variable {
				value,
				name,
				scope_ref,
			} => {
				if let Some(v) = body_fn {
					let pass = value.clone();
					scope_ref.borrow_mut().set_function(
						name,
						Function::Variable {
							value: v.call(Vec::new(), None, Rc::clone(&scope))?,
							scope_ref: Rc::clone(scope_ref),
							name: String::from(name),
						},
					);
					Ok(pass)
				} else {
					Ok(value.clone())
				}
			}
			Function::Constant { value } => {
				if let Some(_) = body_fn {
					Err(Error::new("Tried to edit constant.", ErrorSource::Internal))
				} else {
					Ok(value.clone())
				}
			}
		}
	}

	pub fn call(
		&self,
		args: Vec<Data>,
		body_fn: Option<Function>,
		scope: ScopeRef,
	) -> Result<Data, Error> {
		self.call_verbose(args, body_fn, scope, false, true, None)
	}

	pub fn call_scope(
		&self,
		args: Vec<Data>,
		body_fn: Option<Function>,
		scope: ScopeRef,
	) -> Result<Data, Error> {
		self.call_verbose(args, body_fn, scope, true, false, None)
	}

	pub fn call_from(
		&self,
		args: Vec<Data>,
		body_fn: Option<Function>,
		scope: ScopeRef,
		from_scope: Option<ScopeRef>,
	) -> Result<Data, Error> {
		self.call_verbose(args, body_fn, scope, false, true, from_scope)
	}

	pub fn call_direct(
		&self,
		args: Vec<Data>,
		body_fn: Option<Function>,
		scope: ScopeRef,
	) -> Result<Data, Error> {
		self.call_verbose(args, body_fn, scope, false, false, None)
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
				scope_ref: _,
				name: _,
			} => f.debug_struct("Variable").field("value", value).finish(),
			Self::Constant { value } => {
				f.debug_struct("Constant").field("value", value).finish()
			}
		}
	}
}
