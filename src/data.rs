pub enum DataType {
	String(String),
	Boolean(bool),
	Number(isize),
	Memory {
		// TODO: add scope ref
		name: String,
	},
	Scope( /* TODO: add scope ref */ ),
	None
}