import { Memory } from "./memory.js";
import { lexer } from "./lexer.js";

window.onload = () => {
	let scripts = document.querySelectorAll('script[type="text/f-script"]');

	// Compile scripts
	scripts.forEach((scriptEl) => {
		let code = lexer(scriptEl.innerText);
	});
};
