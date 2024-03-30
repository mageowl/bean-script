use std::{ cell::RefCell, collections::HashMap, fs, ops::Deref, path::PathBuf, rc::Rc };

use once_cell::sync::Lazy;

use crate::{ error::BeanError, evaluator, lexer, parser, util::MutRc };

use super::{ BuiltinModule, CustomModule, Module };

enum RegistryEntry {
    Uninitialized(Box<dyn FnOnce() -> Box<dyn Module>>),
    Available(Box<dyn Module>),
}

impl RegistryEntry {
    fn get_or_init(self) -> Box<dyn Module> {
        match self {
            Self::Uninitialized(init) => init(),
            Self::Available(v) => v,
        }
    }
}

pub struct ModuleRegistry {
    registered: HashMap<String, RegistryEntry>,
    local: HashMap<PathBuf, CustomModule>,
    loading: Vec<PathBuf>,
    runtime: Option<MutRc<BuiltinModule>>,
}

impl ModuleRegistry {
    fn new() -> Self {
        Self {
            registered: HashMap::new(),
            local: HashMap::new(),
            loading: Vec::new(),
            runtime: None,
        }
    }

    pub fn get(&mut self, path: String) -> Result<Box<&dyn Module>, BeanError> {
        if path.starts_with("./") {
            self.get_local(PathBuf::from(path.clone().trim_start_matches("./"))).map(|m|
                Box::new(m as &dyn Module)
            )
        } else {
            self.get_reg(&path).map_or(
                Err(BeanError::new(&format!("Module {} does not exist.", path), None)),
                |s| Ok(Box::new(s))
            )
        }
    }

    fn get_reg(&mut self, name: &str) -> Option<&dyn Module> {
        let registered = &mut self.registered;

        if let Some(RegistryEntry::Uninitialized(_)) = registered.get(name) {
            let v = registered.remove(name).unwrap().get_or_init();
            registered.insert(String::from(name), RegistryEntry::Available(v));
        }

        registered.get(name).map_or(None, |x| {
            match x {
                RegistryEntry::Available(r) => Some(Box::deref(r)),
                RegistryEntry::Uninitialized(_) => None,
            }
        })
    }

    fn get_local(&mut self, path: PathBuf) -> Result<&CustomModule, BeanError> {
        if self.loading.contains(&path) {
            return Err(
                BeanError::new("Trying to load from a file that is currently being loaded.", None)
            );
        }
        match self.local.get(&path) {
            None => {
                let file = fs
                    ::read_to_string(path.clone())
                    .map_err(|e|
                        BeanError::new(
                            &(
                                String::from("Error reading file ") +
                                path.to_str().unwrap_or("") +
                                ":" +
                                &e.to_string()
                            ),
                            None
                        )
                    )?;

                let tokens = lexer::tokenize(file);
                let tree = parser::parse(tokens);

                let runtime = Rc::new(
                    RefCell::new(
                        (*(match
                            self
                                .get(String::from("std/runtime"))
                                .expect("Standard module not found.")
                                .as_any()
                                .downcast_ref::<BuiltinModule>()
                        {
                            Some(t) => t,
                            None =>
                                panic!(
                                    "Standard module is not a builtin. Something is very wrong."
                                ),
                        })).clone()
                    )
                );
                let module = CustomModule::new(runtime, path.clone());
                let module_ref = Rc::new(RefCell::new(module));
                evaluator::evaluate(&tree, CustomModule::to_scope(Rc::clone(&module_ref)));
                self.local.insert(path.clone(), Rc::try_unwrap(module_ref).unwrap().into_inner());
            }
            Some(_) => (),
        }

        Ok(self.local.get(&path).unwrap())
    }

    pub fn register_builtin(&mut self, name: String, constructor: fn(&mut BuiltinModule)) {
        if name == "std" && !self.registered.contains_key("std") {
            let module = BuiltinModule::new(constructor);
            self.runtime = match
                RefCell::borrow(&module.get_submodule("runtime").unwrap())
                    .as_any()
                    .downcast_ref::<BuiltinModule>()
            {
                None => panic!("Runtime submodule not builtin."),
                Some(v) => Some(Rc::new(RefCell::new(v.clone()))),
            };
            self.registered.insert(name, RegistryEntry::Available(Box::new(module)));
        } else if !self.registered.contains_key(&name) {
            self.registered.insert(
                name,
                RegistryEntry::Uninitialized(
                    Box::new(move || Box::new(BuiltinModule::new(constructor)))
                )
            );
        }
    }

    pub fn runtime(&self) -> MutRc<BuiltinModule> {
        Rc::clone(self.runtime.as_ref().unwrap())
    }
}

pub fn instance() -> &'static mut ModuleRegistry {
    static mut REGISTRY: Lazy<ModuleRegistry> = Lazy::new(|| ModuleRegistry::new());
    unsafe { &mut REGISTRY }
}
