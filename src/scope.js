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

	hasFunction(name) {
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

	setFunction(name, value) {
		let path = name.split(".");
		if (path.length > 1 && this.childScopes.has(path[0]))
			return this.childScopes
				.get(path[0])
				.setFunction(path.slice(1).join("."), value);
		else if (this.localFunctions.has(name))
			return this.localFunctions.set(name, value);
		else if (this.parent) return this.parent.setFunction(name, value);
	}

	return(value) {
		this.returnValue = value;
	}
}
