import { error } from "../error.js";
import { execute } from "../executer.js";
import {
	FCallData,
	FNode,
	FNodeBlock,
	FNodeType,
	FNodeValue
} from "../interfaces.js";
import { isWeb } from "../process.js";
import { Scope } from "../scope.js";

class HTMLElementScope extends Scope {
	htmlEl: HTMLElement;
	type: FNodeType = "Block";
	body: FNode[] = [];
	scope = this;
	destroyed = false;

	constructor(parent = null, element) {
		super(parent);

		this.htmlEl = element;
		this._setup();
	}

	_setup() {
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
let bodyEl = new HTMLElementScope(null, document.body);
let trackedElements = [bodyEl];

export function getConsoleEl() {
	return consoleEl;
}

const scope = new Scope();

if (isWeb) {
	scope.localFunctions.set("useElementAsConsole", {
		type: "js",
		run(id) {
			consoleEl = document.getElementById(id.value);
		}
	});

	scope.localFunctions.set("getElement", {
		type: "js",
		run(selector, { scope }) {
			let el = new HTMLElementScope(
				scope,
				document.querySelector(selector.value)
			);
			trackedElements.push(el);
			return el;
		}
	});

	scope.localFunctions.set("createElement", {
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
			return { type: "Block", scope: bodyEl, body: [] };
		}
	});

	scope.localFunctions.set("input", {
		type: "js",
		run(text) {
			return { type: "StringLiteral", value: prompt(text.value) };
		}
	});
}

export default scope;
