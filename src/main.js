import { executer } from "./executer.js";
import { lexer } from "./lexer.js";
import { parser } from "./parser.js";
import { isWeb } from "./process.js";
import { getENVData } from "./env.js";
import { loadModules } from "./moduleLoader.js";
import * as defaultModules from "./defaultModules/main.js";

if (isWeb) {
	console.log("FScript Web detected. Searching for script tags.");
	window.addEventListener("load", async () => {
		let scripts = document.querySelectorAll('script[type="text/f-script"]');

		// Compile scripts
		for (let scriptEl of scripts) {
			if (!scriptEl.src) {
				executer(parser(lexer(scriptEl.innerText)));
			} else {
				const ENV = await getENVData(scriptEl.src);
				const customModules = await loadModules(ENV.dependencies);
				fetch(scriptEl.src)
					.then((res) => res.text())
					.then((text) => {
						// executer(parser(lexer(text)), {
						// 	...defaultModules,
						// 	...customModules
						// });
						console.log(lexer(text));
					});
			}
		}
	});
} else {
	console.warn(
		"FScript could not detect web.\nNode.JS and other server-side javascript runtimes are currently not supported. (see https://github.com/seattleowl/f-script/issues/1.)"
	);
}
