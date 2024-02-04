use std::{borrow::Borrow, cell::RefCell, collections::HashMap, rc::Rc};

use crate::{data::Data, evaluator::evaluate, parser::Node};

pub enum IfState {
	Started,
	Captured,
	Finished,
}

pub struct Scope {
	local_functions: HashMap<String, Function>,
	parent: Option<Rc<RefCell<Scope>>>,
	pub return_value: Data,
	pub if_state: IfState,
	pub match_value: Option<Data>,
}

impl Scope {
	pub fn new(parent: Option<Rc<RefCell<Scope>>>) -> Self {
		Self {
			local_functions: HashMap::new(),
			parent,
			return_value: Data::None,
			if_state: IfState::Finished,
			match_value: None,
		}
	}

	pub fn runtime() -> Self {
		Self {
			local_functions: HashMap::new(),
			parent: None,
			return_value: Data::None,
			if_state: IfState::Finished,
			match_value: None,
		}
	}

	pub fn has_function(&self, name: &str) -> bool {
		if self.local_functions.contains_key(name) {
			true
		} else if let Some(parent) = &self.parent {
			let borrow: &RefCell<Scope> = parent.borrow();
			borrow.borrow().has_function(name)
		} else {
			false
		}
	}

	pub fn get_function(&self, name: &str) -> Option<Function> {
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

	pub fn set_function(&mut self, name: &str, function: Function) {
		if self.local_functions.contains_key(name) {
			*self.local_functions.get_mut(name).unwrap() = function
		} else {
			self.local_functions.insert(String::from(name), function);
		}
	}
}

#[derive(Clone)]
pub enum Function {
	Custom {
		body: Rc<Node>,
		scope_ref: Rc<RefCell<Scope>>,
	},
	BuiltIn {
		callback: Rc<dyn Fn(Vec<Data>, Option<Function>, &mut Scope) -> Data>,
	},
	Variable {
		value: Data,
		constant: bool,
		name: String,
	},
}

impl Function {
	pub fn call(
		&self,
		args: Vec<Data>,
		yield_fn: Option<Function>,
		scope: Rc<RefCell<Scope>>,
	) -> Data {
		match self {
			Function::Custom { body, scope_ref } => evaluate(body, Rc::clone(scope_ref)),
			Function::BuiltIn { callback } => {
				callback(args, yield_fn, &mut scope.borrow_mut())
			}
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
}