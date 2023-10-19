import { error } from "./error.js";
import { Scope } from "./scope.js";
import { FCallData, FNodeAny } from "./interfaces.js";
import { applyRuntimeFunctions } from "./runtimeFunctions.js";

export function execute(node, dataRaw: FCallData = {}): FNodeAny {
	const data: FCallData = { scope: runtime, ...dataRaw };
	let scope: Scope;

	if (node == null) return;
	switch (node.type) {
		case "FunctionCall":
			let fn = data.scope.getFunction(node.name);
			if (!fn) error(`Unknown value or function "${node.name}".`, "Reference");

			if (fn.type === "js") {
				return fn.run(
					...node.parameters.map((node) => execute(node, data)),
					data,
					node.yieldFunction
				);
			} else if (fn.type === "custom") {
				let params =
					(node.parameters.length ? node.parameters : data.parameters) || [];
				return execute(fn.run, {
					scope: fn.scope,
					parameters: params.map((node) => execute(node, data)),
					yieldFunction: node.yieldFunction
				});
			}
			break;

		case "Block":
			scope = node.scope ?? new Scope(data.scope);
			if (scope.returnSelf) return node;
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

		case "NeedOperator":
			if (!modules.has(node.value))
				error(`Unknown module '${node.value}'.`, "Reference");
			scope = modules.get(node.value);
			runtime.childScopes.set(node.value, scope);
			return scope;

		case "MemoryLiteral":
			return {
				slot: data.scope.createSlot(node.value),
				...node
			};

		default:
			return node;
	}
}

const modules = new Map();
const runtime = new Scope();
applyRuntimeFunctions(runtime, execute);

export function executer(ast, defaultModules = {}) {
	for (const mod in defaultModules) {
		if (Object.hasOwnProperty.call(defaultModules, mod)) {
			const scope = defaultModules[mod];
			modules.set(mod, scope);
		}
	}

	return execute(ast);
}
