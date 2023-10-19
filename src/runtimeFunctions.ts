import { ListScope } from "./json.js";
import { getConsoleEl } from "./defaultModules/web.js";
import { error } from "./error.js";
import { FNodeAny, FNodeMemory, FNodeValue } from "./interfaces.js";
import { isDebug, isWeb } from "./process.js";

export function toFString(node) {
	if (!node) return;
	if (node.toFString != null) return node.toFString();
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

export function applyRuntimeFunctions(runtime, execute) {
	function addFunc(name: string, run: Function) {
		runtime.localFunctions.set(name, { type: "js", run });
	}

	addFunc("fn", function (memoryRaw, data, yieldFunction) {
		let memory = execute(memoryRaw, data);
		if (memory.type !== "MemoryLiteral")
			error(`Expected MemoryLiteral, instead got ${memory.type}`, "Type");
		if (memory.slot.scope.hasFunction(memory.value))
			error(`Value <${memory.value}> is already defined.`, "Memory");
		memory.slot.set({
			type: "custom",
			scope: data.scope,
			run: yieldFunction
		});
	});

	addFunc("let", function (memoryRaw, data, yieldFunction) {
		let memory = execute(memoryRaw, data);
		if (memory.type !== "MemoryLiteral")
			error(`Expected MemoryLiteral, instead got ${memory.type}`, "Type");
		if (data.scope.hasFunction(memory.value))
			error(`Value <${memory.value}> is already defined.`, "Memory");

		function literal(node, data) {
			if (node.type.endsWith("Literal")) return execute(node, data);

			return literal(execute(node, data), data);
		}

		memory.slot.set({
			type: "custom",
			scope: data.scope,
			run: literal(yieldFunction, data)
		});
	});

	addFunc("set", function (memory, data, yieldFunction) {
		if (memory.type !== "MemoryLiteral")
			error(`Expected MemoryLiteral, instead got ${memory.type}`, "Type");
		if (!data.scope.hasFunction(memory.value))
			error(`Value <${memory.value}> is not defined.`, "Memory");

		function literal(node, data = null) {
			if (node.type.endsWith("Literal")) return node;

			return literal(execute(node, data));
		}

		data.scope.setFunction(memory.value, {
			type: "custom",
			run: literal(yieldFunction, data)
		});
	});

	addFunc("print", function (string: FNodeAny, data) {
		if (isWeb && getConsoleEl()) {
			getConsoleEl().innerHTML += `<span>${toFString(
				execute(string, data)
			)}</span><br>`;
		} else console.log(toFString(execute(string, data)));
	});

	addFunc("param", function (paramIndex, data) {
		return data.parameters[paramIndex.value];
	});

	addFunc("yield", function (data) {
		return execute(data.yieldFunction, data);
	});

	addFunc("return", function (value, data) {
		data.scope.return(value);
		return value;
	});

	addFunc("add", function (...params) {
		let numbers = params.slice(0, -2);
		let noTypeMatch = numbers.find((num) => num.type !== numbers[0].type);

		if (noTypeMatch)
			error(
				`Cannot add a ${noTypeMatch.type} to a ${numbers[0].type}. Please type cast using str()`,
				"Type"
			);
		return {
			type:
				numbers[0].type === "NumberLiteral" ? "NumberLiteral" : "StringLiteral",
			value: numbers.reduce(
				(num1, num2) =>
					(num1.value != undefined ? num1.value : num1) + num2.value
			)
		};
	});

	addFunc("sub", function (num1, num2) {
		if (num1.type !== "NumberLiteral" || num2.type !== "NumberLiteral")
			error(`To subtract, both objects must be numbers.`, "Type");
		return {
			type: "NumberLiteral",
			value: num1.value - num2.value
		};
	});

	addFunc("mul", function (num1, num2) {
		if (num2.type !== "NumberLiteral")
			error(`To multiply, the second object must be a number.`, "Type");
		return {
			type: num1.type === "NumberLiteral" ? "NumberLiteral" : "StringLiteral",
			value:
				num1.type === "NumberLiteral"
					? num1.value * num2.value
					: "".padStart(num1.value.length * num2.value, num1.value)
		};
	});

	addFunc("div", function (num1, num2) {
		if (num1.type !== "NumberLiteral" || num2.type !== "NumberLiteral")
			error(`To divide, both objects must be numbers.`, "Type");
		return {
			type: "NumberLiteral",
			value: num1.value / num2.value
		};
	});

	addFunc("str", function (node) {
		return { type: "StringLiteral", value: toFString(node) };
	});

	addFunc("num", function (node) {
		return { type: "NumberLiteral", value: parseInt(node.value) };
	});

	addFunc("obj", function (memoryRaw: FNodeAny, data, yieldFunction) {
		let memory: FNodeMemory = execute(memoryRaw, data);
		let block = yieldFunction;

		if (memory.type !== "MemoryLiteral") {
			error(
				`The first parameter for obj() must be a memory literal. Instead, I got a ${memory.type}`,
				"Type"
			);
		}

		function check() {
			if (block.type === "FunctionCall") {
				block = execute(yieldFunction, data);
				check();
			} else if (!block.type.startsWith("Block") && block.type !== "FunctionCall") error(`Yield to obj must be a block. Instead, I got a ${block.type}`, "Type");
		}

		check();

		let scope = execute(block, { ...data, returnScope: true });
		memory.slot.scope.childScopes.set(memory.slot.name, scope);

		memory.slot.set({
			type: "js",
			run() {
				return scope;
			}
		});
	});

	addFunc("if", function (condition, data, yieldFunction) {
		let isTrue = execute(condition, data);
		if (isTrue.value === undefined)
			error(`Hmm... ${isTrue.type} is not type cast-able to boolean.`, "Type");
		if (isTrue?.value) {
			execute(yieldFunction, data);
			return { type: "BooleanLiteral", value: true };
		}
		return { type: "BooleanLiteral", value: false };
	});

	addFunc("unless", function (condition, data, yieldFunction) {
		let isTrue = execute(condition, data);
		if (isTrue.value === undefined)
			error(`${isTrue.type} is not type cast-able to boolean.`, "Type");
		if (!isTrue?.value) {
			execute(yieldFunction, data);
			return { type: "BooleanLiteral", value: true };
		}
		return { type: "BooleanLiteral", value: false };
	});

	addFunc("not", function (bool, data) {
		let isTrue = execute(bool, data);
		if (isTrue.value === undefined)
			error(`${isTrue.type} is not type cast-able to boolean.`, "Type");
		return { type: "BooleanLiteral", value: !isTrue?.value };
	});

	addFunc("exists", function (memory, data) {
		return {
			type: "BooleanLiteral",
			value: data.scope.hasFunction(memory.value)
		};
	});

	addFunc("is", function (node, data, yieldFunction) {
		let obj = execute(node, data);
		let match = execute(yieldFunction, data);
		let value = obj.value === match.value && obj.type === match.type;

		return { type: "BooleanLiteral", value };
	});

	addFunc("less", function (node, data, yieldFunction) {
		let obj: FNodeValue = execute(node, data);
		let match: FNodeValue = execute(yieldFunction, data);
		let value = obj.value < match.value && obj.type === match.type;

		return { type: "BooleanLiteral", value };
	});

	addFunc("more", function (node, data, yieldFunction) {
		let obj: FNodeValue = execute(node, data);
		let match: FNodeValue = execute(yieldFunction, data);
		let value = obj.value > match.value && obj.type === match.type;

		return { type: "BooleanLiteral", value };
	});

	addFunc("list", function (...params) {
		let array = params.slice(0, -2);

		return new ListScope(array);
	});

	addFunc("rand", function (min, max, data) {
		if (min?.type != "NumberLiteral" && min?.type != undefined)
			error(`Minimum value ${min.type} is not a NumberLiteral`, "Type");
		if (max?.type != "NumberLiteral" && max?.type != undefined)
			error(`Maximum value ${max.type} is not a NumberLiteral`, "Type");

		let minInt = data == null ? 0 : min.value != null ? min.value : min;
		let maxInt = max == null ? 0 : max.value != null ? max.value : min;
		if (max == null) return { type: "NumberLiteral", value: Math.random() };

		return {
			type: "NumberLiteral",
			value: Math.floor(Math.random() * (maxInt - minInt)) + minInt
		};
	});

	if (isDebug) {
		addFunc("__debug", function (data) {
			console.log(data);
		});
	}
}
