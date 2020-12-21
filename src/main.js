import { executer } from "./executer.js";
import { lexer } from "./lexer.js";
import { parser } from "./parser.js";
import { isWeb } from "./web.js";

if (isWeb) {
	console.log("FScript Web detected. Searching for script tags.");
	window.addEventListener("load", () => {
		let scripts = document.querySelectorAll('script[type="text/f-script"]');

		// Compile scripts
		scripts.forEach((scriptEl) => {
			if (!scriptEl.src) {
				let code = executer(parser(lexer(scriptEl.innerText)));
			} else {
				fetch(scriptEl.src)
					.then((res) => res.text())
					.then((text) => {
						let code = executer(parser(lexer(text)));
						console.log(JSON.stringify(code, null, 2));
					});
			}
		});
	});
} else {
	console.warn(
		"FScript could not detect web.\nNode.JS and other server-side javascript runtimes are currently not supported. (see https://github.com/seattleowl/f-script/issues/1.)"
	);
}
