use std::{any::Any, cell::RefCell, collections::{ HashMap, VecDeque }, mem, rc::Rc};

use crate::{arg_check, as_mut_type, as_type, data::Data, error::{Error, ErrorSource}, scope::{function::Function, Scope, ScopeRef}};

#[derive(Debug)]
pub struct List {
    parent: Option<ScopeRef>,
    fns: HashMap<String, Function>,
    pub items: VecDeque<Data>,
}

impl List {
    pub fn new(list: Vec<Data>, parent: Option<ScopeRef>) -> Self {
        let mut list = List {
            parent,
            fns: HashMap::new(),
            items: VecDeque::from(list),
        };
        let mut make = |name: &str, closure| {
            list.fns.insert(String::from(name), Function::BuiltIn { callback: closure })
        };

        make(
            "size",
            Rc::new(|_a, _y, list: ScopeRef| {
                Ok(Data::Number(
                    as_type!(RefCell::borrow(&list) => List, 
					"Tried to call fn size on a non-list scope.").items.len() as f64
                ))
            })
        );
        make(
            "empty",
            Rc::new(|_a, _y, list: ScopeRef| {
                Ok(Data::Boolean(
                    as_type!(RefCell::borrow(&list) => List, "Tried to call fn empty on a non-list scope.").items.is_empty()
                ))
            })
        );
        make(
            "has",
            Rc::new(|args, _y, list: ScopeRef| {
                Ok(Data::Boolean(
                    as_type!(RefCell::borrow(&list) => List, 
						"Tried to call fn has on a non-list scope.").items.contains(
                        &args[0]
                    )
                ))
            })
        );
        make(
            "at",
            Rc::new(|args, _y, list: ScopeRef| {
                arg_check!(&args[0] => Data::Number(i), "Expected a number, but instead got {}.", "list:at");
                Ok(as_type!(RefCell::borrow(&list) => List, 
						"Tried to call fn has on a non-list scope.").items.get(*i as usize).cloned().unwrap_or(Data::None))
            })
        );
        make(
            "push",
            Rc::new(|args, _y, list: ScopeRef| {
                as_mut_type!(RefCell::borrow_mut(&list) => List,
						"Tried to call fn push on a non-list scope.").items.push_back(
                    args[0].clone()
                );
                Ok(Data::None)
            })
        );
        make(
            "concat",
            Rc::new(|args, _y, list: ScopeRef| {
                arg_check!(&args[0] => Data::Scope(list2), "Expected scope, but instead got {}.", "list:concat");
                as_mut_type!(RefCell::borrow_mut(&list) => List,
						"Tried to call fn concat on a non-list scope.").items.append(
                    &mut as_type!(RefCell::borrow(&list2) => List,
						"Tried to call fn concat with a non-list scope.").items.clone()
                );
                Ok(Data::None)
            })
        );
        make(
            "pop",
            Rc::new(|_a, _y, list: ScopeRef| {
                Ok(as_mut_type!(RefCell::borrow_mut(&list) => List,
						"Tried to call fn pop on a non-list scope.").items
                    .pop_back()
                    .unwrap_or_default())
            })
        );
        make(
            "delete",
            Rc::new(|args, _y, list: ScopeRef| {
                arg_check!(&args[0] => Data::Number(i), "Expected number, but instead got {}.", "list:delete");
                Ok(as_mut_type!(RefCell::borrow_mut(&list) => List,
						"Tried to call fn delete on a non-list scope.").items
                    .remove(*i as usize)
                    .unwrap_or_default())
            })
        );
        make(
            "insert",
            Rc::new(|args, _y, list: ScopeRef| {
                arg_check!(&args[0] => Data::Number(i), "Expected number, but instead got {}.", "list:insert");
                as_mut_type!(RefCell::borrow_mut(&list) => List,
						"Tried to call fn insert on a non-list scope.").items.insert(
                    *i as usize,
                    args[1].clone()
                );
                Ok(Data::None)
            })
        );
        make(
            "set",
            Rc::new(|mut args, _y, list: ScopeRef| {
                arg_check!(&args[0] => Data::Number(i), "Expected number, but instead got {}.", "list:insert");
                Ok(
                    mem::replace(
                        as_mut_type!(RefCell::borrow_mut(&list) => List,
                            "Tried to call fn set on a non-list scope.").items.get_mut(*i as usize)
                        .ok_or(Error::new("Index not inside list bounds.", ErrorSource::Builtin(String::from("list"))))?,
                        args.remove(1)
                    )
                )
            })
        );

        make(
            "for",
            Rc::new(|args, body_fn, list: ScopeRef| {
                let body_fn = body_fn.expect("Expected body block for fn for.");
                arg_check!(&args[0] => Data::Name { scope: item_scope_ref, name: item_name },
					"Expected name, but instead got {}.",
					"list:for");

                let mut index_scope_ref: Option<&ScopeRef> = None;
                let mut index_name: Option<&String> = None;
                if args.len() > 1 {
                    arg_check!(&args[1] => Data::Name { scope: i_scope_ref, name: i_name },
						"Expected name, but instead got {}.", "list:for");
                    index_scope_ref = Some(i_scope_ref);
                    index_name = Some(i_name);
                }

                let mut mapped: Vec<Data> = Vec::new();

                for (
                    i,
                    item,
                ) in &mut as_mut_type!(list.borrow_mut() => List,
						"Tried to call fn for on a non-list scope.").items
                    .iter()
                    .enumerate() {
                    item_scope_ref.borrow_mut().set_function(&item_name, Function::Constant {
                        value: item.clone(),
                    });
                    index_scope_ref.map(|s|
                        s.borrow_mut().set_function(&index_name.unwrap(), Function::Constant {
                            value: Data::Number(i as f64),
                        })
                    );

                    mapped.push(body_fn.call(Vec::new(), None, Rc::clone(&list))?);
                }

                Ok(Data::Scope(Rc::new(RefCell::new(List::new(mapped, None)))))
            })
        );

        list
    }
}

