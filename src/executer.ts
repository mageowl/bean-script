import { error } from "./error.js";
import { Scope } from "./scope.js";
import {
	FCallData,
	FNodeAny,
	FNodeFunctionAccess,
	FNodeType
} from "./interfaces.js";
import { applyRuntimeFunctions } from "./runtimeFunctions.js";
import call from "./functionCall.js";

export function execute(node: any, dataRaw: FCallData = {}): FNodeAny {
	const data: FCallData = { scope: runtime, ...dataRaw };
	let scope: Scope;

	if (node == null) return;
	switch (node.type as FNodeType) {
		case "FunctionCall":
			let fn = (data.fnScope ?? data.scope).getFunction(node.name);
			if (!fn) error(`Unknown value or function "${node.name}".`, "Reference");

			const response = call(
				fn,
				node.parameters,
				data,
				node.yieldFunction,
				execute
			);
			if (response != null) return response as FNodeAny;
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
			runtime.localFunctions.set(node.value, {
				type: "js",
				run() {
					return scope;
				}
			});
			return scope;

		case "MemoryLiteral":
			return {
				slot: (data.fnScope ?? data.scope).createSlot(node.value),
				...node
			};

		case "FunctionAccess": {
			let target = execute(node.target, data);
			if ((target as Scope)?.subType != "Scope")
				error(
					`To access a function inside a scope, I need a scope. Instead, I got a ${target?.type}.`,
					"Type"
				);

			return execute(node.call, { ...data, fnScope: target as Scope });
		}

		case "ParentAccess": {
			let parentScope = data.scope.parent;
			if (parentScope == null)
				error(
					"Scope is detached. Either you are trying to access the parent of the root scope, or something is wrong.",
					"Reference"
				);

			return execute(node.call, { ...data, fnScope: parentScope });
		}

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
