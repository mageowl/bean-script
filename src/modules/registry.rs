use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};

use crate::util::{make_ref, MutRc};

use super::{bean_std, BuiltinModule, CustomModule, Module};

pub(super) enum RegistryEntry {
	Uninitialized(Box<dyn FnOnce() -> MutRc<dyn Module>>),
	Available(MutRc<dyn Module>),
}

impl RegistryEntry {
	pub fn get_or_init(self) -> MutRc<dyn Module> {
		match self {
			Self::Uninitialized(init) => init(),
			Self::Available(v) => v,
		}
	}
}

pub struct ModuleRegistry {
	pub(super) registered: HashMap<String, RegistryEntry>,
	pub(super) local: HashMap<PathBuf, MutRc<CustomModule>>,
	pub(super) loading: Vec<PathBuf>,
	runtime: MutRc<BuiltinModule>,
}

impl std::fmt::Debug for ModuleRegistry {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ModuleRegistry").finish()
	}
}

impl ModuleRegistry {
	pub fn new() -> Self {
		let standard_lib = BuiltinModule::new(bean_std::construct);
		let mut s = Self {
			registered: HashMap::new(),
			local: HashMap::new(),
			loading: Vec::new(),
			runtime: match RefCell::borrow(
				&standard_lib.get_submodule("runtime").unwrap(),
			)
			.as_any()
			.downcast_ref::<BuiltinModule>()
			{
				Some(r) => Rc::new(RefCell::new(r.clone())),
				None => panic!("Runtime module is custom?"),
			},
		};
		s.registered.insert(
			String::from("std"),
			RegistryEntry::Available(make_ref(standard_lib)),
		);
		s
	}

	pub fn register_builtin(
		&mut self,
		name: String,
		constructor: fn(&mut BuiltinModule),
	) {
		if !self.registered.contains_key(&name) {
			self.registered.insert(
				name,
				RegistryEntry::Uninitialized(Box::new(move || {
					make_ref(BuiltinModule::new(constructor))
				})),
			);
		} else {
			panic!("Trying to register a builtin module under a used name.")
		}
	}

	pub fn runtime(&self) -> MutRc<BuiltinModule> {
		Rc::clone(&self.runtime)
	}
}
