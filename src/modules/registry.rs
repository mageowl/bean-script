use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc};

use crate::{
	logger::Logger,
	util::{make_ref, MutRc},
};

use super::{bean_std, BuiltinModule, CustomModule, Module, ModuleBuilder};

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

#[derive(Clone, Copy)]
pub struct RegistryFeatures {
	pub custom_modules: bool,
	pub import: bool,
	pub lang_debug: bool,
}

impl Default for RegistryFeatures {
	fn default() -> Self {
		Self {
			custom_modules: true,
			import: true,
			lang_debug: false,
		}
	}
}

pub struct ModuleRegistry {
	pub(super) registered: HashMap<String, RegistryEntry>,
	pub(super) local: HashMap<PathBuf, MutRc<CustomModule>>,
	pub(super) loading: Vec<PathBuf>,
	runtime: MutRc<BuiltinModule>,
	pub features: RegistryFeatures,
	pub logger: Logger,
}

impl std::fmt::Debug for ModuleRegistry {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ModuleRegistry").finish()
	}
}

impl ModuleRegistry {
	pub fn new(features: RegistryFeatures) -> Self {
		let standard_lib = BuiltinModule::new(bean_std::construct, features);
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
			features: RegistryFeatures::default(),
			logger: Logger::Stdout,
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
		constructor: fn(&mut ModuleBuilder),
	) {
		if !self.registered.contains_key(&name) {
			let features = self.features.clone();
			self.registered.insert(
				name,
				RegistryEntry::Uninitialized(Box::new(move || {
					make_ref(BuiltinModule::new(constructor, features))
				})),
			);
		} else {
			panic!("Trying to register a builtin module under a used name.")
		}
	}

	pub fn register_initialized_builtin(&mut self, name: String, module: BuiltinModule) {
		if !self.registered.contains_key(&name) {
			self.registered
				.insert(name, RegistryEntry::Available(make_ref(module)));
		} else {
			panic!("Trying to register a builtin module under a used name.")
		}
	}

	pub fn set_logger(&mut self, logger: Logger) {
		self.logger = logger;
	}

	pub fn runtime(&self) -> MutRc<BuiltinModule> {
		Rc::clone(&self.runtime)
	}
}
