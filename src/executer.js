// import { operator } from "./enums.js";
import { error } from "./error.js";
import { Scope } from "./scope.js";

function stringify(node) {
	switch (node.type) {
		case "NumberLiteral":
			return node.value.toString();
		case "StringLiteral":
			return node.value;

		case "MemoryLiteral":
			return `<${node.value}>`;

		default:
			return "[null]";
	}
}

function nodify(value) {
	switch (typeof value) {
		case "string":
			return { type: "StringLiteral", value };

		case "number":
			return { type: "NumberLiteral", value };

		default:
			break;
	}
}

function execute(node, data = { scope: runtime }) {
	if (node == undefined) {
		console.log("...");
		return;
	}
	switch (node.type) {
		case "FunctionCall":
			let fn = data.scope.getFunction(node.name);
			if (!fn) error(`Unkown value or function "${node.name}".`, "Reference");

			if (fn.type == "js") {
				return fn.run(
					...node.parameters.map((node) => execute(node, data)),
					data,
					node.yieldFunction
				);
			} else if (fn.type == "custom") {
				return execute(fn.run, {
					scope: data.scope,
					parameters: node.parameters.map((node) => execute(node, data)),
					yieldFunctionFunction: node.yieldFunction
				});
			}
			break;

		case "Block":
			let scope = new Scope(data.scope ?? runtime);
			node.body.forEach((node) => execute(node, { ...data, scope }));
			break;

		case "Program":
			node.body.forEach((node, i) => {
				execute(node, data);
			});
			break;

		case "ParameterBlock":
			let output = [];
			node.body.forEach((node) => output.push(execute(node, data)));
			return output[0];

		case "PlusOperator":
			let operands = node.operands
				.map((node) => execute(node, data))
				.map((node) =>
					node && node.type != "NumberLiteral"
						? stringify(node)
						: node
						? node.value
						: NaN
				);
			return nodify(operands[0] + operands[1]);

		default:
			return node;
	}
}

const runtime = new Scope();

runtime.localFunctions.set("def", {
	type: "js",
	run(memory, data, yieldFunction) {
		if (memory.type != "MemoryLiteral")
			error(`Expected MemoryLiteral, instead got ${memory.type}`, "Type");
		data.scope.localFunctions.set(memory.value, {
			type: "custom",
			run: yieldFunction
		});
	}
});

runtime.localFunctions.set("print", {
	type: "js",
	run(string, data) {
		console.log(stringify(execute(string, data)));
	}
});

runtime.localFunctions.set("param", {
	type: "js",
	run(paramIndex, data) {
		return data.parameters[paramIndex.value];
	}
});

runtime.localFunctions.set("yield", {
	type: "js",
	run(data) {
		execute(data.yieldFunction, data);
	}
});

export function executer(ast) {
	return execute(ast);
}
