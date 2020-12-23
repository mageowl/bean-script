import { isWeb } from "../process.js";
import { Scope } from "../scope.js";

let consoleEl = null;

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

	scope.localFunctions.set("input", {
		type: "js",
		run(text) {
			return { type: "StringLiteral", value: prompt(text.value) };
		}
	});
}

export default scope;
