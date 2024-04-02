use std::{ cell::RefCell, rc::Rc, thread, time::Duration };

use crate::{
    arg_check,
    as_mut_type,
    as_type,
    data::{ Data, DataType },
    modules::{ loader, CustomModule },
    scope::{ block_scope::{ BlockScope, IfState }, function::Function, Scope, ScopeRef },
};

use super::{ collections::{ List, Map }, super::BuiltinModule };

pub fn construct(module: &mut BuiltinModule) {
    /* NAME */
    module
        .function("fn", fn_fn)
        .function("let", fn_let)
        .function("const", fn_const)
        .function("del", fn_del)
        .function("call", fn_call)
        .function("exists", fn_exists)
        .function("export", fn_export)
        .function("use", fn_use);

    /* SCOPE */
    module
        .function("p", fn_p)
        .function("args", fn_args)
        .function("body", fn_body)
        .function("return", fn_return)
        .function("pass", fn_pass)
        .function("self", fn_self)
        .function("super", fn_super)
        .function("include", fn_include);

    /* INTERFACE */
    module.function("print", fn_print).function("error", fn_error).function("sleep", fn_sleep);
    if cfg!(debug_assertions) {
        module.function("__debug", fn_debug);
    }

    /* MATH */
    module
        .function("add", fn_add)
        .function("+", fn_add)
        .function("sub", fn_sub)
        .function("-", fn_sub)
        .function("mul", fn_mul)
        .function("*", fn_mul)
        .function("div", fn_div)
        .function("/", fn_div)
        .function("pow", fn_pow)
        .function("^", fn_pow)
        .function("rand", fn_rand)
        .function("abs", fn_abs)
        .function("sin", fn_sin)
        .function("cos", fn_cos)
        .function("tan", fn_tan)
        .function("atan", fn_atan)
        .function("sqrt", fn_sqrt)
        .function("round", fn_round)
        .function("floor", fn_floor)
        .function("ceil", fn_ceil);

    /* STRINGS */
    module.function("size", fn_size).function("split", fn_split);

    /* TYPES */
    module
        .function("str", fn_str)
        .function("num", fn_num)
        .function("name", fn_name)
        .function("type", fn_type);

    /* COLLECTIONS */
    module.function("list", fn_list).function("map", fn_map);

    /* LOGIC */
    module
        .function("eq", fn_eq)
        .function("lt", fn_lt)
        .function("gt", fn_gt)
        .function("not", fn_not)
        .function("and", fn_and)
        .function("or", fn_or);

    /* CONTROL BLOCKS */
    module
        .function("if", fn_if)
        .function("else_if", fn_else_if)
        .function("else", fn_else)
        .function("ifv", fn_ifv)
        .function("repeat", fn_repeat)
        .function("while", fn_while)
        .function("match", fn_match);
}

//
// NAME
//

fn fn_fn(args: Vec<Data>, body_fn: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Name { scope, name } => 
		"Expected name as name of function, but instead got {}.");
    let body_fn = body_fn.unwrap_or_else(|| panic!("To define a function, add a body block."));

    RefCell::borrow_mut(&scope).set_function(name, body_fn);

    Data::None
}

fn fn_let(args: Vec<Data>, body_fn: Option<Function>, o_scope: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Name { scope, name } => 
		"Expected name as name of variable, but instead got {}.");
    let value = body_fn
        .unwrap_or_else(|| panic!("To define a variable, add a body block."))
        .call_scope(Vec::new(), None, Rc::clone(&o_scope));

    RefCell::borrow_mut(scope).set_function(name, Function::Variable {
        value,
        constant: false,
        name: String::from(name),
    });

    Data::None
}

fn fn_const(args: Vec<Data>, body_fn: Option<Function>, o_scope: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Name { scope, name } => 
		"Expected name as name of constant, but instead got {}.");
    let value = body_fn
        .unwrap_or_else(|| panic!("To define a constant, add a body block."))
        .call_scope(Vec::new(), None, Rc::clone(&o_scope));

    RefCell::borrow_mut(scope).set_function(name, Function::Variable {
        value,
        constant: true,
        name: String::from(name),
    });

    Data::None
}

fn fn_del(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Name { scope, name } => 
		"Expected name for fn del, but instead got {}.");
    RefCell::borrow_mut(scope).delete_function(name);

    Data::None
}

fn fn_call(args: Vec<Data>, body_fn: Option<Function>, o_scope: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Name { scope, name } =>
		"Expected name for fn call, but instead got {}.");
    let function = scope
        .borrow()
        .get_function(name)
        .unwrap_or_else(|| { panic!("Unknown value or function for fn call '<{}>'.", &name) });

    function.call(args[1..].to_vec(), body_fn, o_scope)
}

