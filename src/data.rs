use std::{ cell::RefCell, hash::Hash };

use crate::{ pat_check, scope::ScopeRef };

pub enum DataType {
    Boolean,
    Number,
    String,
    Memory,
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
            DataType::Or(a, b) => a.to_string() + " | " + &b.to_string(),
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
    Memory {
        scope: ScopeRef,
        name: String,
    },
    Scope(ScopeRef),
    None,
}

impl Data {
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

impl PartialEq for Data {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::None, Self::None) => true,
            _ => false,
        }
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
                if *v { String::from("true") } else { String::from("false") }
            }
            Data::Number(v) => v.to_string(),
            Data::String(s) => s.clone(),
            Data::Memory { scope: _, name } => format!("<{}>", name),
            Data::Scope(scope) => RefCell::borrow(&scope).to_string(),
            Data::None => String::from("[none]"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StaticData(Data);

impl StaticData {
    pub fn from(data: Data) -> Self {
        match data {
            Data::Memory { .. } => panic!("Tried to cast type memory as static."),
            Data::Scope(_) => panic!("Tried to cast type scope as static."),
            _ => Self(data),
        }
    }

    pub fn inner(&self) -> &Data {
        &self.0
    }
}

impl Eq for StaticData {}

impl Hash for StaticData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.get_type().to_string().hash(state);
        match &self.0 {
            Data::Boolean(v) => v.hash(state),
            Data::Number(v) => v.to_string().hash(state),
            Data::String(v) => v.hash(state),
            _ => (),
        };
    }
}
