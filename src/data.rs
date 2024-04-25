use std::{cell::RefCell, hash::Hash, rc::Rc};

use crate::{pat_check, scope::ScopeRef};

pub enum DataType {
	Boolean,
	Number,
	String,
	Name,
	Scope,
	None,
	Or(Box<DataType>, Box<DataType>),
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
		} else if string == "name" {
			DataType::Name
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
			DataType::Name => String::from("name"),
			DataType::Scope => String::from("scope"),
			DataType::None => String::from("none"),
			DataType::Any => String::from("any"),
			DataType::Or(a, b) => a.to_string() + " | " + &b.to_string(),
		}
	}

	pub fn matches(&self, data: &Data) -> bool {
		match self {
			DataType::Boolean => pat_check!(Data::Boolean(_) = data),
			DataType::Number => pat_check!(Data::Number(_) = data),
			DataType::String => pat_check!(Data::String(_) = data),
			DataType::Name => pat_check!(Data::Name { .. } = data),
			DataType::Scope => pat_check!(Data::Scope(_) = data),
			DataType::None => pat_check!(Data::None = data),
			DataType::Or(a, b) => a.matches(data) || b.matches(data),
			DataType::Any => true,
		}
	}
}

#[derive(Clone, Debug)]
pub enum Data {
	Boolean(bool),
	Number(f64),
	String(String),
	Name { scope: ScopeRef, name: String },
	Scope(ScopeRef),
	None,
}

impl Data {
	pub fn get_type(&self) -> DataType {
		match self {
			Data::Boolean(_) => DataType::Boolean,
			Data::Number(_) => DataType::Number,
			Data::String(_) => DataType::String,
			Data::Name { .. } => DataType::Name,
			Data::Scope(_) => DataType::Scope,
			Data::None => DataType::None,
		}
	}
}

impl PartialEq for Data {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Boolean(l), Self::Boolean(r)) => l == r,
			(Self::Number(l), Self::Number(r)) => l == r,
			(Self::String(l), Self::String(r)) => l == r,
			(Self::None, Self::None) => true,
			(
				Self::Name { scope, name },
				Self::Name {
					scope: r_scope,
					name: r_name,
				},
			) => Rc::ptr_eq(scope, r_scope) && name == r_name,
			(Self::Scope(l), Self::Scope(r)) => Rc::ptr_eq(l, r),
			_ => false,
		}
	}
}

impl Eq for Data {}

impl Hash for Data {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		core::mem::discriminant(self).hash(state);
	}
}

impl Default for Data {
	fn default() -> Self {
		Data::None
	}
}

impl ToString for Data {
	fn to_string(&self) -> String {
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
			Data::Name { scope: _, name } => format!("<{}>", name),
			Data::Scope(scope) => RefCell::borrow(&scope).to_string(),
			Data::None => String::from("[none]"),
		}
	}
}
