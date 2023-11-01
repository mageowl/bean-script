import { executer } from "./executer.js";
import { lexer } from "./lexer.js";
import { parser } from "./parser.js";
import { isWeb } from "./process.js";
import { loadModules } from "./moduleLoader.js";
import * as defaultModules from "./defaultModules/main.js";
if (isWeb) {
    console.log("FScript Web detected. Searching for script tags.");
    window.addEventListener("load", async () => {
        let scripts = document.querySelectorAll('script[type="text/f-script"]');
        // Compile scripts
        for (let scriptEl of scripts) {
            if (!scriptEl.src) {
                const lex = lexer(scriptEl.innerText);
                const parse = parser(lex);
                executer(parse, {
                    ...defaultModules
                });
            }
            else {
                const customModules = await loadModules();
                const text = await fetch(scriptEl.src).then((d) => d.text());
                const lex = lexer(text);
                const parse = parser(lex);
                executer(parse, {
                    ...defaultModules,
                    ...customModules
                });
            }
        }
    });
}
else {
    console.log("NodeJS detected. Use src/node.js to run code on a server.");
}
export default function compile(code, options) {
    return executer(parser(lexer(code)), {
        ...defaultModules,
        ...options.modules
    });
}
