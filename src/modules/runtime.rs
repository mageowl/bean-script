use std::{cell::RefCell, rc::Rc};

use crate::{
	arg_check,
	data::Data,
	scope::{Function, Scope},
};

use super::Module;

pub fn construct(module: &mut Module) {
	module
		.function("fn", fn_fn)
		.function("let", fn_let)
		.function("const", fn_const);
}

fn fn_fn(args: Vec<Data>, yield_fn: Option<Function>, _scope: Rc<RefCell<Scope>>) -> Data {
	arg_check!(&args[0], Data::Memory { scope, name } => 
		"Expected memory as name of function, but instead got {}.");
	let yield_fn =
		yield_fn.unwrap_or_else(|| panic!("To define a function, add a yield block."));

	scope.borrow_mut().set_function(name, yield_fn);

	Data::None
}

fn fn_let(args: Vec<Data>, yield_fn: Option<Function>, o_scope: Rc<RefCell<Scope>>) -> Data {
	arg_check!(&args[0], Data::Memory { scope, name } => 
		"Expected memory as name of variable, but instead got {}.");
	let value = yield_fn
		.unwrap_or_else(|| panic!("To define a variable, add a yield block."))
		.call(Vec::new(), None, Rc::clone(&o_scope));

	scope.borrow_mut().set_function(name, Function::Variable { value, constant: false, name: String::from(name) });

	Data::None
}

fn fn_const(args: Vec<Data>, yield_fn: Option<Function>, o_scope: Rc<RefCell<Scope>>) -> Data {
	arg_check!(&args[0], Data::Memory { scope, name } => 
		"Expected memory as name of constant, but instead got {}.");
	let value = yield_fn
		.unwrap_or_else(|| panic!("To define a constant, add a yield block."))
		.call(Vec::new(), None, Rc::clone(&o_scope));

	scope.borrow_mut().set_function(name, Function::Variable { value, constant: true, name: String::from(name) });

	Data::None
}
