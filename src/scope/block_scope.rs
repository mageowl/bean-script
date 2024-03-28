use std::{ any::Any, borrow::Borrow, cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc };

use crate::data::Data;

use super::{ function::{ CallScope, Function }, Scope, ScopeRef };

#[derive(Debug, Clone, Copy)]
pub enum IfState {
    Started,
    Captured,
    Finished,
}

pub struct BlockScope {
    local_functions: HashMap<String, Function>,
    parent: Option<ScopeRef>,
    did_break: bool,
    pub return_value: Data,
    pub if_state: IfState,
    pub match_value: Option<Data>,
}

impl BlockScope {
    pub fn new(parent: Option<ScopeRef>) -> Self {
        Self {
            local_functions: HashMap::new(),
            parent,
            return_value: Data::None,
            did_break: false,
            if_state: IfState::Finished,
            match_value: None,
        }
    }

    pub fn break_self(&mut self) {
        self.did_break = true;
    }

    pub fn did_break(&self) -> bool {
        self.did_break
    }
}

impl Debug for BlockScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockScope")
            .field("local_functions", &self.local_functions)
            .field("parent", &self.parent)
            .field("did_break", &self.did_break)
            .field("return_value", &self.return_value)
            .field("if_state", &self.if_state)
            .field("match_value", &self.match_value)
            .finish()
    }
}

impl Scope for BlockScope {
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
