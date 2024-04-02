use std::{cell::RefCell, rc::Rc};

#[macro_export]
macro_rules! arg_check {
	($arg:expr => $t:pat, $e:literal, $n:literal) => {
		let $t = $arg else {
			return Err(Error::new(
				&format!($e, $arg.get_type().to_string()),
				ErrorSource::Builtin(String::from($n)),
			));
		};
	};
}

#[macro_export]
macro_rules! pat_check {
	($pat:pat = $value:expr) => {
		if let $pat = $value {
			true
		} else {
			false
		}
	};
}

#[macro_export]
macro_rules! as_type {
	($expr:expr => $t:ty, $err:literal) => {
		match $expr.as_any().downcast_ref::<$t>() {
			Some(obj) => obj,
			None => panic!($err),
		}
	};
}

#[macro_export]
macro_rules! as_mut_type {
	($expr:expr => $t:ty, $err:literal) => {
		match $expr.as_mut().downcast_mut::<$t>() {
			Some(obj) => obj,
			None => panic!($err),
		}
	};
}

pub fn make_ref<T>(scope: T) -> MutRc<T> {
	Rc::new(RefCell::new(scope))
}

pub type MutRc<T> = Rc<RefCell<T>>;
