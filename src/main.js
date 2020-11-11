import { executer } from "./executer.js";
import { lexer } from "./lexer.js";
import { parser } from "./parser.js";

window.addEventListener("load", () => {
	let scripts = document.querySelectorAll('script[type="text/f-script"]');

	// Compile scripts
	scripts.forEach((scriptEl) => {
		if (!scriptEl.src) {
			let code = executer(parser(lexer(scriptEl.innerText)));
			console.log(JSON.stringify(code, null, 2));
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
