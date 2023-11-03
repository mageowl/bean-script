import { error } from "../error.js";
import { execute } from "../executer.js";
import { FCallData, FNode, FNodeBlock, FNodeValue } from "../interfaces.js";
import { fromJSON } from "../json.js";
import { isWeb } from "../process.js";
import { Scope } from "../scope.js";

export class HTMLElementScope extends Scope implements FNodeBlock {
	htmlEl: HTMLElement;
	type: "Block" = "Block";
	subType: "HTMLElementScope" = "HTMLElementScope";
	body: FNode[] = [];
	scope = this; // NEED THIS TO SAVE OBJECT PROPERLY.
	destroyed = false;
	returnSelf = true;

	constructor(parent = null, element) {
		super(parent);

		this.htmlEl = element;
		this.applyFunctions();
	}

	private applyFunctions() {
		let el: HTMLElement = this.htmlEl;
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
			run(name: FNodeValue, value: FNodeValue) {
				if (self.destroyed) error("Trying to edit a destroyed element.", "Web");
				el.setAttribute(name.value, value.value);
			}
		});

		this.localFunctions.set("get", {
			type: "js",
			run(property: FNodeValue): FNodeValue {
				if (self.destroyed) error("Trying to edit a destroyed element.", "Web");

				let value = el[property.value];
				return {
					type: typeof value === "number" ? "NumberLiteral" : "StringLiteral",
					value
				};
			}
		});

		this.localFunctions.set("text", {
			type: "js",
			run(text: FNodeValue) {
				if (self.destroyed) error("Trying to edit a destroyed element.", "Web");
				el.innerText = text.value;
			}
		});

		this.localFunctions.set("appendText", {
			type: "js",
			run(text: FNodeValue) {
				if (self.destroyed) error("Trying to edit a destroyed element.", "Web");
				el.innerText += text.value;
			}
		});

		this.localFunctions.set("add", {
			type: "js",
			run(element: HTMLElementScope) {
				if (self.destroyed) error("Trying to edit a destroyed element.", "Web");
				el.appendChild(element.htmlEl);
			}
		});

		this.localFunctions.set("on", {
			type: "js",
			run(event: FNodeValue, data: FCallData, yieldFunction: FNodeBlock) {
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

let consoleEl: HTMLElement | null = null;

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
		run(id: FNodeValue) {
			if (id?.type != "StringLiteral") {
				if (id?.type === "NullLiteral") consoleEl = null;
			} else consoleEl = document.getElementById(id.value);
		}
	});

	scope.localFunctions.set("get", {
		type: "js",
		run(selector: FNodeValue, { scope }: FNodeBlock) {
			let el = new HTMLElementScope(
				scope,
				document.querySelector(selector.value)
			);
			trackedElements.push(el);
			return el;
		}
	});
	scope.localFunctions.set("exists", {
		type: "js",
		run(selector: FNodeValue, { scope }: FNodeBlock) {
			let el = document.querySelector(selector.value);
			return { type: "BooleanLiteral", value: el !== null };
		}
	});

	scope.localFunctions.set("make", {
		type: "js",
		run(type: FNodeValue, { scope }: FNodeBlock) {
			let htmlEl = document.createElement(type.value);
			let el = new HTMLElementScope(scope, htmlEl);
			trackedElements.push(el);
			return el;
		}
	});

	scope.localFunctions.set("body", {
		type: "js",
		run() {
			return { type: "Block", scope: bodyEl, body: [] };
		}
	});

	scope.localFunctions.set("on", {
		type: "js",
		run(event: FNodeValue, data: FCallData, yieldFunction: FNodeBlock) {
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
		run(text: FNodeValue) {
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
}

export default scope;
