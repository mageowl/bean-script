import { isWeb } from "../process.js";
import { Scope } from "../scope.js";

class HTMLElementScope extends Scope {
	constructor(parent = null, element) {
		super(parent);

		this.htmlEl = element;
		this.type = "Block";
		this.body = [];
		this.scope = this;
		this._setup();
	}

	_setup() {
		/**@type {HTMLElement} */
		let el = this.htmlEl;
		let self = this;

		this.localFunctions.set("id", {
			type: "js",
			run() {
				return { type: "StringLiteral", value: el.id };
			}
		});

		this.localFunctions.set("setAttr", {
			type: "js",
			run(name, value) {
				el.setAttribute(name.value, value.value);
			}
		});

		this.localFunctions.set("text", {
			type: "js",
			run(text) {
				el.innerText = text.value;
			}
		});

		this.localFunctions.set("appendText", {
			type: "js",
			run(text) {
				el.innerText += text.value;
			}
		});
		this.localFunctions.set("self", {
			type: "js",
			run(text) {
				return self;
			}
		});

		this.localFunctions.set("add", {
			type: "js",
			run(element) {
				el.appendChild(element.htmlEl);
			}
		});
	}
}

let consoleEl = null;
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
		run({ scope }) {
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
