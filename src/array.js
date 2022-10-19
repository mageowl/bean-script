import { toFString } from "./runtimeFunctions.js";
import { Scope } from "./scope.js";
export default class ListScope extends Scope {
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
                    return array[parseInt(name)];
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
