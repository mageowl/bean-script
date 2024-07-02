use crate::{
    arg_check,
    data::{Data, DataType},
    error::{Error, ErrorSource},
    modules::{bean_std::collections::List, ModuleBuilder},
    scope::{function::Function, ScopeRef},
    util::make_ref,
};

pub(super) fn construct(module: &mut ModuleBuilder) {
    module
        .function("size", fn_size)
        .function("split", fn_split)
        .function("chars", fn_chars)
        .function("substr", fn_substr);
}

fn fn_size(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::String(s), "Expected string, but got {} instead. Use list.size to get the length of a list.", "std/string:size");
    Ok(Data::Number(s.len() as f64))
}

fn fn_split(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::String(s), "Expected string, but got {} instead.", "std/string:split");
    arg_check!(args.get(1).unwrap_or(&Data::None) => Data::String(d), "Expected delimiter string split, but got {} instead.", "split_string");
    let vec = s.split(d).map(|c| Data::String(String::from(c))).collect();
    Ok(Data::Scope(make_ref(List::new(vec, None))))
}

fn fn_chars(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::String(s), "Expected string, but got {} instead.", "std/string:chars");
    let vec = s.chars().map(|c| Data::String(String::from(c))).collect();
    Ok(Data::Scope(make_ref(List::new(vec, None))))
}

fn fn_substr(args: Vec<Data>, _y: Option<Function>, _s: ScopeRef) -> Result<Data, Error> {
    arg_check!(args.get(0).unwrap_or(&Data::None) => Data::String(s), "Expected string, but got {} instead.", "std/string:substr");
    arg_check!(args.get(1).unwrap_or(&Data::None) => Data::Number(start), "Expected number for start, but got {} instead.", "std/string:substr");
    let start = (*start as isize).rem_euclid(s.len() as isize) as usize;
    let end = if let Some(Data::Number(n)) = args.get(2) {
        Ok((*n as isize).rem_euclid(s.len() as isize) as usize)
    } else if args.get(2).is_some_and(|d| DataType::None.matches(d)) || args.get(2).is_none() {
        Ok(s.len())
    } else if let Some(d) = args.get(2) {
        Err(Error::new(
            &format!(
                "Expected number for end, but got {} instead.",
                d.get_type().to_string()
            ),
            ErrorSource::Builtin(String::from("std/string:substr")),
        ))
    } else {
        Err(Error::new("???", ErrorSource::Internal))
    }?;
    let substr = &s[start..end];

    Ok(Data::String(String::from(substr)))
}
