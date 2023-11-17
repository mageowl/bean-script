import { FCallableAny, FNode } from "./interfaces.js";

export class Scope {
	localFunctions: Map<string, FCallableAny> = new Map();
	parent: Scope;
	returnValue = null;
	childScopes: Map<string, Scope> = new Map();
	returnSelf = false;
	type: string = "Block";
	subType: string = "Scope";
	body: FNode[] = [];

	matchCases: Function[] = [];
	hasDefaultCase = false;

	ifValue: boolean = null;

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
			return this.childScopes.get(path[0]).hasFunction(path.slice(1).join("."));
		} else if (this.localFunctions.has(name)) {
			return true;
		} else if (this.parent) {
			return this.parent.hasFunction(name);
		} else return false;
	}

	setFunction(name: string, value: FCallableAny) {
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

	constructor(scope: Scope, name: string) {
		this.scope = scope;
		this.name = name;
	}

	set(value: FCallableAny) {
		this.scope.localFunctions.set(this.name, value);
	}

	get(): FCallableAny | void {
		return this.scope.localFunctions.get(this.name);
	}
}
