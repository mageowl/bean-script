use std::collections::HashMap;
use std::{cell::RefCell, rc::Rc};

use crate::as_type;
use crate::modules::Data;
use crate::scope::function::CallScope;
use crate::scope::{function::Function, Scope};

#[derive(Debug)]
pub struct List {
	parent: Option<Rc<RefCell<dyn Scope>>>,
	fns: HashMap<String, Function>,
	pub items: Vec<Data>,
}

impl List {
	pub fn new(list: Vec<Data>, parent: Option<Rc<RefCell<dyn Scope>>>) -> Self {
		let mut list = List {
			parent,
			fns: HashMap::new(),
			items: list,
		};
		let mut make = |name: &str, closure| {
			list.fns
				.insert(String::from(name), Function::BuiltIn { callback: closure })
		};

		make(
			"size",
			Rc::new(|_a, _y, list: Rc<RefCell<dyn Scope>>| {
				Data::Number(
					as_type!(RefCell::borrow(&list) => List, 
					"Tried to call fn size on a non-list scope.")
					.items
					.len() as isize,
				)
			}),
		);
		make(
			"has",
			Rc::new(|args, _y, scope: Rc<RefCell<dyn Scope>>| {
				Data::Boolean(as_type!(RefCell::borrow(&scope) => List, "Tried to call fn has on a non-list scope.").items.contains(&args[0]))
			}),
		);

		list
	}
}

impl Scope for List {
	fn has_function(&self, _name: &str) -> bool {
		todo!()
	}

	fn get_function(&self, _name: &str) -> Option<Function> {
		todo!()
	}

	fn set_function(&mut self, _n: &str, _f: Function) {}

	fn delete_function(&mut self, _n: &str) {}

	fn get_call_scope(&self) -> Option<Rc<RefCell<CallScope>>>{
		if let Some(p) = &self.parent {
			RefCell::borrow(p).get_call_scope()
		} else {
			None
		}
	}

	fn parent(&self) -> Option<Rc<RefCell<dyn Scope>>> {
		self.parent.as_ref().map(|x| Rc::clone(x))
	}

	fn as_any(&self) -> &dyn std::any::Any {
		self
	}
}
