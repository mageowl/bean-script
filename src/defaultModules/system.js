import { Scope } from "../scope.js";
import { error } from "../error.js";

const scope = new Scope();

scope.localFunctions.set("error", {
	type: "js",
	run(message) {
		console.error(message.value);
	}
});

export default scope;
