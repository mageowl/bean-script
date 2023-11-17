import { FTokenType, operator } from "./enums.js";
import {
	FNodeBlock,
	FNodeAny,
	FNodeFunctionCall,
	FToken,
	FNodeFunctionAccess
} from "./interfaces.js";

export function parser(tokens: FToken[]) {
	let i = 0;
	let prev: FNodeAny = null;
	let ast: FNodeBlock = {
		type: "Program",
		body: []
	};

	function next() {
		return tokens[i++];
	}

	function peek() {
		return tokens[i];
	}

	function parse(token): FNodeAny {
		prev = ((): FNodeAny => {
			if (token.type === FTokenType.VALUE) {
				let node: FNodeFunctionCall = {
					type: "FunctionCall",
					name: token.value,
					parameters: [],
					yieldFunction: null
				};

				if (
					peek() &&
					peek().type === "operator" &&
					peek().value === operator.PAREN.START
				) {
					next();

					while (
						peek() &&
						!(peek().type === "operator" && peek().value === operator.PAREN.END)
					) {
						let parameter: FNodeBlock = { type: "ParameterBlock", body: [] };

						if (peek().type === "operator" && peek().value === operator.COMMA)
							next();

						while (
							peek() &&
							!(
								peek().type === "operator" &&
								(peek().value === operator.COMMA ||
									peek().value === operator.PAREN.END)
							)
						) {
							let paramNode = parse(next());
							if (paramNode) parameter.body.push(paramNode);
						}

						parameter.body.filter((x) => x != undefined);
						node.parameters.push(parameter);
					}

					next();
				}

				if (
					peek() &&
					peek().type === "operator" &&
					(peek().value === operator.ARROW || peek().value === operator.COLON)
				) {
					next();
					node.yieldFunction = parse(next());
				}

				return node;
			} else if (token.type === FTokenType.STRING) {
				return { type: "StringLiteral", value: token.value };
			} else if (
				token.type === FTokenType.MEMORY &&
				token.value.startsWith("!")
			) {
				return { type: "NeedOperator", value: token.value.slice(1) };
			} else if (token.type === FTokenType.MEMORY) {
				return { type: "MemoryLiteral", value: token.value };
			} else if (token.type === FTokenType.NUMBER) {
				return { type: "NumberLiteral", value: token.value };
			} else if (token.type === FTokenType.BOOLEAN) {
				return { type: "BooleanLiteral", value: token.value };
			} else if (token.type === FTokenType.NULL) {
				return { type: "NullLiteral" };
			} else if (token.type === FTokenType.OPERATOR) {
				switch (token.value) {
					case operator.BRACE.START: {
						let body: FNodeAny[] = [];
						while (
							peek() &&
							!(
								peek().type === "operator" &&
								peek().value === operator.BRACE.END
							)
						) {
							body.push(parse(next()));
						}
						next();

						body.filter((x) => x != undefined);
						return { type: "Block", body };
					}

					case operator.PAREN.START: {
						let body: FNodeAny[] = [];
						while (
							peek() &&
							!(
								peek().type === "operator" &&
								peek().value === operator.PAREN.END
							)
						) {
							body.push(parse(next()));
						}
						next();

						body.filter((x) => x != undefined);
						return { type: "ParameterBlock", body };
					}

					case operator.ACCESS:
						return {
							type: "FunctionAccess",
							target: prev,
							call: parse(next())
						};

					case operator.PARENT:
						return {
							type: "ParentAccess",
							call: parse(next())
						};

					default:
						break;
				}
			} else if (token.type === FTokenType.NEWLINE) {
				return { type: "NewLine" };
			}

			return null;
		})();
		if (
			peek() &&
			peek().type == FTokenType.OPERATOR &&
			peek().value == operator.ACCESS
		) {
			parse(next());
		}

		return prev;
	}

	do {
		ast.body.push(parse(next()));
	} while (i < tokens.length);

	return ast;
}
