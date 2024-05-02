use std::{ cell::RefCell, rc::Rc, thread, time::Duration };

use crate::{
    arg_check, as_mut_type, as_type, data::{ Data, DataType }, error::{Error, ErrorSource}, modules::{ loader, CustomModule, ModuleBuilder }, scope::{ block_scope::{ BlockScope, IfState }, function::Function, Scope, ScopeRef }
};

use super::collections::{ List, Map };

pub fn construct(module: &mut ModuleBuilder) {
    /* NAME */
    module
        .function("fn", fn_fn)
        .function("let", fn_let)
        .function("const", fn_const)
        .function("del", fn_del)
        .function("call", fn_call)
        .function("exists", fn_exists);
	if module.features.import {
		module.function("export", fn_export).function("use", fn_use);
	}

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
    if module.features.lang_debug {
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

fn fn_fn(args: Vec<Data>, body_fn: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Name { scope, name },
		"Expected name as name of function, but instead got {}.", "function");
    let body_fn = body_fn.unwrap_or_else(|| panic!("To define a function, add a body block."));

    RefCell::borrow_mut(&scope).set_function(name, body_fn);

    Ok(Data::None)
}

fn fn_let(args: Vec<Data>, body_fn: Option<Function>, o_scope: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Name { scope, name },
		"Expected name as name of variable, but instead got {}.", "let");
    let value = body_fn
        .unwrap_or_else(|| panic!("To define a variable, add a body block."))
        .call_scope(Vec::new(), None, Rc::clone(&o_scope))?;

    RefCell::borrow_mut(scope).set_function(name, Function::Variable {
        value,
        scope_ref: Rc::clone(scope),
        name: String::from(name),
    });

    Ok(Data::None)
}

fn fn_const(args: Vec<Data>, body_fn: Option<Function>, o_scope: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Name { scope, name },
		"Expected name as name of constant, but instead got {}.", "constant");
    let value = body_fn
        .unwrap_or_else(|| panic!("To define a constant, add a body block."))
        .call_scope(Vec::new(), None, Rc::clone(&o_scope))?;

    RefCell::borrow_mut(scope).set_function(name, Function::Constant { value });

    Ok(Data::None)
}

fn fn_del(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Name { scope, name },
		"Expected name to delete, but instead got {}.", "delete");
    RefCell::borrow_mut(scope).delete_function(name);

    Ok(Data::None)
}

fn fn_call(args: Vec<Data>, body_fn: Option<Function>, o_scope: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Name { scope, name },
		"Expected name to call, but instead got {}.", "call");
    let function = scope
        .borrow()
        .get_function(name);
	if let None = function {
		return Err(Error::new(&format!("Unknown value or function {}", name), ErrorSource::Builtin(String::from("call"))));
	}

    function.unwrap().call(args[1..].to_vec(), body_fn, o_scope)
}

fn fn_exists(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Name { scope, name },
		"Expected name, but instead got {}.", "exists");
    Ok(Data::Boolean(scope.borrow().has_function(name)))
}

fn fn_export(args: Vec<Data>, _y: Option<Function>, to_scope: ScopeRef) -> Result<Data, Error> {
    let mut binding = RefCell::borrow_mut(&to_scope);
    let module = as_mut_type!(binding => CustomModule, "Tried to export from a non-module scope.");
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Name { scope, name }, "Expected name, but instead got {}.", "export");

    let target = scope.try_borrow().map_or_else(
        |_|
            module
                .get_function(name)
                .unwrap_or_else(|| panic!("Tried to export empty name {}.", name)),
        |s| s.get_function(name).unwrap_or_else(|| panic!("Tried to export empty name {}.", name))
    );

    if let Function::Variable { .. } = target {
        return Err(Error::new("Cannot export a variable.", ErrorSource::Builtin(String::from("export"))));
    }

    module.exported_functions.borrow_mut().insert(
        args
            .get(1)
            .map(|a| {
                match a {
                    Data::String(s) => s,
                    _ =>
                        panic!(
                            "Expected string export, but instead got {}.",
                            a.get_type().to_string()
                        ),
                }
            })
            .unwrap_or(name)
            .clone(),
        target
    );
    Ok(Data::None)
}