impl Scope for List {
    fn has_function(&self, name: &str) -> bool {
        if let Ok(i) = name.parse() { self.items.len() > i } else { self.fns.contains_key(name) }
    }

    fn get_function(&self, name: &str) -> Option<Function> {
        if let Ok(i) = name.parse::<usize>() {
            Some(Function::BuiltIn {
                callback: Rc::new(move |args, _y, scope: ScopeRef| {
                    let mut binding = RefCell::borrow_mut(&scope);
                    let list = as_mut_type!(binding => List, "Tried to index a non-list scope.");
                    if args.is_empty() {
                        Ok(list.items.get(i).cloned().unwrap_or_default())
                    } else {
                        Ok(mem::replace(&mut list.items[i], args[0].clone()))
                    }
                }),
            })
        } else {
            self.fns.get(name).cloned()
        }
    }

    fn set_function(&mut self, _n: &str, _f: Function) {}

    fn delete_function(&mut self, _n: &str) {}

    fn set_return_value(&mut self, _value: Data) {}
    fn get_function_list(&self) -> HashMap<String, Function> {
        HashMap::new()
    }

    fn parent(&self) -> Option<ScopeRef> {
        self.parent.as_ref().map(|x| Rc::clone(x))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn to_string(&self) -> String {
        format!(
            "[{}]",
            self.items
                .iter()
                .fold(
                    String::new(),
                    |s, e| (if s.is_empty() { String::new() } else { s + ", " }) + &e.to_string()
                )
        )
    }
	
	fn set_if_state(&mut self, _state: crate::scope::block_scope::IfState) {
	}
}

#[derive(Debug)]
pub struct Map {
    parent: Option<ScopeRef>,
    fns: HashMap<String, Function>,
    pub hash: HashMap<Data, Data>,
}

impl Map {
    pub fn new(kv_pairs: Vec<Data>, parent: Option<ScopeRef>) -> Self {
        let mut map = Map {
            parent,
            fns: HashMap::new(),
            hash: kv_pairs
                .chunks(2)
                .map(|pair| (
                    
                        pair.get(0).expect("Number of arguments must be even for fn map.").clone()
                    ,
                    pair.get(1).expect("Number of arguments must be even for fn map.").clone(),
                ))
                .collect::<HashMap<Data, Data>>(),
        };
        let mut make = |name: &str, closure| {
            map.fns.insert(String::from(name), Function::BuiltIn { callback: closure })
        };

        make(
            "size",
            Rc::new(|_a, _y, map: ScopeRef| {
                Ok(Data::Number(
                    as_type!(RefCell::borrow(&map) => Map, 
					"Tried to call fn size on a non-map scope.").hash.len() as f64
                ))
            })
        );
        make(
            "empty",
            Rc::new(|_a, _y, map: ScopeRef| {
                Ok(Data::Boolean(
                    as_type!(RefCell::borrow(&map) => Map, 
					"Tried to call fn size on a non-map scope.").hash.is_empty()
                ))
            })
        );
        make(
            "has",
            Rc::new(|args, _y, map: ScopeRef| {
                Ok(Data::Boolean(
                    as_type!(RefCell::borrow(&map) => Map, "Tried to call fn has on a non-map scope.").hash.contains_key(
                        &args[0]
                    )
                ))
            })
        );
        make(
            "get",
            Rc::new(|args, _y, map: ScopeRef| {
                Ok(as_type!(RefCell::borrow(&map) => Map, "Tried to call fn has on a non-map scope.").hash
                    .get(&args[0])
                    .cloned()
                    .unwrap_or_default())
            })
        );
        make(
            "set",
            Rc::new(|args, body_fn, map: ScopeRef| {
                as_mut_type!(map.borrow_mut() => Map, "Tried to call fn set on a non-map scope.").hash.insert(
                    args[0].clone(),
                    body_fn
                        .expect("Expected body function for fn set.")
                        .call(Vec::new(), None, Rc::clone(&map))?
                );
                Ok(Data::None)
            })
        );
        make(
            "del",
            Rc::new(|args, _y, map: ScopeRef| {
                as_mut_type!(map.borrow_mut() => Map, "Tried to call fn set on a non-map scope.").hash.remove(
                    &args[0].clone()
                );
                Ok(Data::None)
            })
        );
        make(
            "for",
            Rc::new(|args, body_fn, map: ScopeRef| {
                let body_fn = body_fn.expect("Expected body block for fn for.");
                arg_check!(&args[0] => Data::Name { scope: key_scope_ref, name: key_name },
					"Expected name for fn for, but instead got {}.", "map:for");
                arg_check!(&args[1] => Data::Name { scope: value_scope_ref, name: value_name },
					"Expected name for fn for, but instead got {}.", "map:for");

                let mut index_scope_ref: Option<&ScopeRef> = None;
                let mut index_name: Option<&String> = None;
                if args.len() > 2 {
                    arg_check!(&args[2] => Data::Name { scope: i_scope_ref, name: i_name },
						"Expected name for fn for, but instead got {}.", "map:for");
                    index_scope_ref = Some(i_scope_ref);
                    index_name = Some(i_name);
                }

                let mut mapped: Vec<Data> = Vec::new();

                for (
                    i,
                    (key, value),
                ) in &mut as_mut_type!(map.borrow_mut() => Map,
						"Tried to call fn for on a non-list scope.").hash
                    .iter()
                    .enumerate() {
                    key_scope_ref.borrow_mut().set_function(&key_name, Function::Constant { value: key.clone() });
                    value_scope_ref.borrow_mut().set_function(&value_name, Function::Constant { value: value.clone() });
                    index_scope_ref.map(|s|
                        s.borrow_mut().set_function(&index_name.unwrap(), Function::Constant { value: Data::Number(i as f64) })
                    );

                    mapped.push(body_fn.call(Vec::new(), None, Rc::clone(&map))?);
                }

                Ok(Data::Scope(Rc::new(RefCell::new(List::new(mapped, None)))))
            })
        );

        map
    }
}

impl Scope for Map {
    fn has_function(&self, name: &str) -> bool {
        self.fns.contains_key(name) ||
            self.hash.contains_key(&Data::String(String::from(name)))
    }

