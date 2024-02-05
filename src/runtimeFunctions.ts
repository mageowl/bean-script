import { ListScope, MapScope } from "./json.js";
import { getConsoleEl } from "./defaultModules/web.js";
import { error } from "./error.js";
import { FCallData, FNode, FNodeMemory, FNodeValue } from "./interfaces.js";
import { isDebug, isWeb } from "./process.js";
import call from "./functionCall.js";
import { Scope } from "./scope.js";
import toFString from "./util/toString.js";
declare const fScript: { modules: Object; util: Object; isWeb: boolean };

export function applyRuntimeFunctions(
	runtime,
	execute: (nodes, data) => FNode | any,
) {
	function addFunc(name: string, run: Function) {
		runtime.localFunctions.set(name, { type: "js", run });
	}

	addFunc("fn", function (memoryRaw: FNodeMemory, data, yieldFunction) {
		let memory = execute(memoryRaw, data);
		if (memory.type !== "MemoryLiteral")
			error(`Expected MemoryLiteral, instead got ${memory.type}`, "Type");
		if (memory.slot.exists)
			error(`Value <${memory.value}> is already defined.`, "Memory");
		memory.slot.set({
			type: "custom",
			scope: data.scope,
			run: yieldFunction,
		});
	});

	addFunc("let", function (memoryRaw: FNodeMemory, data, yieldFunction) {
		let memory = execute(memoryRaw, data);
		if (memory.type !== "MemoryLiteral")
			error(`Expected MemoryLiteral, instead got ${memory.type}`, "Type");
		if (memory.slot.exists)
			error(`Value <${memory.value}> is already defined.`, "Memory");

		function literal(node, data) {
			if (node.type.endsWith("Literal")) return node;
			if (node.type.endsWith("Block"))
				return execute(node, { ...data, returnScope: true });

			return literal(execute(node, data), data);
		}

		let value = literal(yieldFunction, data);
		if ((value as Scope)?.subType === "Scope") {
			memory.slot.set({
				type: "js",
				run() {
					return value;
				},
			});
		} else {
			memory.slot.set({
				type: "custom",
				scope: data.scope,
				run: value,
			});
		}
	});
	addFunc("set", function (memoryRaw, data, yieldFunction) {
		let memory = execute(memoryRaw, data);
		if (memory.type !== "MemoryLiteral")
			error(`Expected MemoryLiteral, instead got ${memory.type}`, "Type");
		if (!memory.slot.exists)
			error(`Value <${memory.value}> is not already defined.`, "Memory");

		function literal(node, data) {
			if (node.type.endsWith("Literal")) return node;
			if (node.type.endsWith("Block"))
				return execute(node, { ...data, returnScope: true });

			return literal(execute(node, data), data);
		}

		let value = literal(yieldFunction, data);
		if ((value as Scope)?.subType === "Scope") {
			memory.slot.set({
				type: "js",
				run() {
					return value;
				},
			});
		} else {
			memory.slot.set({
				type: "custom",
				scope: data.scope,
				run: value,
			});
		}
	});

	addFunc("del", function (memoryRaw, data: FCallData) {
		let memory = execute(memoryRaw, data);
		if (memory.type !== "MemoryLiteral")
			error(`Expected MemoryLiteral, instead got ${memory.type}`, "Type");
		if (!memory.slot.scope.hasFunction(memory.value))
			error(`Value <${memory.value}> is not defined.`, "Memory");

		memory.slot.scope.localFunctions.delete(memory?.value);
	});

	addFunc("query", function (memory, data) {
		if (memory.type !== "MemoryLiteral")
			error(`Expected MemoryLiteral, instead got ${memory.type}`, "Type");
		if (!data.scope.hasFunction(memory.value))
			error(`Value <${memory.value}> is not defined.`, "Memory");

		let fn;
		if (!memory?.slot) fn = execute(memory, data).slot.get();
		else fn = memory.slot.get();

		if (fn == null) return;

		return call(fn, [], data, null, execute);
	});

	addFunc("print", function (...components) {
		let string = components
			.slice(0, -2)
			.map((x) => toFString(x))
			.join("");
		if (isWeb && getConsoleEl()) {
			getConsoleEl().innerHTML += `<span>${string}</span><br>`;
		} else console.log(string);
	});
	addFunc("error", function (message) {
		console.error("[fscript] " + message.value);
	});

	addFunc("param", function (paramIndex, data) {
		if (!data.parameters?.length || data.parameters.length <= paramIndex.value)
			error(`Parameter ${paramIndex.value} was not given.`, "Reference");
		return data.parameters[paramIndex.value];
	});
	addFunc("params", function (data) {
		return new ListScope(data.parameters);
	});

	addFunc("yield", function (...params) {
		const data = params.at(-2);
		return execute(data.yieldFunction, {
			...data.yieldScope,
			parameters:
				params.length > 2 ? params.slice(0, -2) : data.yieldScope.parameters,
		});
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
				"Type",
			);
		return {
			type:
				numbers[0].type === "NumberLiteral" ? "NumberLiteral" : "StringLiteral",
			value: numbers.reduce(
				(num1, num2) =>
					(num1.value != undefined ? num1.value : num1) + num2.value,
			),
		};
	});

	addFunc("sub", function (num1, num2) {
		if (num1.type !== "NumberLiteral" || num2.type !== "NumberLiteral")
			error(`To subtract, both objects must be numbers.`, "Type");
		return {
			type: "NumberLiteral",
			value: num1.value - num2.value,
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
					: "".padStart(num1.value.length * num2.value, num1.value),
		};
	});

	addFunc("div", function (num1, num2) {
		if (num1.type !== "NumberLiteral" || num2.type !== "NumberLiteral")
			error(`To divide, both objects must be numbers.`, "Type");
		return {
			type: "NumberLiteral",
			value: num1.value / num2.value,
		};
	});

	addFunc("str", function (node) {
		return { type: "StringLiteral", value: toFString(node) };
	});

	addFunc("num", function (node) {
		return { type: "NumberLiteral", value: parseInt(node.value) };
	});

	addFunc("if", function (condition, data, yieldFunction) {
		let isTrue = execute(condition, data);
		if (isTrue.value === undefined)
			error(`Hmm... ${isTrue.type} is not type cast-able to boolean.`, "Type");
		data.scope.ifValue = !!isTrue?.value;
		if (isTrue?.value) {
			execute(yieldFunction, data);
			return { type: "BooleanLiteral", value: true };
		}
		return { type: "BooleanLiteral", value: false };
	});

	addFunc("else", function (data, yieldFunction) {
		let isTrue = data.scope.ifValue;
		if (isTrue == null)
			error(
				"Unexpected else function. Make sure to call if() first.",
				"Syntax",
			);
		data.scope.ifValue = null;
		if (!isTrue) {
			execute(yieldFunction, data);
		}
	});

	addFunc("elseIf", function (condition, data, yieldFunction) {
		if (data.scope.ifValue == null)
			error(
				"Unexpected else if function. Make sure to call if() first.",
				"Syntax",
			);
		if (!data.scope.ifValue) return;
		let isTrue = execute(condition, data);
		if (isTrue.value === undefined)
			error(`Hmm... ${isTrue.type} is not type cast-able to boolean.`, "Type");
		data.scope.ifValue = !!isTrue?.value;
		if (isTrue?.value) {
			execute(yieldFunction, data);
			return { type: "BooleanLiteral", value: true };
		}
		return { type: "BooleanLiteral", value: false };
	});

	addFunc("ifv", function (condition, valueTrue, valueFalse, data) {
		let isTrue = execute(condition, data);
		if (isTrue.value === undefined)
			error(`Hmm... ${isTrue.type} is not type cast-able to boolean.`, "Type");
		if (isTrue?.value) {
			return valueTrue;
		}
		return valueFalse;
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
			value: data.scope.hasFunction(memory.value),
		};
	});

	addFunc("eq", function (a, b, data) {
		let value = a?.type === b?.type && a?.value === b?.value;

		return { type: "BooleanLiteral", value };
	});

	addFunc("lt", function (a, b, data) {
		let value = a?.type === b?.type && a?.value < b?.value;

		return { type: "BooleanLiteral", value };
	});

	addFunc("gt", function (a, b, data) {
		let value = a?.type === b?.type && a?.value > b?.value;

		return { type: "BooleanLiteral", value };
	});

	addFunc("list", function (...params) {
		let array = params.slice(0, -2);

		return new ListScope(array);
	});
	addFunc("map", function (...params) {
		let map = params
			.slice(0, -2)
			.reduce(
				(arr, item) =>
					arr.length > 0
						? arr.at(-1).length === 2
							? arr.concat([[item]])
							: arr.slice(0, -1).concat([[arr.at(-1)[0], item]])
						: [[item]],
				[],
			);

		return new MapScope(map);
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
			value: Math.floor(Math.random() * (maxInt - minInt)) + minInt,
		};
	});

	addFunc("pow", function (num1, num2) {
		if (num1.type !== "NumberLiteral" || num2.type !== "NumberLiteral")
			error(`To raise to the nth power, both objects must be numbers.`, "Type");

		return {
			type: "NumberLiteral",
			value: num1.value ** num2.value,
		};
	});

	addFunc("repeat", function (times, data, yieldFunction) {
		if (times.type !== "NumberLiteral")
			error(
				`Parameter of repeat() should be a number. Instead, I got a ${times.type}.`,
				"Type",
			);

		let i = 0;
		while (i < times.value) {
			const result = execute(yieldFunction, data);
			if (result?.type === "NumberLiteral" && result?.value === 1) break;
			i++;
		}
	});

	addFunc("match", function (value, data, yieldFunction) {
		const matchScope: Scope = execute(yieldFunction, {
			...data,
			returnScope: true,
		});
		const valueLiteral = execute(value, data);
		for (let callback of matchScope.matchCases) {
			const res = callback(valueLiteral, data.scope);
			if (res != null) return res;
		}
	});
	addFunc("case", function (match, data: FCallData, yieldFunction) {
		if (data.scope.hasDefaultCase)
			error("Cannot add cases after default case.", "Syntax");

		const matchValue = execute(match, data);
		data.scope.matchCases.push((input) => {
			if (
				input?.type === matchValue?.type &&
				input?.value === matchValue?.value &&
				input?.type.endsWith("Literal")
			) {
				return execute(yieldFunction, { ...data, returnScope: false });
			}
			return null;
		});
	});
	addFunc("default", function (data: FCallData, yieldFunction) {
		if (data.scope.hasDefaultCase)
			error("Cannot have more than one default case.", "Syntax");

		data.scope.matchCases.push(() => {
			return execute(yieldFunction, { ...data, returnScope: false });
		});
		data.scope.hasDefaultCase = true;
	});

	addFunc("type", function (value) {
		return {
			type: "StringLiteral",
			value: value.type.replace("Literal", "").toLowerCase(),
		};
	});

	addFunc("size", function (string) {
		if (string?.type !== "StringLiteral")
			error(
				`Expected a string. Instead got a ${string?.type}. If you are trying to measure the length of a list, use list.size()`,
				"Type",
			);
		return { type: "NumberLiteral", value: string.value.length };
	});

	addFunc("split", function (string: FNodeValue, delimiter: FNodeValue) {
		if (string?.type !== "StringLiteral")
			error(`Expected a string. Instead got a ${string?.type}.`, "Type");
		if (delimiter?.type !== "StringLiteral")
			error(`Expected a string. Instead got a ${delimiter?.type}.`, "Type");
		return new ListScope(
			string.value
				.split(delimiter.value)
				.map((value: string) => ({ type: "StringLiteral", value })),
		);
	});

	addFunc("export", function (memoryRaw: FNodeMemory, data: FCallData) {
		const memory = execute(memoryRaw, data);
		if (memory?.type !== "MemoryLiteral")
			error(
				`Expected a memory literal. Instead got a ${memory?.type}.`,
				"Type",
			);
		if (!memory?.slot?.exists)
			error(`Trying to export undefined value <${memory.value}>`, "Reference");

		fScript.modules[data.moduleName].localFunctions.set(
			memory.value,
			memory.slot.get(),
		);
	});

	if (isDebug) {
		addFunc("__debug", function (data: FCallData) {
			console.log(data);
		});
	}
}
