import { error } from "./error.js";
import { Scope } from "./scope.js";
import { FCallData, FNodeAny, FNodeBlock, FNodeType } from "./interfaces.js";
import { applyRuntimeFunctions } from "./runtimeFunctions.js";
import call from "./functionCall.js";
declare const fScript: { modules: Object; util: Object; isWeb: boolean };

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
				{ ...data, fnScope: null },
				node.yieldFunction,
				execute,
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

		case "NeedOperator": {
			let [source, name, ...path] = node.value.split(".");
			let shortcut = false;
			if (name == null) {
				shortcut = true;
				name = source;
				source = "local";
			}
			let id = `${source}.${name}`;

			if (!modules.has(id)) {
				if (!shortcut) {
					path = [name];
					name = source;
					source = "local";
					id = `${source}.${name}`;
				}
				if (!modules.has(id)) error(`Unknown module '${id}'.`, "Reference");
			}

			scope = modules.get(id);
			if (path.length > 0) {
				scope = call(
					scope.getFunction(path.join(".")),
					[],
					data,
					null,
					execute,
				) as Scope;
				if (!scope?.subType.endsWith("Scope"))
					error(`Unkown submodule '${node.value}'.`, "Reference");
				name = path.at(-1);
			}

			runtime.childScopes.set(name, scope);
			runtime.localFunctions.set(name, {
				type: "js",
				run() {
					return scope;
				},
			});
			return scope;
		}

		case "ModuleDeclaration": {
			if (!/^\w+$/g.test(node.value))
				error(`Invalid module name '${node.value}'.`, "Syntax");
			if (data.moduleName != null)
				error("Module name has already been defined.", "Syntax");
			dataRaw.moduleName = `${data.moduleSource}.${node.value}`;
			fScript.modules[dataRaw.moduleName] = new Scope();
			return node;
		}

		case "MemoryLiteral":
			return {
				slot: (data.fnScope ?? data.scope).createSlot(node.value),
				...node,
			};

		case "FunctionAccess": {
			let target = execute(node.target, data);
			if (!(target as Scope)?.subType?.endsWith("Scope"))
				error(
					`To access a function, I need a scope. Instead, I got a ${target?.type}.`,
					"Type",
				);

			return execute(node.call, { ...data, fnScope: target as Scope });
		}

		case "ParentAccess": {
			let parentScope = data.scope.parent;
			if (parentScope == null)
				error(
					"Scope is detached. Either you are trying to access the parent of the root scope, or something is wrong.",
					"Reference",
				);

			return execute(node.call, { ...data, fnScope: parentScope });
		}

		default:
			return node;
	}
}

const modules: Map<string, Scope> = new Map();
let runtime: Scope;

export function executer(ast: FNodeBlock, options: FCallData) {
	Object.entries(fScript.modules).forEach(([id, scope]) => {
		if (!modules.has(id)) modules.set(id, scope);
	});

	runtime = new Scope();
	applyRuntimeFunctions(runtime, execute);

	return execute(ast, options);
}