    fn get_function(&self, name: &str) -> Option<Function> {
        let builtin = self.fns.get(name);
        if builtin.is_some() {
            builtin.cloned()
        } else {
            let key = Data::String(String::from(name));
            Some(Function::BuiltIn {
                callback: Rc::new(move |args, _y, scope: ScopeRef| {
                    let mut binding = RefCell::borrow_mut(&scope);
                    let map = as_mut_type!(binding => Map, "Tried to index a non-list scope.");
                    if args.is_empty() {
                        Ok(map.hash.get(&key).cloned().unwrap_or_default())
                    } else {
                        Ok(mem::replace(&mut map.hash.get_mut(&key).unwrap(), args[0].clone()))
                    }
                }),
            })
        }
    }

    fn set_function(&mut self, _name: &str, _function: Function) {}
    fn delete_function(&mut self, _name: &str) {}

    fn set_return_value(&mut self, _value: Data) {}
    fn get_function_list(&self) -> HashMap<String, Function> {
        HashMap::new()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn parent(&self) -> Option<ScopeRef> {
        self.parent.as_ref().map(|s| Rc::clone(s))
    }

    fn to_string(&self) -> String {
        format!(
            "{{{}}}",
            self.hash
                .iter()
                .fold(
                    String::new(),
                    |s, e|
                        (if s.is_empty() { String::new() } else { s + ", " }) +
                        &e.0.to_string() +
                        ": " +
                        &e.1.to_string()
                )
        )
    }
	
	fn set_if_state(&mut self, _state: crate::scope::block_scope::IfState) {
	}
}
