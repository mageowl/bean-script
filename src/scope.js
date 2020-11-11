export class Scope {
	constructor(parent = null) {
		this.localFunctions = new Map();
		this.parent = parent;
		this.returnValue = null;
		this.childScopes = new Map();
	}

	getFunction(name) {
		let path = name.split(".");
		if (path.length > 1 && this.childScopes.has(path[0]))
			return this.childScopes.get(path[0]).getFunction(path.slice(1).join("."));
		else if (this.localFunctions.has(name))
			return this.localFunctions.get(name);
		else if (this.parent) return this.parent.getFunction(name);
		else return undefined;
	}

	return(value) {
		this.returnValue = value;
	}
}
