#[macro_export]
macro_rules! arg_check {
	($arg: expr, $t: pat => $e: literal ) => {
		let $t = $arg else {
			panic!($e, $arg.type_str())
		};
	};
}
