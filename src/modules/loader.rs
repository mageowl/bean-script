use core::panic;
use std::{any::Any, collections::HashMap, fs, path::PathBuf, rc::Rc};

use crate::{
	data::Data,
	error::{Error, ErrorSource},
	evaluator, lexer, parser,
	scope::{function::Function, Scope},
	util::{make_ref, MutRc},
};

use super::{
	registry::{ModuleRegistry, RegistryEntry},
	CustomModule, Module,
};

#[derive(Debug)]
pub struct ModuleWrapper(MutRc<dyn Module>);

impl Scope for ModuleWrapper {
	fn has_function(&self, name: &str) -> bool {
		self.0.borrow().has_pub_function(name)
	}

	fn get_function(&self, name: &str) -> Option<Function> {
		self.0.borrow().get_pub_function(name)
	}

	fn set_function(&mut self, _name: &str, _function: Function) {
		panic!("Tried to set function inside external module.")
	}

	fn delete_function(&mut self, _name: &str) {
		panic!("Tried to delete function inside external module.")
	}

	fn set_return_value(&mut self, _value: Data) {}

	fn get_function_list(&self) -> HashMap<String, Function> {
		self.0.borrow().get_function_list()
	}

	fn as_any(&self) -> &dyn Any {
		panic!("INTERNAL! tried to cast ModuleWrapper")
	}
	fn as_mut(&mut self) -> &mut dyn Any {
		self
	}
}

pub fn get(module: &CustomModule, path: String) -> Result<MutRc<ModuleWrapper>, Error> {
	let registry = module.registry.clone();

	if path.starts_with("./") {
		let mut path_buf = module.file_path.clone();
		path_buf.push(path.clone() + ".bean");

		get_local(Rc::clone(&registry), path_buf).map(|m| make_ref(ModuleWrapper(m)))
	} else {
		get_reg(&mut registry.borrow_mut().registered, path.clone()).map_or(
			Err(Error::new(
				&format!("Module {} does not exist.", path),
				ErrorSource::Unknown,
			)),
			|s| Ok(make_ref(ModuleWrapper(s))),
		)
	}
}

fn get_reg(
	registered: &mut HashMap<String, RegistryEntry>,
	name: String,
) -> Option<MutRc<dyn Module>> {
	if let Some(RegistryEntry::Uninitialized(_)) = registered.get(&name) {
		let v = registered.remove(&name).unwrap().get_or_init();
		registered.insert(name.clone(), RegistryEntry::Available(v));
	}

	registered.get(&name).map_or(None, |x| match x {
		RegistryEntry::Available(r) => Some(Rc::clone(r)),
		RegistryEntry::Uninitialized(_) => None,
	})
}

fn get_local(
	registry: MutRc<ModuleRegistry>,
	path: PathBuf,
) -> Result<MutRc<CustomModule>, Error> {
	if registry.borrow().loading.contains(&path) {
		return Err(Error::new(
			"Trying to load from a file that is currently being loaded.",
			ErrorSource::Unknown,
		));
	}
	let exists = registry.borrow().local.get(&path).is_none();
	if exists {
		let file = fs::read_to_string(path.clone()).map_err(|e| {
			Error::new(
				&(String::from("Error reading file ")
					+ path.to_str().unwrap_or("")
					+ ": " + &e.to_string()),
				ErrorSource::Unknown,
			)
		})?;

		let tokens = lexer::tokenize(file);
		let tree = parser::parse(tokens);

		let module = CustomModule::new(Rc::clone(&registry), path.clone());
		let module_ref = make_ref(module);
		registry.borrow_mut().loading.push(path.clone());

		evaluator::evaluate(&tree, CustomModule::to_scope(Rc::clone(&module_ref)));

		let mut registry_mut = registry.borrow_mut();
		registry_mut.local.insert(path.clone(), module_ref);
		registry_mut.loading.pop();
	}

	Ok(Rc::clone(registry.borrow().local.get(&path).unwrap()))
}
