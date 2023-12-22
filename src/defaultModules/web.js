import { error } from "../error.js";
import { execute } from "../executer.js";
import { fromJSON } from "../json.js";
import { isWeb } from "../process.js";
import { Scope } from "../scope.js";
export class HTMLElementScope extends Scope {
    htmlEl;
    type = "Block";
    subType = "HTMLElementScope";
    body = [];
    scope = this; // NEED THIS TO SAVE OBJECT PROPERLY.
    destroyed = false;
    returnSelf = true;
    constructor(parent = null, element) {
        super(parent);
        this.htmlEl = element;
        this.applyFunctions();
    }
    applyFunctions() {
        let el = this.htmlEl;
        let self = this;
        this.localFunctions.set("id", {
            type: "js",
            run() {
                if (self.destroyed)
                    error("Trying to access a destroyed element.", "Web");
                return { type: "StringLiteral", value: el.id };
            }
        });
        this.localFunctions.set("setAttr", {
            type: "js",
            run(name, value) {
                if (self.destroyed)
                    error("Trying to edit a destroyed element.", "Web");
                el.setAttribute(name.value, value.value);
            }
        });
        this.localFunctions.set("get", {
            type: "js",
            run(property) {
                if (self.destroyed)
                    error("Trying to edit a destroyed element.", "Web");
                let value = el[property.value];
                return {
                    type: typeof value === "number" ? "NumberLiteral" : "StringLiteral",
                    value
                };
            }
        });
        this.localFunctions.set("text", {
            type: "js",
            run(text) {
                if (self.destroyed)
                    error("Trying to edit a destroyed element.", "Web");
                el.innerText = text.value;
            }
        });
        this.localFunctions.set("appendText", {
            type: "js",
            run(text) {
                if (self.destroyed)
                    error("Trying to edit a destroyed element.", "Web");
                el.innerText += text.value;
            }
        });
        this.localFunctions.set("add", {
            type: "js",
            run(element) {
                if (self.destroyed)
                    error("Trying to edit a destroyed element.", "Web");
                el.appendChild(element.htmlEl);
            }
        });
        this.localFunctions.set("on", {
            type: "js",
            run(event, data, yieldFunction) {
                if (self.destroyed)
                    error("Trying to access a destroyed element.", "Web");
                el.addEventListener(event.value, () => {
                    execute(yieldFunction, data);
                });
            }
        });
        this.localFunctions.set("destroy", {
            type: "js",
            run() {
                el.remove();
                self.destroyed = true;
            }
        });
    }
}
let consoleEl = null;
export function getConsoleEl() {
    return consoleEl;
}
const scope = new Scope();
if (isWeb) {
    let bodyEl = new HTMLElementScope(null, document.body);
    let trackedElements = [bodyEl];
    scope.childScopes.set("body", bodyEl);
    scope.localFunctions.set("useElementAsConsole", {
        type: "js",
        run(id) {
            if (id?.type != "StringLiteral") {
                if (id?.type === "NullLiteral")
                    consoleEl = null;
            }
            else
                consoleEl = document.getElementById(id.value);
        }
    });
    scope.localFunctions.set("get", {
        type: "js",
        run(selector, { scope }) {
            let el = new HTMLElementScope(scope, document.querySelector(selector.value));
            trackedElements.push(el);
            return el;
        }
    });
    scope.localFunctions.set("exists", {
        type: "js",
        run(selector, { scope }) {
            let el = document.querySelector(selector.value);
            return { type: "BooleanLiteral", value: el !== null };
        }
    });
    scope.localFunctions.set("make", {
        type: "js",
        run(type, { scope }) {
            let htmlEl = document.createElement(type.value);
            let el = new HTMLElementScope(scope, htmlEl);
            trackedElements.push(el);
            return el;
        }
    });
    scope.localFunctions.set("body", {
        type: "js",
        run() {
            return bodyEl;
        }
    });
    scope.localFunctions.set("on", {
        type: "js",
        run(event, data, yieldFunction) {
            window.addEventListener(event.value, (e) => {
                let scope = new Scope(data.scope);
                let eventScope = fromJSON(e, data.scope, true);
                yieldFunction.scope = scope;
                scope.childScopes.set("event", eventScope);
                execute(yieldFunction, data);
            });
        }
    });
    scope.localFunctions.set("input", {
        type: "js",
        run(text) {
            return { type: "StringLiteral", value: prompt(text.value) };
        }
    });
    scope.localFunctions.set("startLoop", {
        type: "js",
        run() {
            function tick() {
                window.dispatchEvent(new Event("tick"));
                requestAnimationFrame(tick);
            }
            tick();
        }
    });
    scope.localFunctions.set("width", {
        type: "js",
        run() {
            return { type: "NumberLiteral", value: window.innerWidth };
        }
    });
    scope.localFunctions.set("height", {
        type: "js",
        run() {
            return { type: "NumberLiteral", value: window.innerHeight };
        }
    });
}
export default scope;
