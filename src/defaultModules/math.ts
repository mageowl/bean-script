import { error } from "../error.js";
import { Scope } from "../scope.js";

const scope = new Scope();

scope.localFunctions.set("abs", {
	type: "js",
	run(number) {
		if (number.type !== "NumberLiteral") {
			error(
				`I need a number to get the absolute value. I got a ${number.type}.`,
				"Type"
			);
		}

		return { type: "NumberLiteral", value: Math.abs(number.value) };
	}
});

scope.localFunctions.set("sin", {
	type: "js",
	run(number) {
		if (number.type !== "NumberLiteral") {
			error(`I need a number to get the sine. I got a ${number.type}.`, "Type");
		}

		return { type: "NumberLiteral", value: Math.sin(number.value) };
	}
});

scope.localFunctions.set("cos", {
	type: "js",
	run(number) {
		if (number.type !== "NumberLiteral") {
			error(
				`I need a number to get the cosine. I got a ${number.type}.`,
				"Type"
			);
		}

		return { type: "NumberLiteral", value: Math.cos(number.value) };
	}
});

scope.localFunctions.set("tan", {
	type: "js",
	run(number) {
		if (number.type !== "NumberLiteral") {
			error(
				`I need a number to get the tangent. I got a ${number.type}.`,
				"Type"
			);
		}

		return { type: "NumberLiteral", value: Math.tan(number.value) };
	}
});

scope.localFunctions.set("atan", {
	type: "js",
	run(number) {
		if (number.type !== "NumberLiteral") {
			error(
				`I need a number to get the arc tangent. I got a ${number.type}.`,
				"Type"
			);
		}

		return { type: "NumberLiteral", value: Math.atan(number.value) };
	}
});

scope.localFunctions.set("sqrt", {
	type: "js",
	run(number) {
		if (number.type !== "NumberLiteral") {
			error(
				`I need a number to get the square root. I got a ${number.type}.`,
				"Type"
			);
		}

		return { type: "NumberLiteral", value: Math.sqrt(number.value) };
	}
});

scope.localFunctions.set("round", {
	type: "js",
	run(number, snap) {
		if (typeof snap?.value !== "number") {
			return { type: "NumberLiteral", value: Math.round(number?.value) };
		}
		return {
			type: "NumberLiteral",
			value: Math.round(number?.value * snap?.value) / snap?.value
		};
	}
});

export default scope;