fn fn_exists(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Name { scope, name } =>
		"Expected name for fn exists, but instead got {}.");
    Data::Boolean(scope.borrow().has_function(name))
}

fn fn_export(args: Vec<Data>, _y: Option<Function>, to_scope: ScopeRef) -> Data {
    let mut binding = RefCell::borrow_mut(&to_scope);
    let module = as_mut_type!(binding => CustomModule, "Tried to export from a non-module scope.");
    arg_check!(&args[0], Data::Name { scope, name } => "Expected name for fn export, but instead got {}.");

    let target = RefCell::borrow(scope)
        .get_function(name)
        .unwrap_or_else(|| panic!("Tried to export empty name {}.", name));

    if let Function::Variable { constant: false, .. } = target {
        panic!("Tried to export non-constant value {}.", name);
    }

    module.exported_functions.borrow_mut().insert(
        args
            .get(1)
            .map(|a| {
                match a {
                    Data::String(s) => s,
                    _ =>
                        panic!(
                            "Expected string for fn export, but instead got {}.",
                            a.get_type().to_string()
                        ),
                }
            })
            .unwrap_or(name)
            .clone(),
        target
    );
    Data::None
}

fn fn_use(args: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Data {
    let registry =
        as_type!(
        scope
            .borrow()
            .get_file_module()
            .expect(
                "Cannot import modules outside of a module. Are you using the interactive terminal?"
            )
            .borrow() => CustomModule, "Fail ??"
    ).registry.clone();
    arg_check!(&args[0], Data::String(mod_id) => "Expected string for fn use, but instead got {}.");
    let (path_str, target) = {
        let mut iter = mod_id.split(":");
        (iter.next().expect("Tried to import from blank path."), iter.next())
    };
    let path: Vec<&str> = path_str.split("/").collect();

	let name_str;
	let name_scope;
    if args.len() > 1 {
        match &args[1] {
            Data::Name { name, scope } => {
				name_str = name as &str;
				name_scope = Rc::clone(scope);
			}
            _ =>
                panic!(
                    "Expected name for fn use, but instead got {}.",
                    &args[1].get_type().to_string()
                ),
        }
    } else {
		name_scope = Rc::clone(&scope);
		name_str = target.unwrap_or(*path.last().unwrap());
    };

    let module = loader::get(registry, String::from(path_str)).expect("..."); // TODO: convert return value of fns to Result<Data, Error>

	if name_str == "*" {
		let mut scope = RefCell::borrow_mut(&scope);

		for (name, func) in RefCell::borrow(&module).get_function_list() {
			scope.set_function(&name, func.clone());
		}

		Data::None
	} else if name_str == "" {
		Data::Scope(module)
	} else {
		name_scope.borrow_mut().set_function(name_str, Function::Variable { value: Data::Scope(module), constant: true, name: String::new() });
		Data::None
	}
}

//
// SCOPE
//

fn fn_p(args: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Number(i) => "Expected integer for fn p, but instead got {}.");
    let arg_type = args
        .get(1)
        .map(|x| {
            match x {
                Data::String(v) => DataType::from_string(v),
                _ =>
                    panic!(
                        "Expected type string for fn p, but instead got {}.",
                        x.get_type().to_string()
                    ),
            }
        })
        .unwrap_or(DataType::Any);
    let index = *i as usize;
    let arguments = scope
        .borrow()
        .get_call_scope()
        .expect("Cannot call fn p outside a call scope.")
        .borrow()
        .args();
    let arg = arguments.get(index).unwrap_or(&Data::None);
    if !arg_type.matches(&arg) {
        panic!(
            "Expected argument of type {}, but instead got {}.",
            arg_type.to_string(),
            arg.to_string()
        )
    } else {
        arg.clone()
    }
}

