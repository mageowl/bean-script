use std::{borrow::Borrow, cell::RefCell, collections::HashMap, mem, rc::Rc};

use crate::{data::Data, evaluator::evaluate_verbose, parser::Node};

pub enum IfState {
	Started,
	Captured,
	Finished,
}

pub trait Scope {
	fn has_function(&self, name: &str) -> bool;
	fn get_function(&self, name: &str) -> Option<Function>;
	fn set_function(&mut self, name: &str, function: Function);
	fn delete_function(&mut self, name: &str);

	fn parent(&self) -> Option<Rc<RefCell<dyn Scope>>> {
		None
	}
}

pub struct BlockScope {
	local_functions: HashMap<String, Function>,
	parent: Option<Rc<RefCell<dyn Scope>>>,
	pub arguments: Vec<Data>,
	pub return_value: Data,
	pub if_state: IfState,
	pub match_value: Option<Data>,
}

impl BlockScope {
	pub fn new(parent: Option<Rc<RefCell<dyn Scope>>>) -> Self {
		Self {
			local_functions: HashMap::new(),
			arguments: parent
				.as_ref()
				.map(|p| {
					Borrow::<RefCell<Scope>>::borrow(p)
						.borrow()
						.arguments
						.clone()
				})
				.unwrap_or_else(|| Vec::new()),
			parent,
			return_value: Data::None,
			if_state: IfState::Finished,
			match_value: None,
		}
	}
}

impl Scope for BlockScope {
	fn has_function(&self, name: &str) -> bool {
		if self.local_functions.contains_key(name) {
			true
		} else if let Some(parent) = &self.parent {
			let borrow: &RefCell<Scope> = parent.borrow();
			borrow.borrow().has_function(name)
		} else {
			false
		}
	}

	fn get_function(&self, name: &str) -> Option<Function> {
		let function = self.local_functions.get(name);
		if function.is_some() {
			function.map(|x| x.clone())
		} else if let Some(parent) = &self.parent {
			let borrow: &RefCell<Scope> = parent.borrow();
			borrow.borrow().get_function(name).map(|x| x.clone())
		} else {
			None
		}
	}

	fn set_function(&mut self, name: &str, function: Function) {
		if self.local_functions.contains_key(name) {
			*self.local_functions.get_mut(name).unwrap() = function
		} else {
			self.local_functions.insert(String::from(name), function);
		}
	}

	fn delete_function(&mut self, name: &str) {
		self.local_functions.remove(name);
	}

	fn parent(&self) -> Option<Rc<RefCell<dyn Scope>>> {
		self.parent
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
				let args_prev = mem::replace(&mut scope_ref.borrow_mut().arguments, args);

				let value = evaluate_verbose(body, Rc::clone(scope_ref), return_scope);
				scope_ref.borrow_mut().arguments = args_prev;
				value
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
		scope: Rc<RefCell<Scope>>,
	) -> Data {
		self.call_verbose(args, yield_fn, scope, false)
	}

	pub fn call_scope(
		&self,
		args: Vec<Data>,
		yield_fn: Option<Function>,
		scope: Rc<RefCell<Scope>>,
	) -> Data {
		self.call_verbose(args, yield_fn, scope, true)
	}
}
