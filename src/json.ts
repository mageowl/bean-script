import {
	FCallData,
	FCallableAny,
	FNodeAny,
	FNodeBlock,
	FNodeValue,
} from "./interfaces.js";
import toFString from "./toString.js";
import { Scope } from "./scope.js";
import { error } from "./error.js";
import { execute } from "./executer.js";

export class ListScope extends Scope implements FNodeBlock {
	array: FNodeValue[] = [];
	type: "Block" = "Block";
	body = [];
	scope = this;
	returnSelf = true;
	isArray = true;
	subType = "ListScope"

	constructor(array: any[]) {
		super();
		this.array = array;

		this.applyFunctions();
	}

	private applyFunctions() {
		const array = this.array;

		this.localFunctions.set("size", {
			type: "js",
			run() {
				return { type: "NumberLiteral", value: array.length };
			},
		});

		this.localFunctions.set("has", {
			type: "js",
			run(item) {
				return { type: "BooleanLiteral", value: array.some((i) => i?.type == item?.type && i?.value == item?.value) }
			}
		})

		this.localFunctions.set("push", {
			type: "js",
			run(item: FNodeValue) {
				array.push(item);
			},
		});

		this.localFunctions.set("pop", {
			type: "js",
			run() {
				return array.pop();
			},
		});

		this.localFunctions.set("delete", {
			type: "js",
			run(index: FNodeValue) {
				return array.splice(index.value, 1)[0];
			},
		});

		this.localFunctions.set("for", {
			type: "js",
			run(data: FCallData, yieldFunction) {
				const scope = new Scope(data.scope);
				let currentItem = null;
				scope.localFunctions.set("item", {
					type: "js",
					run() {
						return currentItem;
					},
				});

				let returnValues = [];
				array.forEach((item) => {
					currentItem = item;
					returnValues.push(execute(yieldFunction, { ...data, scope }));
				});

				return new ListScope(returnValues);
			},
		});

		this.localFunctions.set("join", {
			type: "js",
			run(seperator: FNodeValue) {
				if (seperator?.type != "StringLiteral")
					error(`Delimiter must be a string, instead got ${seperator?.type}`, "Type")
				
				return { type: "StringLiteral", value: array.map(toFString).join(seperator.value) }
			}
		})
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
					return array[parseInt(name)] ?? { type: "NullLiteral" };
				},
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

export class MapScope extends Scope implements FNodeBlock {
	map: Map<string, FNodeAny> = new Map();
	type: "Block" = "Block";
	body = [];
	scope = this;
	returnSelf = true;
	subType: string = "MapScope";

	constructor(kvPairs = []) {
		super();

		kvPairs.forEach(([key, value]: [FNodeValue, FNodeAny]) => {
			if (key?.type !== "StringLiteral")
				error(`Key must be a string, instead got ${key?.type}`, "Type");
			this.map.set(key.value, value);
		});

		this.applyFunctions();
	}

	private applyFunctions() {
		const map = this.map;

		this.localFunctions.set("size", {
			type: "js",
			run() {
				return { type: "NumberLiteral", value: map.size };
			},
		});

		this.localFunctions.set("set", {
			type: "js",
			run(key, value) {
				if (key?.type !== "StringLiteral")
					error(`Expected a string, instead got ${key?.type}`, "Type");
				map.set(key.value, value);
			},
		});

		this.localFunctions.set("delete", {
			type: "js",
			run(key) {
				if (key?.type !== "StringLiteral")
					error(`Expected a string, instead got ${key?.type}`, "Type");
				map.delete(key.value);
			},
		});

		this.localFunctions.set("get", {
			type: "js",
			run(key) {
				if (key?.type !== "StringLiteral")
					error(`Expected a string, instead got ${key?.type}`, "Type");
				map.get(key.value);
			},
		});

		this.localFunctions.set("has", {
			type: "js",
			run(key) {
				if (key?.type !== "StringLiteral")
					error(`Expected a string, instead got ${key?.type}`, "Type");
				return { type: "BooleanLiteral", value: map.has(key.value) };
			},
		});

		this.localFunctions.set("for", {
			type: "js",
			run(data, yieldFunction) {
				const scope = new Scope(data.scope);
				let currentValue = null;
				let currentKey = "";

				scope.localFunctions.set("key", {
					type: "js",
					run() {
						return { type: "StringLiteral", value: currentKey };
					},
				});
				scope.localFunctions.set("value", {
					type: "js",
					run() {
						return currentValue;
					},
				});

				let returnValues = [];
				Array.from(map.entries()).forEach(([key, value]) => {
					currentKey = key;
					currentValue = value;
					returnValues.push(key, execute(yieldFunction, { ...data, scope }));
				});
			},
		});
	}

	hasFunction(name: string): boolean {
		console.log(name);
		if (this.map.has(name)) return true;
		return super.hasFunction(name);
	}

	getFunction(name: string): FCallableAny {
		if (this.map.has(name)) {
			const value = this.map.get(name) ?? { type: "NullLiteral" };
			console.log(name);
			return {
				type: "js",
				run() {
					return value;
				},
			};
		}
		return super.getFunction(name);
	}

	toFString(): string {
		return (
			"{ " +
			Array.from(this.map.entries())
				.map(([v, k]) => `${v} = ${toFString(k)}`)
				.join(", ") +
			" }"
		);
	}
}

export function fromJSON(
	json: Object,
	parent: Scope = null,
	all: boolean = false,
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
					Array.isArray(value),
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
