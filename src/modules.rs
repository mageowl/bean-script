use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
	data::Data,
	scope::{Function, Scope},
};

pub mod runtime;

pub struct Module {
	functions: HashMap<
		String,
		Rc<dyn Fn(Vec<Data>, Option<Function>, Rc<RefCell<Scope>>) -> Data>,
	>,
	submodules: HashMap<String, Box<Module>>,
}

impl Module {
	pub fn new<F>(constructor: F) -> Self
	where
		F: FnOnce(&mut Module),
	{
		let mut module = Module {
			functions: HashMap::new(),
			submodules: HashMap::new(),
		};
		constructor(&mut module);
		module
	}

	pub fn function<F>(&mut self, name: &str, function: F) -> &mut Self
	where
		F: Fn(Vec<Data>, Option<Function>, Rc<RefCell<Scope>>) -> Data + 'static,
	{
		self.functions.insert(String::from(name), Rc::new(function));
		self
	}

	pub fn submodule<F>(&mut self, name: &str, constructor: F) -> &mut Self
	where
		F: FnOnce(&mut Module),
	{
		let mut module = Module {
			functions: HashMap::new(),
			submodules: HashMap::new(),
		};
		constructor(&mut module);
		self.submodules.insert(String::from(name), Box::new(module));
		self
	}

	pub fn as_scope(&self) -> Scope {
		let mut scope = Scope::new(None);

		for (n, f) in &self.functions {
			scope.set_function(
				&n,
				Function::BuiltIn {
					callback: Rc::clone(f),
				},
			)
		}

		for (n, m) in &self.submodules {
			scope.set_function(
				&n,
				Function::Variable {
					value: Data::Scope(Rc::new(RefCell::new(m.as_scope()))),
					constant: true,
					name: String::from(n),
				},
			)
		}

		scope
	}
}
