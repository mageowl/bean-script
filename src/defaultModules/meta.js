import { error } from "../error.js";
import { execute } from "../executer.js";
import { Scope } from "../scope.js";
const scope = new Scope();
scope.localFunctions.set("dispatcher", {
    type: "js",
    run(data, yieldFunction) {
        const events = new Map();
        const dispatcherInterface = new Scope(data.scope);
        dispatcherInterface.localFunctions.set("event", {
            type: "js",
            run(name) {
                if (name.type !== "StringLiteral")
                    error(`Expected a string, instead got a ${name.type}`, "Type");
                events.set(name.value, []);
            }
        });
        dispatcherInterface.localFunctions.set("emit", {
            type: "js",
            run(name) {
                if (name.type !== "StringLiteral")
                    error(`Expected a string, instead got a ${name.type}`, "Type");
                if (!events.has(name.value))
                    error(`Event ${name.value} does not exist.`, "Reference");
                events.get(name.value).forEach((cb) => cb());
            }
        });
        const scope = execute(yieldFunction, {
            ...data,
            scope: dispatcherInterface,
            returnScope: true
        });
        scope.localFunctions.set("on", {
            type: "js",
            run(eventName, data, yieldFunction) {
                if (eventName.type !== "StringLiteral")
                    error(`Expected a string, instead got a ${eventName.type}`, "Type");
                if (!events.has(eventName.value))
                    error(`Event ${eventName.value} does not exist.`, "Reference");
                events.get(eventName.value).push(() => {
                    execute(yieldFunction, data);
                });
            }
        });
        return scope;
    }
});
scope.localFunctions.set("include", {
    type: "js",
    run(block, data) {
        const scope = block?.subType === "Scope"
            ? block
            : execute(block, {
                ...data,
                returnScope: true
            });
        scope.localFunctions.forEach((fn, name) => data.scope.localFunctions.set(name, fn));
        scope.childScopes.forEach((sc, name) => data.scope.localFunctions.set(name, sc));
    }
});
scope.localFunctions.set("bind", {
    type: "js",
    run(...params) {
        const data = params.at(-2);
        const yieldFunction = params.at(-1);
        execute(yieldFunction, {
            ...data,
            parameters: [...params.slice(0, -2), ...(data?.parameters ?? [])]
        });
    }
});
scope.localFunctions.set("binds", {
    type: "js",
    run(params, data, yieldFunction) {
        execute(yieldFunction, {
            ...data,
            parameters: [...params.array.slice(0, -2), ...(data?.parameters ?? [])]
        });
    }
});
scope.localFunctions.set("self", {
    type: "js",
    run(data) {
        return data.scope;
    }
});
export default scope;
