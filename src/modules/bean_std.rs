use super::ModuleBuilder;

mod collections;
mod runtime;
mod strings;

pub(super) fn construct(module: &mut ModuleBuilder) {
    module
        .submodule("runtime", runtime::construct)
        .submodule("strings", strings::construct);
}
