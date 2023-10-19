import { FCallableAny, FNodeBlock, FNodeValue } from "./interfaces.js";
import { toFString } from "./runtimeFunctions.js";
import { Scope } from "./scope.js";

export class ListScope extends Scope implements FNodeBlock {
	array: FNodeValue[] = [];
	type: "Block" = "Block";
	body = [];
	scope = this;
	returnSelf = true;

	constructor(array: any[]) {
		super();
		this.array = array;

		this.applyFunctions();
	}

	private applyFunctions() {
		const array = this.array;

		this.localFunctions.set("push", {
			type: "js",
			run(item: FNodeValue) {
				array.push(item);
			}
		});

		this.localFunctions.set("pop", {
			type: "js",
			run() {
				return array.pop();
			}
		});

		this.localFunctions.set("delete", {
			type: "js",
			run(index: FNodeValue) {
				return array.splice(index.value, 1)[0];
			}
		});
	}

	hasFunction(name: string): boolean {
		if (!isNaN(parseInt(name))) return true;
		return super.hasFunction(name);
	}

	getFunction(name: string): FCallableAny {
		if (
			!isNaN(parseInt(name)) &&
			parseInt(name).toString().length === name.length
		) {
			const array = this.array;
			return {
				type: "js",
				run() {
					return array[parseInt(name)];
				}
			};
		}
		return super.getFunction(name);
	}

	toFString() {
		return `[${this.array
			.map((item: FNodeValue) => toFString(item))
			.join(", ")}]`;
	}
}

export function fromJSON(
	json: Object,
	parent: Scope = null,
	all: boolean = false
) {
	const scope = new Scope(parent);

	function storeObject(obj: Object, s: Scope) {
		let entries = all ? [] : Object.entries(json);
		if (all && obj === json)
			for (let p in obj) {
				if (!["view", "sourceCap"].includes(p)) entries.push([p, obj[p]]);
			}

		entries
			.filter(
				([_, value]) =>
					["string", "number", "boolean", "undefined"].includes(typeof value) ||
					Array.isArray(value)
			)
			.forEach(([key, value]) => {
				switch (typeof value) {
					// prettier-ignore
					case "string": {
						s.localFunctions.set(key, {
							type: "custom",
							run: { type: "StringLiteral", value }
						});
					} break;
					// prettier-ignore
					case "number": {
						s.localFunctions.set(key, { type: "custom", run: { type: "NumberLiteral", value }});
					} break;
					// prettier-ignore
					case "boolean": {
						s.localFunctions.set(key, {
							type: "custom",
							run: { type: "BooleanLiteral", value }
						});
					} break;
					// prettier-ignore
					case "undefined": {
						s.localFunctions.set(key, {
							type: "custom",
							run: { type: "NullLiteral" }
						});
					}

					// prettier-ignore
					default: {
						if (Array.isArray(value)) {
							let list = new ListScope(value);
							s.childScopes.set(key, list);
							s.setFunction(key, { type: "js", run: () => list });
							break;
						}

						let object = new Scope();
						storeObject(value, object);
						s.childScopes.set(key, object);
						s.setFunction(key, { type: "js", run: () => object });
					}
				}
			});
	}

	storeObject(json, scope);
	return scope;
}
