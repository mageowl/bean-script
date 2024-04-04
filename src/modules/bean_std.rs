use super::ModuleBuilder;

mod collections;
mod runtime;

pub(super) fn construct(module: &mut ModuleBuilder) {
	module.submodule("runtime", runtime::construct);
}
