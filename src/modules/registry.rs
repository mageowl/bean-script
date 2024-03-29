use std::{ cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc };

use once_cell::sync::Lazy;

use super::{ CustomModule, Module };

enum RegistryEntry {
    Uninitialized(Box<dyn FnOnce() -> Rc<RefCell<dyn Module>>>),
    Available(Rc<RefCell<dyn Module>>),
}

pub struct ModuleRegistry {
    registered: HashMap<String, RegistryEntry>,
    local: HashMap<PathBuf, CustomModule>,
}

impl ModuleRegistry {
    fn new() -> Self {
        Self { registered: HashMap::new(), local: HashMap::new() }
    }

    pub fn get(&mut self, name: String) -> Option<Rc<RefCell<dyn Module>>> {
        let registered = &mut self.registered;
        match registered.get(&name) {
            Some(e) => {
                Some(match e {
                    RegistryEntry::Available(module_ref) => Rc::clone(module_ref),
                    RegistryEntry::Uninitialized(_) => {
                        let value = match registered.remove(&name).unwrap() {
                            RegistryEntry::Uninitialized(init) => init(),
                            RegistryEntry::Available(_) => panic!("idk why this happened."),
                        };
                        registered.insert(name, RegistryEntry::Available(Rc::clone(&value)));
                        value
                    }
                })
            }
            None => None,
        }
    }

    pub fn get_local(&mut self, path: PathBuf) -> Option<CustomModule> {}
}

pub fn registry() -> &'static mut ModuleRegistry {
    static mut REGISTRY: Lazy<ModuleRegistry> = Lazy::new(|| ModuleRegistry::new());
    unsafe { &mut REGISTRY }
}