fn fn_args(_a: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Data {
    Data::Scope(
        Rc::new(
            RefCell::new(
                List::new(
                    Vec::clone(
                        &scope
                            .borrow()
                            .get_call_scope()
                            .expect("Cannot call fn params outside a call scope.")
                            .borrow()
                            .args()
                    ),
                    None
                )
            )
        )
    )
}

fn fn_body(args: Vec<Data>, body_fn: Option<Function>, scope: ScopeRef) -> Data {
    let call_scope = &scope
        .borrow()
        .get_call_scope()
        .expect("Cannot call fn body outside a call scope.");
    let call_scope = RefCell::borrow(&call_scope);
    Option::as_ref(call_scope.body_fn().as_ref())
        .expect("Expected body function.")
        .call(args, body_fn, call_scope.from_scope())
}

fn fn_return(args: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Data {
    let value = args.get(0).cloned().unwrap_or(Data::None);
    RefCell::borrow_mut(&scope).set_return_value(value.clone());
    match RefCell::borrow_mut(&scope).as_mut().downcast_mut::<BlockScope>() {
        Some(block) => block.break_self(),
        None => (),
    }
    value
}

fn fn_pass(args: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Data {
    let value = args.get(0).cloned().unwrap_or(Data::None);
    RefCell::borrow_mut(&scope).set_return_value(value.clone());
    value
}

fn fn_self(_a: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Data {
    Data::Scope(Rc::clone(&scope))
}

fn fn_super(_a: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Data {
    RefCell::borrow(&scope)
        .parent()
        .map(|s| Data::Scope(Rc::clone(&s)))
        .unwrap_or(Data::None)
}

fn fn_include(args: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Scope(target) => "Expected scope for fn include, but instead got {}.");
    let mut scope = RefCell::borrow_mut(&scope);

    for (name, func) in RefCell::borrow(&target).get_function_list() {
        scope.set_function(&name, func.clone());
    }

    Data::None
}

//
// INTERFACE
//

fn fn_print(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    let mut string = Vec::new();
    for data in args {
        string.push(data.to_string());
    }
    println!("{}", string.join(" "));

    Data::None
}

fn fn_error(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::String(msg) => "Expected string for fn error, but instead got {}.");
    panic!("{}", msg)
}

fn fn_sleep(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Number(ms) => "Expected number for fn sleep, but instead got {}.");
    thread::sleep(Duration::from_millis(*ms as u64));

    Data::None
}

fn fn_debug(_a: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Data {
    dbg!(scope);
    Data::None
}

//
// MATH
//

fn fn_add(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    match args[0].get_type() {
        DataType::String => {
            let mut string = String::new();
            for data in args {
                arg_check!(data, Data::String(v) => "Expected argument to be string for fn add, but instead got {}.");
                string.push_str(&v);
            }

            Data::String(string)
        }
        DataType::Number => {
            let mut n: f64 = 0.0;
            for data in args {
                arg_check!(data, Data::Number(a) => "Expected argument to be number for fn add, but instead got {}.");
                n += a;
            }

            Data::Number(n)
        }
        _ =>
            panic!(
                "Expected arguments of type string or number for fn add, but got {}.",
                args[0].get_type().to_string()
            ),
    }
}
fn fn_sub(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Number(a) => "Expected argument of type number for fn sub, but got {}.");
    arg_check!(&args[1], Data::Number(b) => "Expected argument of type number for fn sub, but got {}.");
    Data::Number(a - b)
}
fn fn_mul(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[1], Data::Number(b) => "Expected argument of type number for fn mul, but got {}.");
    match &args[0] {
        Data::Number(a) => Data::Number(a * b),
        Data::String(s) => Data::String(s.repeat(*b as usize)),
        _ =>
            panic!(
                "Expected argument of type number or string for fn mul, but got {} instead.",
                args[0].get_type().to_string()
            ),
    }
}
fn fn_div(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Number(a) => "Expected argument of type number for fn div, but got {}.");
    arg_check!(&args[1], Data::Number(b) => "Expected argument of type number for fn div, but got {}.");
    Data::Number(a / b)
}

fn fn_rand(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    match args.len() {
        0 => Data::Number(rand::random()),
        1 => {
            arg_check!(&args[0], Data::Number(max) => "Expected number for fn rand, but got {} instead.");
            Data::Number((rand::random::<f64>() * max).floor())
        }
        2.. => {
            arg_check!(&args[0], Data::Number(min) => "Expected number for fn rand, but got {} instead.");
            arg_check!(&args[1], Data::Number(max) => "Expected number for fn rand, but got {} instead.");
            Data::Number((rand::random::<f64>() * (max - min)).floor() + min)
        }
    }
}
fn fn_abs(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Number(n) => "Expected number for fn abs, but got {} instead.");
    Data::Number(n.round())
}

fn fn_pow(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Number(a) => "Expected number for fn pow, but got {} instead.");
    arg_check!(&args[1], Data::Number(b) => "Expected number for fn pow, but got {} instead.");
    Data::Number(a.powf(*b))
}
fn fn_sqrt(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Number(n) => "Expected number for fn sqrt, but got {} instead.");
    Data::Number(n.sqrt())
}

