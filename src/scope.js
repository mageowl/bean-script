export class Scope {
	constructor(parent = null) {
		this.localFunctions = new Map();
		this.parent = parent;
	}

	getFunction(name) {
		if (this.localFunctions.has(name)) return this.localFunctions.get(name);
		else if (this.parent) return this.parent.getFunction(name);
		else return undefined;
	}
}
