import { executer } from "./executer.js";
import { lexer } from "./lexer.js";
import { parser } from "./parser.js";
import { isWeb, webModule } from "./web.js";
import math from "./defaultModules/math.js";

if (isWeb) {
	console.log("FScript Web detected. Searching for script tags.");
	window.addEventListener("load", () => {
		let scripts = document.querySelectorAll('script[type="text/f-script"]');

		// Compile scripts
		scripts.forEach((scriptEl) => {
			if (!scriptEl.src) {
				executer(parser(lexer(scriptEl.innerText)));
			} else {
				fetch(scriptEl.src)
					.then((res) => res.text())
					.then((text) => {
						executer(parser(lexer(text)), { web: webModule, math });
						// console.log(parser(lexer(text)));
					});
			}
		});
	});
} else {
	console.warn(
		"FScript could not detect web.\nNode.JS and other server-side javascript runtimes are currently not supported. (see https://github.com/seattleowl/f-script/issues/1.)"
	);
}
