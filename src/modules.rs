use std::{any::Any, cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use crate::{
	data::Data,
	scope::{
		function::{CallScope, Function},
		Scope,
	},
};

pub mod collections;
pub mod runtime;

pub struct Module {
	functions: HashMap<
		String,
		Rc<dyn Fn(Vec<Data>, Option<Function>, Rc<RefCell<dyn Scope>>) -> Data>,
	>,
	submodules: HashMap<String, Rc<RefCell<Module>>>,
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
		F: Fn(Vec<Data>, Option<Function>, Rc<RefCell<dyn Scope>>) -> Data + 'static,
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
		self.submodules
			.insert(String::from(name), Rc::new(RefCell::new(module)));
		self
	}
}

impl Scope for Module {
	fn has_function(&self, name: &str) -> bool {
		self.functions.contains_key(name) || self.submodules.contains_key(name)
	}

	fn get_function(&self, name: &str) -> Option<Function> {
		self.functions
			.get(name)
			.map(|x| Function::BuiltIn {
				callback: Rc::clone(x),
			})
			.or_else(|| {
				self.submodules.get(name).map(|x: &Rc<RefCell<Module>>| {
					Function::Variable {
						value: Data::Scope(Rc::clone(x) as Rc<RefCell<dyn Scope>>),
						constant: true,
						name: String::new(),
					}
				})
			})
	}

	fn set_function(&mut self, _name: &str, _function: Function) {}
	fn delete_function(&mut self, _name: &str) {}
	fn set_return_value(&mut self, _value: Data) {
		panic!("Cannot return from module.")
	}

	fn get_call_scope(&self) -> Option<Rc<RefCell<CallScope>>> {
		None
	}

	fn as_any(&self) -> &dyn Any {
		self
	}
	fn as_mut(&mut self) -> &mut dyn Any {
		self
	}
}

impl Debug for Module {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Module")
			.field("functions", &self.functions.keys())
			.field("submodules", &self.submodules)
			.finish()
	}
}
