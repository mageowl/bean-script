pub enum DataType {
	Boolean(bool),
	Number(isize),
	String(String),
	Memory {
		// TODO: add scope ref
		name: String,
	},
	Scope(/* TODO: add scope ref */),
	None,
}