fn fn_use(args: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Result<Data, Error> {
    let binding = scope
        .borrow()
        .get_file_module()
		.ok_or(Error::new(
			"Cannot import modules outside of a module. Are you using the interactive terminal?",
			ErrorSource::Builtin(String::from("use"))
		))?;

    let borrowed = binding.borrow();
    let file_module = as_type!(borrowed => CustomModule, "Fail ??");

    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::String(mod_id), "Expected string, but instead got {}.", "use");
    let (path_str, target) = {
        let mut iter = mod_id.split(":");
        (iter.next().expect("Tried to import from blank path."), iter.next())
    };
    let path: Vec<&str> = path_str.split("/").collect();

    let name_str;
    let name_scope;
    if args.len() > 1 {
        match args.get(1).unwrap_or(&Data::None) {
            Data::Name { name, scope } => {
                name_str = name as &str;
                name_scope = Rc::clone(scope);
            }
            _ =>
                panic!(
                    "Expected name use, but instead got {}.",
                    args.get(1).unwrap_or(&Data::None).get_type().to_string()
                ),
        }
    } else {
        name_scope = Rc::clone(&scope);
        name_str = target.unwrap_or(*path.last().unwrap());
    }

    let module = loader::get(file_module, String::from(path_str))?;

    if target == Some("*") {
        let mut scope = RefCell::borrow_mut(&scope);

        for (name, func) in RefCell::borrow(&module).get_function_list() {
            scope.set_function(&name, func.clone());
        }

        Ok(Data::None)
    } else if target == Some("") {
        Ok(Data::Scope(module))
    } else if let Some(t) = target {
        name_scope
            .borrow_mut()
            .set_function(name_str, 
				module
					.borrow()
					.get_function(t)
					.unwrap_or_else(|| panic!("Tried to import non-existent function {} from module {}.", t, path_str))
			);
		Ok(Data::None)
    } else {
		name_scope
            .borrow_mut()
            .set_function(name_str, Function::Constant { value: Data::Scope(module) });
        Ok(Data::None)
	}
}

//
// SCOPE
//

fn fn_p(args: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Number(i), "Expected integer, but instead got {}.", "get_argument");
    let arg_type = args
        .get(1)
        .map(|x| {
            match x {
                Data::String(v) => DataType::from_string(v),
                _ =>
                    panic!(
                        "Expected string, but instead got {}.",
                        x.get_type().to_string()
                    ),
            }
        })
        .unwrap_or(Ok(DataType::Any))?;
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
            "Expected {}, but instead got {}.",
            arg_type.to_string(),
            arg.to_string()
        )
    } else {
        Ok(arg.clone())
    }
}

