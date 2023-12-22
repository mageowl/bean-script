import { isWeb } from "./process.js";
import * as defaultModules from "./defaultModules/main.js";
import * as devUtilities from "./devUtilities.js";
import run from "./interperter.js";

globalThis.fScript = {
	util: devUtilities,
	modules: { ...defaultModules },
	isWeb,
};

if (isWeb) {
	console.log("FScript Web detected. Searching for script tags.");
	window.addEventListener("load", async () => {
		let scripts = document.querySelectorAll<HTMLScriptElement>(
			'script[type="text/f-script"]',
		);

		// Compile scripts
		for (let scriptEl of scripts) {
			if (!scriptEl.src) {
				run(scriptEl.innerText);
			} else {
				const text = await fetch(scriptEl.src).then((d) => d.text());
				run(text);
			}
		}
	});
} else {
	console.log("NodeJS detected. Use src/node.js to run code on a server.");
}
