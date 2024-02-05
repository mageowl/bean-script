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

impl Data {
	pub fn type_str(&self) -> String {
		match self {
			Data::Boolean(_) => String::from("boolean"),
			Data::Number(_) => String::from("number"),
			Data::String(_) => String::from("string"),
			Data::Memory { scope, name } => String::from("memory"),
			Data::Scope(_) => String::from("scope"),
			Data::None => String::from("none"),
		}
	}
}
