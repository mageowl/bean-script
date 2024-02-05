import { isWeb } from "./process.js";
import * as defaultModules from "./defaultModules/main.js";
import * as devUtilities from "./devUtilities.js";
import run from "./interperter.js";
import mapObject from "./util/mapObject.js";

globalThis.fScript = {
	util: devUtilities,
	modules: mapObject(defaultModules, ([k, v]) => ["default." + k, v]),
	isWeb,
};

if (isWeb) {
	console.log("FScript Web detected. Searching for script tags.");
	window.addEventListener("load", async () => {
		let scripts = document.querySelectorAll<HTMLScriptElement>(
			'script[type="text/f-script"], script[type="fscript"]',
		);

		// Compile scripts
		for (let scriptEl of scripts) {
			if (!scriptEl.src) {
				run(scriptEl.innerText, { moduleSource: "local" });
			} else {
				const text = await fetch(scriptEl.src).then((d) => d.text());
				run(text, { moduleSource: "local" });
			}
		}
	});
} else {
	console.log("NodeJS detected. Use src/node.js to run code on a server.");
}
