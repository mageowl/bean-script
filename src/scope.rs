use std::{any::Any, cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use crate::data::Data;
use function::{CallScope, Function};

use self::block_scope::IfState;

pub mod block_scope;
pub mod function;

pub type ScopeRef = Rc<RefCell<dyn Scope>>;

pub trait Scope: Debug {
	fn has_function(&self, name: &str) -> bool;
	fn get_function(&self, name: &str) -> Option<Function>;
	fn set_function(&mut self, name: &str, function: Function);
	fn delete_function(&mut self, name: &str);

	fn parent(&self) -> Option<ScopeRef> {
		None
	}

	fn get_call_scope(&self) -> Option<Rc<RefCell<CallScope>>> {
		self.parent().map_or(None, |p| p.borrow().get_call_scope())
	}
	fn get_file_module(&self) -> Option<ScopeRef> {
		self.parent().map_or(None, |p| p.borrow().get_file_module())
	}
	fn set_return_value(&mut self, value: Data);
	fn set_if_state(&mut self, state: IfState);
	fn get_if_state(&self) -> Option<IfState> {
		None
	}
	fn get_function_list(&self) -> HashMap<String, Function>;

	fn as_any(&self) -> &dyn Any;
	fn as_mut(&mut self) -> &mut dyn Any;

	fn to_string(&self) -> String {
		String::from("[scope]")
	}
}
