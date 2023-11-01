export class Scope {
    localFunctions = new Map();
    parent;
    returnValue = null;
    childScopes = new Map();
    returnSelf = false;
    type = "Block";
    subType = "Scope";
    body = [];
    matchCases = [];
    hasDefaultCase = false;
    constructor(parent = null) {
        this.parent = parent;
    }
    getFunction(name) {
        let path = name.split(".");
        if (path.length > 1 && this.childScopes.has(path[0]))
            return this.childScopes.get(path[0]).getFunction(path.slice(1).join("."));
        else if (this.localFunctions.has(name))
            return this.localFunctions.get(name);
        else if (this.parent)
            return this.parent.getFunction(name);
        else
            return undefined;
    }
    hasFunction(name) {
        let path = name.split(".");
        if (path.length > 1 && this.childScopes.has(path[0])) {
            return this.childScopes.get(path[0]).hasFunction(path.slice(1).join("."));
        }
        else if (this.localFunctions.has(name)) {
            return true;
        }
        else if (this.parent) {
            return this.parent.hasFunction(name);
        }
        else
            return false;
    }
    setFunction(name, value) {
        let path = name.split(".");
        if (path.length > 1 && this.childScopes.has(path[0]))
            return this.childScopes
                .get(path[0])
                .setFunction(path.slice(1).join("."), value);
        else if (this.localFunctions.has(name))
            return this.localFunctions.set(name, value);
        else if (this.parent)
            return this.parent.setFunction(name, value);
    }
    createSlot(name) {
        return new Slot(this, name);
    }
    return(value) {
        this.returnValue = value;
    }
}
export class Slot {
    scope;
    name;
    constructor(scope, name) {
        this.scope = scope;
        this.name = name;
    }
    set(value) {
        this.scope.localFunctions.set(this.name, value);
    }
    get() {
        return this.scope.localFunctions.get(this.name);
    }
}
