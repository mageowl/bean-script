import { FCallableAny } from "./interfaces";

export class Scope {
	localFunctions: Map<string, FCallableAny> = new Map();
	parent: Scope;
	returnValue = null;
	childScopes = new Map();
	returnSelf = false;

	constructor(parent: Scope = null) {
		this.parent = parent;
	}

	getFunction(name: string) {
		let path = name.split(".");
		if (path.length > 1 && this.childScopes.has(path[0]))
			return this.childScopes.get(path[0]).getFunction(path.slice(1).join("."));
		else if (this.localFunctions.has(name))
			return this.localFunctions.get(name);
		else if (this.parent) return this.parent.getFunction(name);
		else return undefined;
	}

	hasFunction(name: string): boolean {
		let path = name.split(".");
		if (path.length > 1 && this.childScopes.has(path[0])) {
			// console.log(`Child Scope: ${path[0]}`);
			return this.childScopes.get(path[0]).hasFunction(path.slice(1).join("."));
		} else if (this.localFunctions.has(name)) {
			// console.log(`Found: ${name}, Local Functions: `, this.localFunctions);
			return true;
		} else if (this.parent) {
			// console.log("Parent: ", this.parent);
			return this.parent.hasFunction(name);
		} else return false;
	}

	setFunction(name: string, value) {
		let path = name.split(".");
		if (path.length > 1 && this.childScopes.has(path[0]))
			return this.childScopes
				.get(path[0])
				.setFunction(path.slice(1).join("."), value);
		else if (this.localFunctions.has(name))
			return this.localFunctions.set(name, value);
		else if (this.parent) return this.parent.setFunction(name, value);
	}

	createSlot(name: string) {
		return new Slot(this, name);
	}

	return(value: string) {
		this.returnValue = value;
	}
}

export class Slot {
	scope: Scope;
	name: string;
	used: boolean;

	constructor(scope: Scope, name: string) {
		this.scope = scope;
		this.name = name;
	}

	set(value: any) {
		if (this.used) return;
		this.scope.localFunctions.set(this.name, value);
	}
}