use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
	data::Data,
	scope::{Function, Scope},
};

pub mod runtime;

pub struct Module {
	functions: HashMap<
		String,
		Rc<dyn Fn(Vec<Data>, Option<Function>, Rc<RefCell<dyn Scope>>) -> Data>,
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
		self.submodules.insert(String::from(name), Box::new(module));
		self
	}
}

impl Scope for Module {
	fn has_function(&self, name: &str) -> bool {
		self.functions.contains_key(name)
	}

	fn get_function(&self, name: &str) -> Option<Function> {
		self.functions.get(name).map(|x| Function::BuiltIn { callback: x })
	}

	fn set_function(&self, name: &str, function: Function) {}
	fn delete_function(&self, name: &str) {}
}
