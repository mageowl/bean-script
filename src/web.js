import { Scope } from "./scope.js";

export const isWeb = window != null;

let consoleEl = null;

export function getConsoleEl() {
	return consoleEl;
}

export const webModule = new Scope();

if (isWeb) {
	webModule.localFunctions.set("useElementAsConsole", {
		type: "js",
		run(id) {
			consoleEl = document.getElementById(id.value);
		}
	});

	webModule.localFunctions.set("input", {
		type: "js",
		run(text) {
			return { type: "StringLiteral", value: prompt(text.value) };
		}
	});
}
