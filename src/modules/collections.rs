use std::any::Any;
use std::collections::{HashMap, VecDeque};
use std::{cell::RefCell, rc::Rc};

use crate::data::{Data, StaticData};
use crate::{arg_check, as_mut_type, as_type};
use crate::scope::function::CallScope;
use crate::scope::{function::Function, Scope};

#[derive(Debug)]
pub struct List {
	parent: Option<Rc<RefCell<dyn Scope>>>,
	fns: HashMap<String, Function>,
	pub items: VecDeque<Data>,
}

impl List {
	pub fn new(list: Vec<Data>, parent: Option<Rc<RefCell<dyn Scope>>>) -> Self {
		let mut list = List {
			parent,
			fns: HashMap::new(),
			items: VecDeque::from(list),
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
					.len() as f64,
				)
			}),
		);
		make(
			"empty",
			Rc::new(|_a, _y, list: Rc<RefCell<dyn Scope>>| {
				Data::Boolean(as_type!(RefCell::borrow(&list) => List, "Tried to call fn empty on a non-list scope.").items.is_empty())
			}),
		);
		make(
			"has",
			Rc::new(|args, _y, list: Rc<RefCell<dyn Scope>>| {
				Data::Boolean(
					as_type!(RefCell::borrow(&list) => List, 
						"Tried to call fn has on a non-list scope.")
						.items.contains(&args[0])
				)
			}),
		);
		make(
			"at",
			Rc::new(|args, _y, list: Rc<RefCell<dyn Scope>>| {
					arg_check!(&args[0], Data::Number(i) => "Expected a number for fn at, but instead got {}.");
					as_type!(RefCell::borrow(&list) => List, 
						"Tried to call fn has on a non-list scope.")
						.items[*i as usize]
						.clone()
			}),
		);
		make(
			"push",
			Rc::new(|args, _y, list: Rc<RefCell<dyn Scope>>| {
				as_mut_type!(RefCell::borrow_mut(&list) => List,
						"Tried to call fn push on a non-list scope.")
					.items
					.push_back(args[0].clone());
				Data::None
			}),
		);
		make("concat",
			Rc::new(|args, _y, list: Rc<RefCell<dyn Scope>>| {
				arg_check!(&args[0], Data::Scope(list2) => "Expected scope for fn concat, but instead got {}.");
				as_mut_type!(RefCell::borrow_mut(&list) => List,
						"Tried to call fn concat on a non-list scope.")
					.items
					.append(&mut as_type!(RefCell::borrow(&list2) => List,
						"Tried to call fn concat with a non-list scope.").items.clone());
				Data::None
			})
		);
		make(
			"pop",
			Rc::new(|_a, _y, list: Rc<RefCell<dyn Scope>>| {
				as_mut_type!(RefCell::borrow_mut(&list) => List,
						"Tried to call fn pop on a non-list scope.")
					.items
					.pop_back().unwrap_or_default()
			}),
		);
		make(
			"delete",
			Rc::new(|args, _y, list: Rc<RefCell<dyn Scope>>| {
				arg_check!(&args[0], Data::Number(i) => "Expected number for fn delete, but instead got {}.");
				as_mut_type!(RefCell::borrow_mut(&list) => List,
						"Tried to call fn delete on a non-list scope.")
					.items
					.remove(*i as usize).unwrap_or_default()
			}),
		);
		make(
			"insert",
			Rc::new(|args, _y, list: Rc<RefCell<dyn Scope>>| {
				arg_check!(&args[0], Data::Number(i) => "Expected number for fn insert, but instead got {}.");
				as_mut_type!(RefCell::borrow_mut(&list) => List,
						"Tried to call fn delete on a non-list scope.")
					.items
					.insert(*i as usize, args[1].clone());
				Data::None
			}),
		);

		make(
			"for",
			Rc::new(|args, yield_fn, list: Rc<RefCell<dyn Scope>>| {
				let yield_fn = yield_fn.expect("Expected yield block for fn for.");
				arg_check!(&args[0], Data::Memory { scope: item_scope_ref, name: item_name } =>
					"Expected memory for fn for, but instead got {}.");
				let mut item_scope = RefCell::borrow_mut(&item_scope_ref);

				let mut mapped: VecDeque<Data> = VecDeque::new();

				for item in &as_mut_type!(RefCell::borrow_mut(&list) => List,
						"Tried to call fn delete on a non-list scope.").items {
					item_scope.set_function(&item_name, Function::Variable 
						{ value: item.clone(), constant: true, name: item_name.clone() });
					mapped.push_back(yield_fn.call(Vec::new(), None, Rc::clone(&list)))
				}

				Data::None
			})
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

	fn get_call_scope(&self) -> Option<Rc<RefCell<CallScope>>> {
		if let Some(p) = &self.parent {
			RefCell::borrow(p).get_call_scope()
		} else {
			None
		}
	}

	fn set_return_value(&mut self, _value: Data) {}
	fn get_function_list(&self) -> HashMap<String, Function> {
		HashMap::new()
	}

	fn parent(&self) -> Option<Rc<RefCell<dyn Scope>>> {
		self.parent.as_ref().map(|x| Rc::clone(x))
	}

	fn as_any(&self) -> &dyn Any {
		self
	}
	fn as_mut(&mut self) -> &mut dyn Any {
		self
	}

	fn to_string(&self) -> String {
		format!("[{}]", self.items.iter().fold(
			String::new(), 
			|s, e| (if s.is_empty() { String::new() } else { s + ", " }) + &e.to_string()
		))
	}
}

#[derive(Debug)]
pub struct Map {
	pub parent: Option<Rc<RefCell<dyn Scope>>>,
	fns: HashMap<String, Function>,
	pub hash: HashMap<StaticData, Data>,
}

impl Map {
	pub fn new(list: Vec<Data>, parent: Option<Rc<RefCell<dyn Scope>>>) -> Self {
		let mut map = Map {
			parent,
			fns: HashMap::new(),
			hash: list.chunks(2)
					.map(|pair| 
						(
							StaticData::from(pair.get(0).expect("Number of arguments must be even for fn map.").clone()),
							pair.get(1).expect("Number of arguments must be even for fn map.").clone()
						))
					.collect::<HashMap<StaticData, Data>>(),
		};
		let mut make = |name: &str, closure| {
			map.fns
				.insert(String::from(name), Function::BuiltIn { callback: closure })
		};

		make(
			"size",
			Rc::new(|_a, _y, map: Rc<RefCell<dyn Scope>>| {
				Data::Number(
					as_type!(RefCell::borrow(&map) => Map, 
					"Tried to call fn size on a non-map scope.")
					.hash
					.len() as f64,
				)
			}),
		);
		make(
			"empty",
			Rc::new(|_a, _y, map: Rc<RefCell<dyn Scope>>| {
				Data::Boolean(
					as_type!(RefCell::borrow(&map) => Map, 
					"Tried to call fn size on a non-map scope.")
					.hash
					.is_empty(),
				)
			}),
		);
		make(
			"has",
			Rc::new(|args, _y, map: Rc<RefCell<dyn Scope>>| {
				Data::Boolean(as_type!(RefCell::borrow(&map) => Map, "Tried to call fn has on a non-map scope.").hash.contains_key(&StaticData::from(args[0].clone())))
			}),
		);

		map
	}
}