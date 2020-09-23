import { literal, operator } from "./enums.js";

export function parser(tokens) {
	let i = 0;
	let ast = {
		type: "Block",
		body: []
	};

	function next() {
		return tokens[i++];
	}

	function peek() {
		return tokens[i];
	}

	function prev() {
		return tokens[i - 2];
	}

	function parse(token) {
		if (token.type == literal.VALUE) {
			let node = {
				type: "FunctionCall",
				name: token.value,
				parameters: [],
				yield: null
			};

			if (peek().type == "operator" && peek().value == operator.PAREN.START) {
				next();

				while (
					peek() &&
					!(peek().type == "operator" && peek().value == operator.PAREN.END)
				) {
					let parameter = { type: "Block", body: [] };

					while (
						peek() &&
						!(
							peek().type == "operator" &&
							(peek().value == operator.COMMA ||
								peek().value == operator.PAREN.END)
						)
					) {
						let paramNode = parse(next());
						if (paramNode) parameter.body.push(paramNode);
					}

					node.parameters.push(parameter);
				}

				next();
			}

			if (
				peek() &&
				peek().type == "operator" &&
				peek().value == operator.ARROW
			) {
				next();
				node.yield = parse(next());
			}

			return node;
		} else if (
			token.type == literal.STRING &&
			!(peek().type == "operator" && peek().value == operator.MATH.PLUS)
		) {
			return { type: "StringLiteral", value: token.value };
		} else if (token.type == literal.MEMORY) {
			return { type: "MemoryLiteral", value: token.value };
		} else if (
			token.type == literal.NUMBER &&
			peek().type != "operator" &&
			Object.values(operator.MATH).includes(peek().value)
		) {
			return { type: "NumberLiteral", value: token.value };
		} else if (token.type == "operator") {
			switch (token.value) {
				case operator.BRACE.START:
					let body = [];
					while (
						peek() &&
						!(peek().type == "operator" && peek().value == operator.BRACE.END)
					) {
						body.push(parse(next()));
					}
					next();

					return { type: "Block", body };

				case operator.MATH.PLUS:
					let operands = [parse(prev()), parse(next())];

					return {
						type: "PlusOperator",
						operands
					};

				default:
					break;
			}
		} else if (token.type == "newline") {
			return { type: "NewLine" };
		}
	}

	do {
		ast.body.push(parse(next()));
	} while (i < tokens.length);

	return ast;
}
