use std::{cell::RefCell, rc::Rc};

use crate::scope::Scope;

#[derive(Clone)]
pub enum Data {
	Boolean(bool),
	Number(isize),
	String(String),
	Memory {
		scope: Rc<RefCell<Scope>>,
		name: String,
	},
	Scope(Rc<RefCell<Scope>>),
	None,
}