fn fn_sin(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Number(n) => "Expected number for fn sin, but got {} instead.");
    Data::Number(n.sin())
}
fn fn_cos(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Number(n) => "Expected number for fn cos, but got {} instead.");
    Data::Number(n.cos())
}
fn fn_tan(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Number(n) => "Expected number for fn tan, but got {} instead.");
    Data::Number(n.tan())
}
fn fn_atan(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Number(n) => "Expected number for fn atan, but got {} instead.");
    Data::Number(n.atan())
}

fn fn_round(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Number(n) => "Expected number for fn round, but got {} instead.");
    Data::Number(n.round())
}
fn fn_floor(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Number(n) => "Expected number for fn floor, but got {} instead.");
    Data::Number(n.floor())
}
fn fn_ceil(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Number(n) => "Expected number for fn ceil, but got {} instead.");
    Data::Number(n.ceil())
}

//
// STRING
//

fn fn_size(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::String(s) => "Expected string for fn size, but got {} instead.");
    Data::Number(s.len() as f64)
}

fn fn_split(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::String(s) => "Expected string for fn split, but got {} instead.");
    arg_check!(&args[1], Data::String(d) => "Expected delimiter string for fn split, but got {} instead.");
    let vec = s
        .split(d)
        .map(|c| Data::String(String::from(c)))
        .collect();
    Data::Scope(Rc::new(RefCell::new(List::new(vec, None))))
}

//
// TYPES
//

fn fn_str(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    Data::String(args.get(0).unwrap_or(&Data::None).to_string())
}

fn fn_num(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    match &args[0] {
        Data::Boolean(v) => Data::Number(if *v { 1.0 } else { 0.0 }),
        Data::Number(v) => Data::Number(*v),
        Data::String(s) =>
            s
                .parse()
                .map(|v| Data::Number(v))
                .unwrap_or(Data::None),
        Data::Name { .. } => Data::None,
        Data::Scope(_) => Data::None,
        Data::None => Data::None,
    }
}
fn fn_name(args: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Data {
    arg_check!(&args[0], Data::String(name) => "Expected string for fn mem, but got {} instead.");
    Data::Name {
        scope,
        name: name.clone(),
    }
}

fn fn_type(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    Data::String(args.get(0).unwrap_or(&Data::None).get_type().to_string())
}

//
// COLLECTIONS
//

fn fn_list(args: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Data {
    Data::Scope(Rc::new(RefCell::new(List::new(args, Some(scope)))))
}

fn fn_map(args: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Data {
    Data::Scope(Rc::new(RefCell::new(Map::new(args, Some(scope)))))
}

//
// LOGIC
//

fn fn_eq(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    Data::Boolean(&args[0] == &args[1])
}

fn fn_gt(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Number(a) => "Expected number for fn gt, but instead got {}.");
    arg_check!(&args[1], Data::Number(b) => "Expected number for fn gt, but instead got {}.");
    Data::Boolean(a > b)
}

fn fn_lt(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Number(a) => "Expected number for fn lt, but instead got {}.");
    arg_check!(&args[1], Data::Number(b) => "Expected number for fn lt, but instead got {}.");
    Data::Boolean(a < b)
}

fn fn_not(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Boolean(v) => "Expected boolean for fn not, but instead got {}.");
    Data::Boolean(!v)
}

fn fn_and(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Boolean(a) => "Expected boolean for fn and, but instead got {}.");
    arg_check!(&args[1], Data::Boolean(b) => "Expected boolean for fn and, but instead got {}.");
    Data::Boolean(*a && *b)
}

fn fn_or(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Boolean(a) => "Expected boolean for fn or, but instead got {}.");
    arg_check!(&args[1], Data::Boolean(b) => "Expected boolean for fn or, but instead got {}.");
    Data::Boolean(*a || *b)
}

//
// CONDITIONS
//

fn fn_if(args: Vec<Data>, body_fn: Option<Function>, scope: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Boolean(v) => "Expected boolean for fn if, but instead got {}.");

    let state: IfState = if *v {
        body_fn
            .expect("Expected body block for if statement")
            .call_direct(Vec::new(), None, Rc::clone(&scope));
        IfState::Captured
    } else {
        IfState::Started
    };

    as_mut_type!(RefCell::borrow_mut(&scope) => BlockScope, 
		"Cannot use if conditionals on a non-block scope.").if_state =
        state;

    Data::None
}

fn fn_else_if(args: Vec<Data>, body_fn: Option<Function>, scope: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Boolean(v) => "Expected boolean for fn else_if, but instead got {}.");

    let mut binding = RefCell::borrow_mut(&scope);
    let block_scope =
        as_mut_type!(binding => BlockScope, 
		"Cannot use if conditionals on a non-block scope.");

    match block_scope.if_state {
        IfState::Started => {
            if *v {
                block_scope.if_state = IfState::Captured;
                drop(binding);
                body_fn
                    .unwrap_or_else(|| panic!("To define a variable, add a body block."))
                    .call_direct(Vec::new(), None, Rc::clone(&scope));
            } else {
                block_scope.if_state = IfState::Started;
            }
        }
        IfState::Captured => (),
        IfState::Finished => panic!("Tried to call else_if before calling if."),
    }

    Data::None
}