fn fn_args(_a: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Result<Data, Error> {
    Ok(Data::Scope(
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
    ))
}

fn fn_body(args: Vec<Data>, body_fn: Option<Function>, scope: ScopeRef) -> Result<Data, Error> {
    let call_scope = &scope
        .borrow()
        .get_call_scope()
        .expect("Cannot call fn body outside a call scope.");
    let call_scope = RefCell::borrow(&call_scope);
    Option::as_ref(call_scope.body_fn().as_ref())
        .expect("Expected body function.")
        .call(args, body_fn, call_scope.from_scope())
}

fn fn_return(args: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Result<Data, Error> {
    let value = args.get(0).cloned().unwrap_or(Data::None);
    RefCell::borrow_mut(&scope).set_return_value(value.clone());
    match RefCell::borrow_mut(&scope).as_mut().downcast_mut::<BlockScope>() {
        Some(block) => block.break_self(),
        None => (),
    }
    Ok(value)
}

fn fn_pass(args: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Result<Data, Error> {
    let value = args.get(0).cloned().unwrap_or(Data::None);
    RefCell::borrow_mut(&scope).set_return_value(value.clone());
    Ok(value)
}

fn fn_self(_a: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Result<Data, Error> {
    Ok(Data::Scope(Rc::clone(&scope)))
}

fn fn_super(_a: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Result<Data, Error> {
    Ok(RefCell::borrow(&scope)
        .parent()
        .map(|s| Data::Scope(Rc::clone(&s)))
        .unwrap_or(Data::None))
}

fn fn_include(args: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Scope(target), "Expected scope, but instead got {}.", "include");
    let mut scope = RefCell::borrow_mut(&scope);

    for (name, func) in RefCell::borrow(&target).get_function_list() {
        scope.set_function(&name, func.clone());
    }

    Ok(Data::None)
}

//
// INTERFACE
//

fn fn_print(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    let mut string = Vec::new();
    for data in args {
        string.push(data.to_string());
    }
    println!("{}", string.join(" "));

    Ok(Data::None)
}

fn fn_error(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::String(msg), "Expected string, but instead got {}.", "error");
    Err(Error::new(&msg, ErrorSource::Internal))
}

fn fn_sleep(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Number(ms), "Expected number of milliseconds, but instead got {}.", "sleep");
    thread::sleep(Duration::from_millis(*ms as u64));

    Ok(Data::None)
}

fn fn_debug(_a: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Result<Data, Error> {
    dbg!(scope);
    Ok(Data::None)
}

//
// MATH
//

fn fn_add(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    match args[0].get_type() {
        DataType::String => {
            let mut string = String::new();
            for data in args {
                arg_check!(data => Data::String(v), "Expected string, but instead got {}.", "add");
                string.push_str(&v);
            }

            Ok(Data::String(string))
        }
        DataType::Number => {
            let mut n: f64 = 0.0;
            for data in args {
                arg_check!(data => Data::Number(a), "Expected number, but instead got {}.", "add");
                n += a;
            }

            Ok(Data::Number(n))
        }
        _ =>
            Err(Error::new(&format!(
                "Expected arguments of type string or number, but got {}.",
                args[0].get_type().to_string()
            ), ErrorSource::Builtin(String::from("add")))),
    }
}
fn fn_sub(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Number(a), "Expected number, but got {}.", "subtract");
    arg_check!(args.get(1).unwrap_or(&Data::None) => Data::Number(b), "Expected number, but got {}.", "subtract");
    Ok(Data::Number(a - b))
}
fn fn_mul(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(1).unwrap_or(&Data::None) => Data::Number(b), "Expected number, but got {}.", "multiply");
    match args.get(0).unwrap_or(&Data::None) {
        Data::Number(a) => Ok(Data::Number(a * b)),
        Data::String(s) => Ok(Data::String(s.repeat(*b as usize))),
        _ =>
            Err(Error::new(&format!(
                "Expected number or string, but got {} instead.",
                args[0].get_type().to_string()
            ), ErrorSource::Builtin(String::from("multiply")))),
    }
}
fn fn_div(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Number(a), "Expected number, but got {}.", "divide");
    arg_check!(args.get(1).unwrap_or(&Data::None) => Data::Number(b), "Expected number, but got {}.", "divide");
    Ok(Data::Number(a / b))
}

fn fn_rand(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    match args.len() {
        0 => Ok(Data::Number(rand::random())),
        1 => {
            arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Number(max), "Expected number, but got {} instead.", "random");
            Ok(Data::Number((rand::random::<f64>() * max).floor()))
        }
        2.. => {
            arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Number(min), "Expected number, but got {} instead.", "random");
            arg_check!(args.get(1).unwrap_or(&Data::None) => Data::Number(max), "Expected number, but got {} instead.", "random");
            Ok(Data::Number((rand::random::<f64>() * (max - min)).floor() + min))
        }
    }
}
fn fn_abs(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Number(n), "Expected number, but got {} instead.", "absolute_value");
    Ok(Data::Number(n.round()))
}

