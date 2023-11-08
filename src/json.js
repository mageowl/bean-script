import { toFString } from "./runtimeFunctions.js";
import { Scope } from "./scope.js";
import { error } from "./error.js";
import { execute } from "./executer.js";
export class ListScope extends Scope {
    array = [];
    type = "Block";
    body = [];
    scope = this;
    returnSelf = true;
    constructor(array) {
        super();
        this.array = array;
        this.applyFunctions();
    }
    applyFunctions() {
        const array = this.array;
        this.localFunctions.set("size", {
            type: "js",
            run() {
                return { type: "NumberLiteral", value: array.length };
            }
        });
        this.localFunctions.set("push", {
            type: "js",
            run(item) {
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
            run(index) {
                return array.splice(index.value, 1)[0];
            }
        });
        this.localFunctions.set("for", {
            type: "js",
            run(data, yieldFunction) {
                const scope = new Scope(data.scope);
                let currentItem = null;
                scope.localFunctions.set("item", {
                    type: "js",
                    run() {
                        return currentItem;
                    }
                });
                array.forEach((item) => {
                    currentItem = item;
                    execute(yieldFunction, { ...data, scope });
                });
            }
        });
    }
    hasFunction(name) {
        if (!isNaN(parseInt(name)))
            return true;
        return super.hasFunction(name);
    }
    getFunction(name) {
        if (!isNaN(parseInt(name)) &&
            parseInt(name).toString().length === name.length) {
            const array = this.array;
            return {
                type: "js",
                run() {
                    return array[parseInt(name)] ?? { type: "NullLiteral" };
                }
            };
        }
        return super.getFunction(name);
    }
    toFString() {
        return `[${this.array
            .map((item) => toFString(item))
            .join(", ")}]`;
    }
}
export class MapScope extends Scope {
    map = new Map();
    type = "Block";
    body = [];
    scope = this;
    returnSelf = true;
    constructor(kvPairs = []) {
        super();
        kvPairs.forEach(([key, value]) => {
            if (key?.type !== "StringLiteral")
                error(`Key must be a string, instead got ${key?.type}`, "Type");
            this.map.set(key.value, value);
        });
        this.applyFunctions();
    }
    applyFunctions() {
        const map = this.map;
        this.localFunctions.set("set", {
            type: "js",
            run(key, value) {
                if (key?.type !== "StringLiteral")
                    error(`Expected a string, instead got ${key?.type}`, "Type");
                map.set(key.value, value);
            }
        });
        this.localFunctions.set("get", {
            type: "js",
            run(key) {
                if (key?.type !== "StringLiteral")
                    error(`Expected a string, instead got ${key?.type}`, "Type");
                map.get(key.value);
            }
        });
        this.localFunctions.set("has", {
            type: "js",
            run(key) {
                if (key?.type !== "StringLiteral")
                    error(`Expected a string, instead got ${key?.type}`, "Type");
                return { type: "BooleanLiteral", value: map.has(key.value) };
            }
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
                    }
                });
                scope.localFunctions.set("value", {
                    type: "js",
                    run() {
                        return currentValue;
                    }
                });
                Array.from(map.entries()).forEach(([key, value]) => {
                    currentKey = key;
                    currentValue = value;
                    execute(yieldFunction, { ...data, scope });
                });
            }
        });
    }
    hasFunction(name) {
        console.log(name);
        if (this.map.has(name))
            return true;
        return super.hasFunction(name);
    }
    getFunction(name) {
        if (this.map.has(name)) {
            const value = this.map.get(name) ?? { type: "NullLiteral" };
            console.log(name);
            return {
                type: "js",
                run() {
                    return value;
                }
            };
        }
        return super.getFunction(name);
    }
    toFString() {
        return ("{ " +
            Array.from(this.map.entries())
                .map(([v, k]) => `${v} = ${toFString(k)}`)
                .join(", ") +
            " }");
    }
}
export function fromJSON(json, parent = null, all = false) {
    const scope = new Scope(parent);
    function storeObject(obj, s) {
        let entries = all ? [] : Object.entries(json);
        if (all && obj === json)
            for (let p in obj) {
                if (!["view", "sourceCap"].includes(p))
                    entries.push([p, obj[p]]);
            }
        entries
            .filter(([_, value]) => ["string", "number", "boolean", "undefined"].includes(typeof value) ||
            Array.isArray(value))
            .forEach(([key, value]) => {
            switch (typeof value) {
                // prettier-ignore
                case "string":
                    {
                        s.localFunctions.set(key, {
                            type: "custom",
                            run: { type: "StringLiteral", value }
                        });
                    }
                    break;
                // prettier-ignore
                case "number":
                    {
                        s.localFunctions.set(key, { type: "custom", run: { type: "NumberLiteral", value } });
                    }
                    break;
                // prettier-ignore
                case "boolean":
                    {
                        s.localFunctions.set(key, {
                            type: "custom",
                            run: { type: "BooleanLiteral", value }
                        });
                    }
                    break;
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
