use std::{
	cell::{Ref, RefCell},
	collections::HashMap,
	rc::Rc,
};

use crate::{data::Data, parser::Node};

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

	pub fn get_function(&self, name: &str) -> Option<Function> {
		let function = self.local_functions.get(name);
		if function.is_some() {
			function.map(|x| x.clone())
		} else if let Some(parent) = &self.parent {
			parent.borrow().get_function(name).map(|x| x.clone())
		} else {
			None
		}
	}
}

#[derive(Clone)]
pub enum Function {
	Custom {
		body: Rc<Vec<Node>>,
	},
	BuiltIn {
		callback: Rc<dyn Fn(Vec<&Data>, Function, &mut Scope) -> Data>,
	},
	Variable {
		value: Data,
		constant: bool,
	},
}

impl Function {
	pub fn call(
		&self,
		args: Vec<&Data>,
		yield_fn: Option<Function>,
		scope: Rc<RefCell<Scope>>,
	) -> Data {
		todo!()
	}
}
