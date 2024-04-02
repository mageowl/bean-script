use std::any::Any;
use std::collections::{ HashMap, VecDeque };
use std::mem;
use std::{ cell::RefCell, rc::Rc };

use crate::data::{ Data, StaticData };
use crate::scope::ScopeRef;
use crate::{ arg_check, as_mut_type, as_type };
use crate::scope::{ function::Function, Scope };

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
                Data::Number(
                    as_type!(RefCell::borrow(&list) => List, 
					"Tried to call fn size on a non-list scope.").items.len() as f64
                )
            })
        );
        make(
            "empty",
            Rc::new(|_a, _y, list: ScopeRef| {
                Data::Boolean(
                    as_type!(RefCell::borrow(&list) => List, "Tried to call fn empty on a non-list scope.").items.is_empty()
                )
            })
        );
        make(
            "has",
            Rc::new(|args, _y, list: ScopeRef| {
                Data::Boolean(
                    as_type!(RefCell::borrow(&list) => List, 
						"Tried to call fn has on a non-list scope.").items.contains(
                        &args[0]
                    )
                )
            })
        );
        make(
            "at",
            Rc::new(|args, _y, list: ScopeRef| {
                arg_check!(&args[0], Data::Number(i) => "Expected a number for fn at, but instead got {}.");
                as_type!(RefCell::borrow(&list) => List, 
						"Tried to call fn has on a non-list scope.").items[
                    *i as usize
                ].clone()
            })
        );
        make(
            "push",
            Rc::new(|args, _y, list: ScopeRef| {
                as_mut_type!(RefCell::borrow_mut(&list) => List,
						"Tried to call fn push on a non-list scope.").items.push_back(
                    args[0].clone()
                );
                Data::None
            })
        );
        make(
            "concat",
            Rc::new(|args, _y, list: ScopeRef| {
                arg_check!(&args[0], Data::Scope(list2) => "Expected scope for fn concat, but instead got {}.");
                as_mut_type!(RefCell::borrow_mut(&list) => List,
						"Tried to call fn concat on a non-list scope.").items.append(
                    &mut as_type!(RefCell::borrow(&list2) => List,
						"Tried to call fn concat with a non-list scope.").items.clone()
                );
                Data::None
            })
        );
        make(
            "pop",
            Rc::new(|_a, _y, list: ScopeRef| {
                as_mut_type!(RefCell::borrow_mut(&list) => List,
						"Tried to call fn pop on a non-list scope.").items
                    .pop_back()
                    .unwrap_or_default()
            })
        );
        make(
            "delete",
            Rc::new(|args, _y, list: ScopeRef| {
                arg_check!(&args[0], Data::Number(i) => "Expected number for fn delete, but instead got {}.");
                as_mut_type!(RefCell::borrow_mut(&list) => List,
						"Tried to call fn delete on a non-list scope.").items
                    .remove(*i as usize)
                    .unwrap_or_default()
            })
        );
        make(
            "insert",
            Rc::new(|args, _y, list: ScopeRef| {
                arg_check!(&args[0], Data::Number(i) => "Expected number for fn insert, but instead got {}.");
                as_mut_type!(RefCell::borrow_mut(&list) => List,
						"Tried to call fn delete on a non-list scope.").items.insert(
                    *i as usize,
                    args[1].clone()
                );
                Data::None
            })
        );

        make(
            "for",
            Rc::new(|args, body_fn, list: ScopeRef| {
                let body_fn = body_fn.expect("Expected body block for fn for.");
                arg_check!(&args[0], Data::Name { scope: item_scope_ref, name: item_name } =>
					"Expected name for fn for, but instead got {}.");

                let mut index_scope_ref: Option<&ScopeRef> = None;
                let mut index_name: Option<&String> = None;
                if args.len() > 1 {
                    arg_check!(&args[1], Data::Name { scope: i_scope_ref, name: i_name } =>
						"Expected name for fn for, but instead got {}.");
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

                    mapped.push(body_fn.call(Vec::new(), None, Rc::clone(&list)));
                }

                Data::Scope(Rc::new(RefCell::new(List::new(mapped, None))))
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
                        list.items.get(i).cloned().unwrap_or_default()
                    } else {
                        mem::replace(&mut list.items[i], args[0].clone())
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
}

#[derive(Debug)]
pub struct Map {
    parent: Option<ScopeRef>,
    fns: HashMap<String, Function>,
    pub hash: HashMap<StaticData, Data>,
}

impl Map {
    pub fn new(kv_pairs: Vec<Data>, parent: Option<ScopeRef>) -> Self {
        let mut map = Map {
            parent,
            fns: HashMap::new(),
            hash: kv_pairs
                .chunks(2)
                .map(|pair| (
                    StaticData::from(
                        pair.get(0).expect("Number of arguments must be even for fn map.").clone()
                    ),
                    pair.get(1).expect("Number of arguments must be even for fn map.").clone(),
                ))
                .collect::<HashMap<StaticData, Data>>(),
        };
        let mut make = |name: &str, closure| {
            map.fns.insert(String::from(name), Function::BuiltIn { callback: closure })
        };

        make(
            "size",
            Rc::new(|_a, _y, map: ScopeRef| {
                Data::Number(
                    as_type!(RefCell::borrow(&map) => Map, 
					"Tried to call fn size on a non-map scope.").hash.len() as f64
                )
            })
        );
        make(
            "empty",
            Rc::new(|_a, _y, map: ScopeRef| {
                Data::Boolean(
                    as_type!(RefCell::borrow(&map) => Map, 
					"Tried to call fn size on a non-map scope.").hash.is_empty()
                )
            })
        );
        make(
            "has",
            Rc::new(|args, _y, map: ScopeRef| {
                Data::Boolean(
                    as_type!(RefCell::borrow(&map) => Map, "Tried to call fn has on a non-map scope.").hash.contains_key(
                        &StaticData::from(args[0].clone())
                    )
                )
            })
        );
        make(
            "get",
            Rc::new(|args, _y, map: ScopeRef| {
                as_type!(RefCell::borrow(&map) => Map, "Tried to call fn has on a non-map scope.").hash
                    .get(&StaticData::from(args[0].clone()))
                    .cloned()
                    .unwrap_or_default()
            })
        );
        make(
            "set",
            Rc::new(|args, body_fn, map: ScopeRef| {
                as_mut_type!(map.borrow_mut() => Map, "Tried to call fn set on a non-map scope.").hash.insert(
                    StaticData::from(args[0].clone()),
                    body_fn
                        .expect("Expected body function for fn set.")
                        .call(Vec::new(), None, Rc::clone(&map))
                );
                Data::None
            })
        );
        make(
            "del",
            Rc::new(|args, _y, map: ScopeRef| {
                as_mut_type!(map.borrow_mut() => Map, "Tried to call fn set on a non-map scope.").hash.remove(
                    &StaticData::from(args[0].clone())
                );
                Data::None
            })
        );
        make(
            "for",
            Rc::new(|args, body_fn, map: ScopeRef| {
                let body_fn = body_fn.expect("Expected body block for fn for.");
                arg_check!(&args[0], Data::Name { scope: key_scope_ref, name: key_name } =>
					"Expected name for fn for, but instead got {}.");
                arg_check!(&args[1], Data::Name { scope: value_scope_ref, name: value_name } =>
					"Expected name for fn for, but instead got {}.");

                let mut index_scope_ref: Option<&ScopeRef> = None;
                let mut index_name: Option<&String> = None;
                if args.len() > 2 {
                    arg_check!(&args[2], Data::Name { scope: i_scope_ref, name: i_name } =>
						"Expected name for fn for, but instead got {}.");
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
                    key_scope_ref.borrow_mut().set_function(&key_name, Function::Constant { value: key.inner().clone() });
                    value_scope_ref.borrow_mut().set_function(&value_name, Function::Constant { value: value.clone() });
                    index_scope_ref.map(|s|
                        s.borrow_mut().set_function(&index_name.unwrap(), Function::Constant { value: Data::Number(i as f64) })
                    );

                    mapped.push(body_fn.call(Vec::new(), None, Rc::clone(&map)));
                }

                Data::Scope(Rc::new(RefCell::new(List::new(mapped, None))))
            })
        );

        map
    }
}

impl Scope for Map {
    fn has_function(&self, name: &str) -> bool {
        self.fns.contains_key(name) ||
            self.hash.contains_key(&StaticData::from(Data::String(String::from(name))))
    }

    fn get_function(&self, name: &str) -> Option<Function> {
        let builtin = self.fns.get(name);
        if builtin.is_some() {
            builtin.cloned()
        } else {
            let key = StaticData::from(Data::String(String::from(name)));
            Some(Function::BuiltIn {
                callback: Rc::new(move |args, _y, scope: ScopeRef| {
                    let mut binding = RefCell::borrow_mut(&scope);
                    let map = as_mut_type!(binding => Map, "Tried to index a non-list scope.");
                    if args.is_empty() {
                        map.hash.get(&key).cloned().unwrap_or_default()
                    } else {
                        mem::replace(&mut map.hash.get_mut(&key).unwrap(), args[0].clone())
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
                        &e.0.inner().to_string() +
                        ": " +
                        &e.1.to_string()
                )
        )
    }
}
