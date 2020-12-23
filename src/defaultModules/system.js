import { Scope } from "../scope.js";

const scope = new Scope();

scope.localFunctions.set("error", {
	type: "js",
	run(message) {
		console.error("[fscript] " + message.value);
	}
});

export default scope;
