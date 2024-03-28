use std::{ any::Any, cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc };

use crate::{
    data::Data,
    scope::{ block_scope::IfState, function::{ CallScope, Function }, Scope, ScopeRef },
};

pub mod collections;
pub mod runtime;

pub struct Module {
    functions: HashMap<String, Rc<dyn Fn(Vec<Data>, Option<Function>, ScopeRef) -> Data>>,
    submodules: HashMap<String, Rc<RefCell<Module>>>,
}

impl Module {
    pub fn new(constructor: fn(&mut Module)) -> Self {
        let mut module = Module {
            functions: HashMap::new(),
            submodules: HashMap::new(),
        };
        constructor(&mut module);
        module
    }

    pub fn function<F>(&mut self, name: &str, function: F) -> &mut Self
        where F: Fn(Vec<Data>, Option<Function>, ScopeRef) -> Data + 'static
    {
        self.functions.insert(String::from(name), Rc::new(function));
        self
    }

    pub fn submodule<F>(&mut self, name: &str, constructor: F) -> &mut Self
        where F: FnOnce(&mut Module)
    {
        let mut module = Module {
            functions: HashMap::new(),
            submodules: HashMap::new(),
        };
        constructor(&mut module);
        self.submodules.insert(String::from(name), Rc::new(RefCell::new(module)));
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
                        value: Data::Scope(Rc::clone(x) as ScopeRef),
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

    fn get_function_list(&self) -> HashMap<String, Function> {
        let mut map = HashMap::new();
        for (k, fun) in &self.functions {
            map.insert(k.clone(), Function::BuiltIn {
                callback: Rc::clone(fun),
            });
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

impl Debug for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Module")
            .field("functions", &self.functions.keys())
            .field("submodules", &self.submodules)
            .finish()
    }
}

pub struct CustomModule {
    local_functions: HashMap<String, Function>,
    pub if_state: IfState,
    pub exported_functions: HashMap<String, Function>,
    pub submodules: HashMap<String, CustomModule>,
}

impl CustomModule {
    pub fn new() {
        CustomModule {
            local_functions: HashMap::new(),
            if_state: IfState::Captured,
            exported_functions: HashMap::new(),
            submodules: HashMap::new(),
        }
    }
}

impl Scope for CustomModule {
    fn has_function(&self, name: &str) -> bool {
        if self.local_functions.contains_key(name) {
            true
        } else if let Some(parent) = &self.parent {
            let borrow: &RefCell<dyn Scope> = parent.borrow();
            borrow.borrow().has_function(name)
        } else {
            false
        }
    }

    fn get_function(&self, name: &str) -> Option<Function> {
        let function = self.local_functions.get(name);
        if function.is_some() {
            function.map(|x| x.clone())
        } else if let Some(parent) = &self.parent {
            let borrow: &RefCell<dyn Scope> = parent.borrow();
            borrow
                .borrow()
                .get_function(name)
                .map(|x| x.clone())
        } else {
            None
        }
    }

    fn set_function(&mut self, name: &str, function: Function) {
        if self.local_functions.contains_key(name) {
            *self.local_functions.get_mut(name).unwrap() = function;
        } else {
            self.local_functions.insert(String::from(name), function);
        }
    }

    fn delete_function(&mut self, name: &str) {
        self.local_functions.remove(name);
    }

    fn parent(&self) -> Option<ScopeRef> {
        self.parent.as_ref().map(|x| Rc::clone(x))
    }

    fn get_call_scope(&self) -> Option<Rc<RefCell<CallScope>>> {
        if let Some(p) = &self.parent { RefCell::borrow(&p).get_call_scope() } else { None }
    }

    fn set_return_value(&mut self, value: Data) {
        self.return_value = value;
    }

    fn get_function_list(&self) -> HashMap<String, Function> {
        self.local_functions.clone()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut(&mut self) -> &mut dyn Any {
        self
    }
}
