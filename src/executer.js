// import { operator } from "./enums.js";
// import { literal } from "./enums.js";
import { error } from "./error.js";
import { Scope } from "./scope.js";

function stringify(node) {
	if (!node) return;
	switch (node.type) {
		case "NumberLiteral":
			return node.value.toString();
		case "StringLiteral":
			return node.value;

		case "BooleanLiteral":
			return node.value.toString();

		case "MemoryLiteral":
			return `<${node.value}>`;

		default:
			return "[null]";
	}
}

function execute(node, data = { scope: runtime }) {
	if (!node) return;
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
				let params =
					(node.parameters.length ? node.parameters : data.parameters) || [];
				return execute(fn.run, {
					scope: data.scope,
					parameters: params.map((node) => execute(node, data)),
					yieldFunctionFunction: node.yieldFunction
				});
			}
			break;

		case "Block":
			let scope = new Scope(data.scope);
			node.body.forEach((node) => execute(node, { ...data, scope }));
			if (scope.returnValue != null && !data.returnScope)
				return scope.returnValue;
			else if (data.returnScope) return scope;
			break;

		case "Program":
			node.body.forEach((node) => {
				execute(node, data);
			});
			break;

		case "ParameterBlock":
			let output = [];
			node.body.forEach((node) => output.push(execute(node, data)));
			return output.slice(-1)[0];

		default:
			return node;
	}
}

const runtime = new Scope();

runtime.localFunctions.set("def", {
	type: "js",
	run(memory, { scope }, yieldFunction) {
		if (memory.type != "MemoryLiteral")
			error(`Expected MemoryLiteral, instead got ${memory.type}`, "Type");
		if (scope.hasFunction(memory.value))
			error(`Value <${memory.value}> is already defined.`, "Memory");
		scope.localFunctions.set(memory.value, {
			type: "custom",
			run: yieldFunction
		});
	}
});

runtime.localFunctions.set("set", {
	type: "js",
	run(memory, data, yieldFunction) {
		if (memory.type != "MemoryLiteral")
			error(`Expected MemoryLiteral, instead got ${memory.type}`, "Type");
		if (!data.scope.hasFunction(memory.value))
			error(`Value <${memory.value}> is not defined.`, "Memory");

		function literal(node, data) {
			if (
				node.type == "StringLiteral" ||
				node.type == "NumberLiteral" ||
				node.type == "MemoryLiteral" ||
				node.type == "BooleanLiteral"
			)
				return node;

			return literal(execute(node, data));
		}

		data.scope.setFunction(memory.value, {
			type: "custom",
			run: literal(yieldFunction, data)
		});
	}
});

runtime.localFunctions.set("print", {
	type: "js",
	run(string, data) {
		if (document.getElementById("fscript-logs")) {
			document.getElementById("fscript-logs").innerHTML += `<span>${stringify(
				execute(string, data)
			)}</span><br>`;
		} else console.log(stringify(execute(string, data)));
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

runtime.localFunctions.set("return", {
	type: "js",
	run(value, data) {
		data.scope.return(value);
		return value;
	}
});

runtime.localFunctions.set("add", {
	type: "js",
	run(...params) {
		let nums = params.slice(0, -2);
		let noTypeMatch = nums.find((num) => num.type != nums[0].type);

		if (noTypeMatch)
			error(
				`Cannot add a ${noTypeMatch.type} to a ${nums[0].type}. Please type cast using str()`,
				"Type"
			);
		return {
			type: nums[0].type == "NumberLiteral" ? "NumberLiteral" : "StringLiteral",
			value: nums.reduce(
				(num1, num2) =>
					(num1.value != undefined ? num1.value : num1) + num2.value
			)
		};
	}
});

runtime.localFunctions.set("sub", {
	type: "js",
	run(num1, num2) {
		if (num1.type != "NumberLiteral" || num2.type != "NumberLiteral")
			error(`To subtract, both objects must be numbers.`, "Type");
		return {
			type: "NumberLiteral",
			value: num1.value - num2.value
		};
	}
});

runtime.localFunctions.set("mul", {
	type: "js",
	run(num1, num2) {
		if (num2.type != "NumberLiteral")
			error(`To multiply, the second object must be a number.`, "Type");
		return {
			type: num1.type == "NumberLiteral" ? "NumberLiteral" : "StringLiteral",
			value:
				num1.type == "NumberLiteral"
					? num1.value * num2.value
					: "".padStart(num1.value.length * num2.value, num1.value)
		};
	}
});

runtime.localFunctions.set("div", {
	type: "js",
	run(num1, num2) {
		if (num1.type != "NumberLiteral" || num2.type != "NumberLiteral")
			error(`To divide, both objects must be numbers.`, "Type");
		return {
			type: "NumberLiteral",
			value: num1.value / num2.value
		};
	}
});

runtime.localFunctions.set("str", {
	type: "js",
	run(node) {
		return { type: "StringLiteral", value: stringify(node) };
	}
});

runtime.localFunctions.set("num", {
	type: "js",
	run(node) {
		return { type: "NumberLiteral", value: parseInt(node.value) };
	}
});

runtime.localFunctions.set("scope", {
	type: "js",
	run(memory, data, yieldFunction) {
		if (yieldFunction.type != "Block")
			error(
				`Yield to scope must be a block. Instead, I got a ${yieldFunction.type}`
			);

		data.scope.childScopes.set(
			memory.value,
			execute(yieldFunction, { ...data, returnScope: true })
		);
	}
});

runtime.localFunctions.set("if", {
	type: "js",
	run(condition, data, yieldFunction) {
		let isTrue = execute(condition, data);
		if (isTrue.value == undefined)
			error(`Hmm... ${isTrue.type} is not type castable to boolean.`, "Type");
		if (isTrue?.value) {
			execute(yieldFunction, data);
			return { type: "BooleanLiteral", value: true };
		}
		return { type: "BooleanLiteral", value: false };
	}
});

runtime.localFunctions.set("unless", {
	type: "js",
	run(condition, data, yieldFunction) {
		let isTrue = execute(condition, data);
		if (isTrue.value == undefined)
			error(`${isTrue.type} is not type castable to boolean.`, "Type");
		if (!isTrue?.value) {
			execute(yieldFunction, data);
			return { type: "BooleanLiteral", value: true };
		}
		return { type: "BooleanLiteral", value: false };
	}
});

runtime.localFunctions.set("not", {
	type: "js",
	run(bool, data) {
		let isTrue = execute(bool, data);
		if (isTrue.value == undefined)
			error(`${isTrue.type} is not type castable to boolean.`, "Type");
		return { type: "BooleanLiteral", value: !isTrue?.value };
	}
});

runtime.localFunctions.set("is", {
	type: "js",
	run(node, data, yieldFunction) {
		let obj = execute(node, data);
		let match = execute(yieldFunction, data);
		let value = obj.value == match.value && obj.type == match.type;

		return { type: "BooleanLiteral", value };
	}
});

runtime.localFunctions.set("abs", {
	type: "js",
	run(number) {
		if (number.type != "NumberLiteral") {
			error(
				`I need a number to get the absolute value. I got a ${number.type}.`,
				"Type"
			);
		}

		return { type: "NumberLiteral", value: Math.abs(number.value) };
	}
});

runtime.localFunctions.set("input", {
	type: "js",
	run(text) {
		return { type: "StringLiteral", value: prompt(text.value) };
	}
});

export function executer(ast) {
	return execute(ast);
}
