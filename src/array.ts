import { FCallableAny, FNodeBlock, FNodeValue } from "./interfaces.js";
import { toFString } from "./runtimeFunctions.js";
import { Scope } from "./scope.js";

export default class ListScope extends Scope implements FNodeBlock {
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