fn fn_pow(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Number(a), "Expected number, but got {} instead.", "power");
    arg_check!(args.get(1).unwrap_or(&Data::None) => Data::Number(b), "Expected number, but got {} instead.", "power");
    Ok(Data::Number(a.powf(*b)))
}
fn fn_sqrt(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Number(n), "Expected number, but got {} instead.", "square_root");
    Ok(Data::Number(n.sqrt()))
}

fn fn_sin(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Number(n), "Expected number, but got {} instead.", "sine");
    Ok(Data::Number(n.sin()))
}
fn fn_cos(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Number(n), "Expected number, but got {} instead.", "cosine");
    Ok(Data::Number(n.cos()))
}
fn fn_tan(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Number(n), "Expected number, but got {} instead.", "tangent");
    Ok(Data::Number(n.tan()))
}
fn fn_atan(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Number(n), "Expected number, but got {} instead.", "arctangent");
    Ok(Data::Number(n.atan()))
}

fn fn_round(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Number(n), "Expected number, but got {} instead.", "round");
    Ok(Data::Number(n.round()))
}
fn fn_floor(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Number(n), "Expected number, but got {} instead.", "floor");
    Ok(Data::Number(n.floor()))
}
fn fn_ceil(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Number(n), "Expected number, but got {} instead.", "ceiling");
    Ok(Data::Number(n.ceil()))
}

//
// STRING
//

fn fn_size(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::String(s), "Expected string, but got {} instead. Use list.size to get the length of a list.", "string_size");
    Ok(Data::Number(s.len() as f64))
}

fn fn_split(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::String(s), "Expected string, but got {} instead.", "split_string");
    arg_check!(args.get(1).unwrap_or(&Data::None) => Data::String(d), "Expected delimiter string split, but got {} instead.", "split_string");
    let vec = s
        .split(d)
        .map(|c| Data::String(String::from(c)))
        .collect();
    Ok(Data::Scope(Rc::new(RefCell::new(List::new(vec, None)))))
}

//
// TYPES
//

fn fn_str(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    Ok(Data::String(args.get(0).unwrap_or(&Data::None).to_string()))
}

fn fn_num(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    match args.get(0).unwrap_or(&Data::None) {
        Data::Boolean(v) => Ok(Data::Number(if *v { 1.0 } else { 0.0 })),
        Data::Number(v) => Ok(Data::Number(*v)),
        Data::String(s) =>
            Ok(s
                .parse()
                .map(|v| Data::Number(v))
                .unwrap_or(Data::None)),
        Data::Name { .. } => Ok(Data::None),
        Data::Scope(_) => Ok(Data::None),
        Data::None => Ok(Data::None),
    }
}
fn fn_name(args: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::String(name), "Expected string, but got {} instead.", "to_name");
    Ok(Data::Name {
        scope,
        name: name.clone(),
    })
}

fn fn_type(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    Ok(Data::String(args.get(0).unwrap_or(&Data::None).get_type().to_string()))
}

//
// COLLECTIONS
//

fn fn_list(args: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Result<Data, Error> {
    Ok(Data::Scope(Rc::new(RefCell::new(List::new(args, Some(scope))))))
}

fn fn_map(args: Vec<Data>, _y: Option<Function>, scope: ScopeRef) -> Result<Data, Error> {
    Ok(Data::Scope(Rc::new(RefCell::new(Map::new(args, Some(scope))))))
}

//
// LOGIC
//

fn fn_eq(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    Ok(Data::Boolean(args.get(0).unwrap_or(&Data::None) == args.get(1).unwrap_or(&Data::None)))
}

fn fn_gt(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Number(a), "Expected number, but instead got {}.", "greater_than");
    arg_check!(args.get(1).unwrap_or(&Data::None) => Data::Number(b), "Expected number, but instead got {}.", "greater_than");
    Ok(Data::Boolean(a > b))
}

fn fn_lt(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Number(a), "Expected number, but instead got {}.", "less_than");
    arg_check!(args.get(1).unwrap_or(&Data::None) => Data::Number(b), "Expected number, but instead got {}.", "less_than");
    Ok(Data::Boolean(a < b))
}