fn fn_else(_a: Vec<Data>, body_fn: Option<Function>, scope: ScopeRef) -> Data {
    let mut binding = RefCell::borrow_mut(&scope);
    let block_scope =
        as_mut_type!(binding => BlockScope, 
		"Cannot use if conditionals on a non-block scope.");

    match block_scope.if_state {
        IfState::Started => {
            block_scope.if_state = IfState::Finished;
            drop(binding);
            body_fn
                .unwrap_or_else(|| panic!("To define a variable, add a body block."))
                .call_direct(Vec::new(), None, Rc::clone(&scope));
        }
        IfState::Captured => {
            block_scope.if_state = IfState::Finished;
        }
        IfState::Finished => panic!("Tried to call else before calling if."),
    }

    Data::None
}

fn fn_ifv(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Boolean(v) => "Expected boolean for fn ifv, but instead got {}.");

    if *v {
        args[1].clone()
    } else {
        args[2].clone()
    }
}

fn fn_repeat(args: Vec<Data>, body_fn: Option<Function>, scope: ScopeRef) -> Data {
    arg_check!(&args[0], Data::Number(n) => "Expected integer for fn repeat, but instead got {}.");
    let body_fn = body_fn.unwrap_or_else(|| panic!("Expected body block for fn repeat."));

    for _ in 0..*n as usize {
        body_fn.call_direct(Vec::new(), None, Rc::clone(&scope));
    }

    Data::None
}

fn fn_while(_a: Vec<Data>, body_fn: Option<Function>, scope: ScopeRef) -> Data {
    let body_fn = body_fn.unwrap_or_else(|| panic!("Expected body block for fn repeat."));

    loop {
        let v = body_fn.call_direct(Vec::new(), None, Rc::clone(&scope));
        if Data::Boolean(false) == v {
            break;
        }
    }

    Data::None
}

#[derive(Debug)]
struct MatchScope {
    parent: ScopeRef,
    value: Data,
}

impl Scope for MatchScope {
    fn has_function(&self, name: &str) -> bool {
        if name == "case" || name == "default" {
            true
        } else {
            RefCell::borrow(&self.parent).has_function(name)
        }
    }

    fn get_function(&self, name: &str) -> Option<Function> {
        if name == "case" {
            let match_value = self.value.clone();
            Some(Function::BuiltIn {
                callback: Rc::new(move |args, body_fn, scope| {
                    if match_value == args[0] {
                        let mut scope_m = RefCell::borrow_mut(&scope);
                        scope_m.set_return_value(
                            body_fn
                                .expect("Expected body block for function case.")
                                .call(Vec::new(), None, Rc::clone(&scope))
                        );
                        as_mut_type!(scope_m => BlockScope, "Tried to call case in a non-block scope.").break_self();
                    }
                    Data::None
                }),
            })
        } else if name == "default" {
            Some(Function::BuiltIn {
                callback: Rc::new(move |_a, body_fn, scope| {
                    RefCell::borrow_mut(&scope).set_return_value(
                        body_fn
                            .expect("Expected body block for function default.")
                            .call(Vec::new(), None, Rc::clone(&scope))
                    );
                    Data::None
                }),
            })
        } else {
            RefCell::borrow(&self.parent).get_function(name)
        }
    }

    fn set_function(&mut self, name: &str, function: Function) {
        RefCell::borrow_mut(&self.parent).set_function(name, function)
    }

    fn delete_function(&mut self, name: &str) {
        RefCell::borrow_mut(&self.parent).delete_function(name)
    }

    fn parent(&self) -> Option<ScopeRef> {
        Some(Rc::clone(&self.parent))
    }

    fn set_return_value(&mut self, _value: Data) {}

    fn get_function_list(&self) -> std::collections::HashMap<String, Function> {
        RefCell::borrow(&self.parent).get_function_list()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn fn_match(args: Vec<Data>, body_fn: Option<Function>, scope: ScopeRef) -> Data {
    let match_scope = Rc::new(
        RefCell::new(MatchScope {
            parent: Rc::clone(&scope),
            value: args[0].clone(),
        })
    );

    body_fn
        .expect("Expected body for fn match")
        .call_direct(Vec::new(), None, Rc::clone(&match_scope) as ScopeRef)
}
