class FScriptError extends Error {
	/**
	 *
	 * @param {string} msg Error message.
	 * @param {string} type Type of error.
	 */
	constructor(msg, type) {
		super(msg);
		this.name = `FScript${type}Error`;
	}
}

export function error(msg, type) {
	throw new FScriptError(msg, type);
}
