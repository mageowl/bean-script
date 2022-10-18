export const operator = {
	PAREN: {
		START: "(",
		END: ")"
	},
	BRACE: {
		START: "{",
		END: "}"
	},
	ARROW: "->",
	COMMA: ","
};

export enum FTokenType {
	STRING = "lit.string",
	NUMBER = "lit.number",
	BOOLEAN = "lit.bool",
	MEMORY = "lit.memory",
	VALUE = "lit.value",
	NULL = "lit.null",
	OPERATOR = "operator",
	NEWLINE = "newline"
}
