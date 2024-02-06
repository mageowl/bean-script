use std::{cell::RefCell, rc::Rc};

use crate::{
	arg_check,
	data::Data,
	scope::{Function, Scope},
};

use super::Module;

pub fn construct(module: &mut Module) {
	/* MEMORY */
	module
		.function("fn", fn_fn)
		.function("let", fn_let)
		.function("const", fn_const)
		.function("del", fn_del)
		.function("call", fn_call)
		.function("exists", fn_exists);

	/* SCOPE */
	module.function("p", fn_p);
	// .function("params", fn_params)
	// .function("yield", fn_yield)
	// .function("return", fn_return)
	// .function("pass", fn_pass)
	// .function("self", fn_self)
	// .function("super", fn_super)
	// .function("include", fn_include);
}

// MEMORY
fn fn_fn(args: Vec<Data>, yield_fn: Option<Function>, _s: Rc<RefCell<dyn Scope>>) -> Data {
	arg_check!(&args[0], Data::Memory { scope, name } => 
		"Expected memory as name of function, but instead got {}.");
	let yield_fn =
		yield_fn.unwrap_or_else(|| panic!("To define a function, add a yield block."));

	scope.borrow_mut().set_function(name, yield_fn);

	Data::None
}

fn fn_let(
	args: Vec<Data>,
	yield_fn: Option<Function>,
	o_scope: Rc<RefCell<dyn Scope>>,
) -> Data {
	arg_check!(&args[0], Data::Memory { scope, name } => 
		"Expected memory as name of variable, but instead got {}.");
	let value = yield_fn
		.unwrap_or_else(|| panic!("To define a variable, add a yield block."))
		.call(Vec::new(), None, Rc::clone(&o_scope));

	scope.borrow_mut().set_function(
		name,
		Function::Variable {
			value,
			constant: false,
			name: String::from(name),
		},
	);

	Data::None
}

fn fn_const(
	args: Vec<Data>,
	yield_fn: Option<Function>,
	o_scope: Rc<RefCell<dyn Scope>>,
) -> Data {
	arg_check!(&args[0], Data::Memory { scope, name } => 
		"Expected memory as name of constant, but instead got {}.");
	let value = yield_fn
		.unwrap_or_else(|| panic!("To define a constant, add a yield block."))
		.call(Vec::new(), None, Rc::clone(&o_scope));

	scope.borrow_mut().set_function(
		name,
		Function::Variable {
			value,
			constant: true,
			name: String::from(name),
		},
	);

	Data::None
}

fn fn_del(args: Vec<Data>, _y: Option<Function>, _s: Rc<RefCell<dyn Scope>>) -> Data {
	arg_check!(&args[0], Data::Memory { scope, name } => 
		"Expected memory for fn del, but instead got {}.");
	scope.borrow_mut().delete_function(name);

	Data::None
}

fn fn_call(
	args: Vec<Data>,
	yield_fn: Option<Function>,
	o_scope: Rc<RefCell<dyn Scope>>,
) -> Data {
	arg_check!(&args[0], Data::Memory { scope, name } =>
		"Expected memory for fn call, but instead got {}.");
	let function = scope.borrow().get_function(name).unwrap_or_else(|| {
		panic!("Unknown value or function for fn call '<{}>'.", &name)
	});

	function.call(args[1..].to_vec(), yield_fn, o_scope)
}

fn fn_exists(args: Vec<Data>, _y: Option<Function>, _s: Rc<RefCell<dyn Scope>>) -> Data {
	arg_check!(&args[0], Data::Memory { scope, name } =>
		"Expected memory for fn exists, but instead got {}.");
	Data::Boolean(scope.borrow().has_function(name))
}

// SCOPE
fn fn_p(args: Vec<Data>, _y: Option<Function>, scope: Rc<RefCell<dyn Scope>>) -> Data {
	todo!()
}
