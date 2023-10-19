import * as readline from "node:readline/promises";
import { stdin, stdout } from "node:process";

import { Scope } from "../scope.js";
import { execute } from "../executer.js";

const scope = new Scope();
const rl = readline.createInterface(stdin, stdout);
rl.pause();

scope.localFunctions.set("read", {
	type: "js",
	run(data, yieldFunction) {
		rl.resume();
		rl.once("line", (input) => {
			rl.pause();
			execute(yieldFunction, {
				...data,
				parameters: [{ type: "StringLiteral", value: input }]
			});
		});
	}
});

export default scope;
