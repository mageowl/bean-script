import { FCallData, FCallableAny, FNodeAny, FNodeBlock } from "./interfaces";

export default function call(
	fn: FCallableAny,
	parameters: FNodeAny[],
	data: FCallData,
	yieldFunction: FNodeAny,
	execute: Function
): void | FNodeAny {
	if (fn.type === "js") {
		return fn.run(
			...parameters.map((node) => execute(node, data)),
			data,
			yieldFunction
		);
	} else if (fn.type === "custom") {
		let params = (parameters.length ? parameters : data.parameters) ?? [];
		return execute(fn.run, {
			scope: fn.scope,
			parameters: params.map((node) => execute(node, data)),
			yieldFunction,
			yieldScope: data
		});
	}
}
