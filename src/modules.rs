use std::{
	any::Any, cell::RefCell, collections::HashMap, fmt::Debug, path::PathBuf, rc::Rc,
};

use crate::{
	data::Data,
	scope::{block_scope::IfState, function::Function, Scope, ScopeRef},
	util::{make_ref, MutRc},
};

use self::registry::ModuleRegistry;

pub mod bean_std;
pub mod loader;
pub mod registry;

pub trait Module: Scope {
	fn get_submodule(&self, name: &str) -> Option<MutRc<dyn Module>>;

	fn has_pub_function(&self, name: &str) -> bool;
	fn get_pub_function(&self, name: &str) -> Option<Function>;
}

#[derive(Clone)]
pub struct BuiltinModule {
	functions: HashMap<String, Rc<dyn Fn(Vec<Data>, Option<Function>, ScopeRef) -> Data>>,
	submodules: HashMap<String, Rc<RefCell<BuiltinModule>>>,
}

impl BuiltinModule {
	pub fn new(constructor: fn(&mut BuiltinModule)) -> Self {
		let mut module = BuiltinModule {
			functions: HashMap::new(),
			submodules: HashMap::new(),
		};
		constructor(&mut module);
		module
	}

	pub fn function<F>(&mut self, name: &str, function: F) -> &mut Self
	where
		F: Fn(Vec<Data>, Option<Function>, ScopeRef) -> Data + 'static,
	{
		self.functions.insert(String::from(name), Rc::new(function));
		self
	}

	pub fn submodule<F>(&mut self, name: &str, constructor: F) -> &mut Self
	where
		F: FnOnce(&mut BuiltinModule),
	{
		let mut module = BuiltinModule {
			functions: HashMap::new(),
			submodules: HashMap::new(),
		};
		constructor(&mut module);
		self.submodules
			.insert(String::from(name), Rc::new(RefCell::new(module)));
		self
	}
}

impl Scope for BuiltinModule {
	fn has_function(&self, name: &str) -> bool {
		self.functions.contains_key(name)
	}

	fn get_function(&self, name: &str) -> Option<Function> {
		self.functions.get(name).map(|x| Function::BuiltIn {
			callback: Rc::clone(x),
		})
	}

	fn set_function(&mut self, _name: &str, _function: Function) {}
	fn delete_function(&mut self, _name: &str) {}
	fn set_return_value(&mut self, _value: Data) {
		panic!("Cannot return from module.")
	}

	fn get_function_list(&self) -> HashMap<String, Function> {
		let mut map = HashMap::new();
		for (k, fun) in &self.functions {
			map.insert(
				k.clone(),
				Function::BuiltIn {
					callback: Rc::clone(fun),
				},
			);
		}
		map
	}

	fn as_any(&self) -> &dyn Any {
		self
	}
	fn as_mut(&mut self) -> &mut dyn Any {
		self
	}
}

impl Debug for BuiltinModule {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Module")
			.field("functions", &self.functions.keys())
			.field("submodules", &self.submodules)
			.finish()
	}
}

impl Module for BuiltinModule {
	fn get_submodule(&self, name: &str) -> Option<MutRc<dyn Module>> {
		self.submodules
			.get(name)
			.map(|rc| Rc::clone(rc) as MutRc<dyn Module>)
	}

	fn has_pub_function(&self, name: &str) -> bool {
		self.has_function(name)
	}

	fn get_pub_function(&self, name: &str) -> Option<Function> {
		self.get_function(name)
	}
}

#[derive(Debug, Clone)]
pub struct CustomModule {
	local_functions: MutRc<HashMap<String, Function>>,
	registry: MutRc<ModuleRegistry>,
	pub file_path: PathBuf,
	pub if_state: IfState,
	pub exported_functions: MutRc<HashMap<String, Function>>,
	pub submodules: MutRc<HashMap<String, MutRc<CustomModule>>>,
}

impl CustomModule {
	pub fn new(registry: MutRc<ModuleRegistry>, file_path: PathBuf) -> Self {
		Self {
			local_functions: make_ref(HashMap::new()),
			registry,
			file_path,
			if_state: IfState::Captured,
			exported_functions: make_ref(HashMap::new()),
			submodules: make_ref(HashMap::new()),
		}
	}

	// courtesy of [stack overflow](https://stackoverflow.com/a/64400756)
	fn to_scope(it: MutRc<Self>) -> ScopeRef {
		it
	}
}

impl Scope for CustomModule {
	fn has_function(&self, name: &str) -> bool {
		self.local_functions.borrow().contains_key(name)
	}

	fn get_function(&self, name: &str) -> Option<Function> {
		let binding = RefCell::borrow(&self.local_functions);
		let function = binding.get(name);
		function.map(|x| x.clone()).or_else(|| {
			RefCell::borrow(&self.registry.borrow().runtime()).get_function(name)
		})
	}

	fn set_function(&mut self, name: &str, function: Function) {
		RefCell::borrow_mut(&self.local_functions).insert(String::from(name), function);
	}

	fn delete_function(&mut self, name: &str) {
		RefCell::borrow_mut(&self.local_functions).remove(name);
	}

	fn parent(&self) -> Option<ScopeRef> {
		None
	}

	fn set_return_value(&mut self, _v: Data) {
		panic!("Attempted to return from root scope.");
	}

	fn get_function_list(&self) -> HashMap<String, Function> {
		self.exported_functions.borrow().clone()
	}

	fn as_any(&self) -> &dyn Any {
		self
	}
	fn as_mut(&mut self) -> &mut dyn Any {
		self
	}
}

impl Module for CustomModule {
	fn get_submodule(&self, name: &str) -> Option<Rc<RefCell<dyn Module>>> {
		self.submodules
			.borrow()
			.get(name)
			.map(|rc| Rc::clone(rc) as MutRc<dyn Module>)
	}

	fn has_pub_function(&self, name: &str) -> bool {
		self.exported_functions.borrow().contains_key(name)
	}

	fn get_pub_function(&self, name: &str) -> Option<Function> {
		self.exported_functions.borrow().get(name).cloned()
	}
}
