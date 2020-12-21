import { Scope } from "./scope.js";

export const isWeb = window != null;

export const dom = new Scope();
dom.localFunctions.set("useElementAsConsole", {
	type: "js",
	run() {
		console.log("bingo");
	}
});