fn fn_not(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Boolean(v), "Expected boolean, but instead got {}.", "not");
    Ok(Data::Boolean(!v))
}

fn fn_and(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Boolean(a), "Expected boolean, but instead got {}.", "and");
    arg_check!(args.get(1).unwrap_or(&Data::None) => Data::Boolean(b), "Expected boolean, but instead got {}.", "and");
    Ok(Data::Boolean(*a && *b))
}

fn fn_or(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Boolean(a), "Expected boolean, but instead got {}.", "or");
    arg_check!(args.get(1).unwrap_or(&Data::None) => Data::Boolean(b), "Expected boolean, but instead got {}.", "or");
    Ok(Data::Boolean(*a || *b))
}

//
// CONDITIONS
//

fn fn_if(args: Vec<Data>, body_fn: Option<Function>, scope: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Boolean(v), "Expected boolean, but instead got {}.", "if");

    let state: IfState = if *v {
        body_fn
            .expect("Expected body block for if statement")
            .call_direct(Vec::new(), None, Rc::clone(&scope))?;
        IfState::Captured
    } else {
        IfState::Started
    };

    scope.borrow_mut().set_if_state(state);

    Ok(Data::None)
}

fn fn_else_if(args: Vec<Data>, body_fn: Option<Function>, scope: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Boolean(v), "Expected boolean, but instead got {}.", "else_if");

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
                    .call_direct(Vec::new(), None, Rc::clone(&scope))?;
            } else {
                block_scope.if_state = IfState::Started;
            }
        }
        IfState::Captured => (),
        IfState::Finished => panic!("Tried to call else_if before calling if."),
    }

    Ok(Data::None)
}

fn fn_else(_a: Vec<Data>, body_fn: Option<Function>, scope: ScopeRef) -> Result<Data, Error> {
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
                .call_direct(Vec::new(), None, Rc::clone(&scope))?;
        }
        IfState::Captured => {
            block_scope.if_state = IfState::Finished;
        }
        IfState::Finished => panic!("Tried to call else before calling if."),
    }

    Ok(Data::None)
}

fn fn_ifv(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Boolean(v), "Expected boolean, but instead got {}.", "ifv");

    if *v {
        Ok(args[1].clone())
    } else {
        Ok(args[2].clone())
    }
}

fn fn_repeat(args: Vec<Data>, body_fn: Option<Function>, scope: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::Number(n), "Expected integer, but instead got {}.", "repeat");
    let body_fn = body_fn.unwrap_or_else(|| panic!("Expected body block for fn repeat."));

    for _ in 0..*n as usize {
        body_fn.call_direct(Vec::new(), None, Rc::clone(&scope))?;
    }

    Ok(Data::None)
}

fn fn_while(_a: Vec<Data>, body_fn: Option<Function>, scope: ScopeRef) -> Result<Data, Error> {
    let body_fn = body_fn.unwrap_or_else(|| panic!("Expected body block for fn repeat."));

    loop {
        let v = body_fn.call_direct(Vec::new(), None, Rc::clone(&scope))?;
        if Data::Boolean(false) == v {
            break;
        }
    }

    Ok(Data::None)
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
                                .call(Vec::new(), None, Rc::clone(&scope))?
                        );
                        as_mut_type!(scope_m => BlockScope, "Tried to call case in a non-block scope.").break_self();
                    }
                    Ok(Data::None)
                }),
            })
        } else if name == "default" {
            Some(Function::BuiltIn {
                callback: Rc::new(move |_a, body_fn, scope| {
                    RefCell::borrow_mut(&scope).set_return_value(
                        body_fn
                            .expect("Expected body block for function default.")
                            .call(Vec::new(), None, Rc::clone(&scope))?
                    );
                    Ok(Data::None)
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
	
	fn set_if_state(&mut self, _state: IfState) {}
}

fn fn_match(args: Vec<Data>, body_fn: Option<Function>, scope: ScopeRef) -> Result<Data, Error> {
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
