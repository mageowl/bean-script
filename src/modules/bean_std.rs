use super::BuiltinModule;

mod runtime;
mod collections;

pub(super) fn construct(module: &mut BuiltinModule) {
    module.submodule("runtime", runtime::construct);
}
