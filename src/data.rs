use std::{cell::RefCell, rc::Rc};

use crate::{pat_check, scope::Scope};

pub enum DataType {
	Boolean,
	Number,
	String,
	Memory,
	Scope,
	None,
	Any,
}

impl DataType {
	pub fn from_string(string: &String) -> DataType {
		if string == "boolean" {
			DataType::Boolean
		} else if string == "number" {
			DataType::Number
		} else if string == "string" {
			DataType::String
		} else if string == "memory" {
			DataType::Memory
		} else if string == "scope" {
			DataType::Scope
		} else if string == "none" {
			DataType::None
		} else if string == "any" {
			DataType::Any
		} else {
			panic!("Invalid type string '{}'.", string)
		}
	}

	pub fn to_string(&self) -> String {
		match self {
			DataType::Boolean => String::from("boolean"),
			DataType::Number => String::from("number"),
			DataType::String => String::from("string"),
			DataType::Memory => String::from("memory"),
			DataType::Scope => String::from("scope"),
			DataType::None => String::from("none"),
			DataType::Any => String::from("any"),
		}
	}

	pub fn matches(&self, data: &Data) -> bool {
		match self {
			DataType::Boolean => pat_check!(Data::Boolean(_) = data),
			DataType::Number => pat_check!(Data::Number(_) = data),
			DataType::String => pat_check!(Data::String(_) = data),
			DataType::Memory => pat_check!(Data::Memory { .. } = data),
			DataType::Scope => pat_check!(Data::Scope(_) = data),
			DataType::None => pat_check!(Data::None = data),
			DataType::Any => true,
		}
	}
}

#[derive(Clone, Debug)]
pub enum Data {
	Boolean(bool),
	Number(isize),
	String(String),
	Memory {
		scope: Rc<RefCell<dyn Scope>>,
		name: String,
	},
	Scope(Rc<RefCell<dyn Scope>>),
	None,
}

impl Data {
	pub fn to_string(&self) -> String {
		match self {
			Data::Boolean(v) => {
				if *v {
					String::from("true")
				} else {
					String::from("false")
				}
			}
			Data::Number(v) => v.to_string(),
			Data::String(s) => s.clone(),
			Data::Memory { scope: _, name } => format!("<{}>", name),
			Data::Scope(_) => String::from("[scope]"),
			Data::None => String::from("[none]"),
		}
	}

	pub fn get_type(&self) -> DataType {
		match self {
			Data::Boolean(_) => DataType::Boolean,
			Data::Number(_) => DataType::Number,
			Data::String(_) => DataType::String,
			Data::Memory { .. } => DataType::Memory,
			Data::Scope(_) => DataType::Scope,
			Data::None => DataType::None,
		}
	}
}
