use std::{any::Any, cell::RefCell, fmt::Debug, rc::Rc};

use self::function::{CallScope, Function};

pub mod block_scope;
pub mod function;

pub trait Scope: Debug {
	fn has_function(&self, name: &str) -> bool;
	fn get_function(&self, name: &str) -> Option<Function>;
	fn set_function(&mut self, name: &str, function: Function);
	fn delete_function(&mut self, name: &str);

	fn parent(&self) -> Option<Rc<RefCell<dyn Scope>>> {
		None
	}
	fn get_call_scope(&self) -> Option<Rc<RefCell<CallScope>>>;

	fn as_any(&self) -> &dyn Any;
}
