/*
 THIS CODE WAS NOT MADE BY ME (@seattleowl).
 This is a excerpt from dotenv by @motdotla on GitHub, modified to use ES6 exports. (https://github.com/motdotla/dotenv/blob/master/LICENSE)

 (LICENSE: https://github.com/motdotla/dotenv/blob/master/LICENSE)
*/

const NEWLINE = "\n";
const RE_INI_KEY_VAL = /^\s*([\w.-]+)\s*=\s*(.*)?\s*$/;
const RE_NEWLINES = /\\n/g;
const NEWLINES_MATCH = /\n|\r|\r\n/;

export function parse(
	src /*: string | Buffer */,
	options /*: ?DotenvParseOptions */
) /*: DotenvParseOutput */ {
	const debug = Boolean(options && options.debug);
	const obj = {};

	// convert Buffers before splitting into lines and processing
	src
		.toString()
		.split(NEWLINES_MATCH)
		.forEach(function (line, idx) {
			// matching "KEY' and 'VAL' in 'KEY=VAL'
			const keyValueArr = line.match(RE_INI_KEY_VAL);
			// matched?
			if (keyValueArr != null) {
				const key = keyValueArr[1];
				// default undefined or missing values to empty string
				let val = keyValueArr[2] || "";
				const end = val.length - 1;
				const isDoubleQuoted = val[0] === '"' && val[end] === '"';
				const isSingleQuoted = val[0] === "'" && val[end] === "'";

				// if single or double quoted, remove quotes
				if (isSingleQuoted || isDoubleQuoted) {
					val = val.substring(1, end);

					// if double quoted, expand newlines
					if (isDoubleQuoted) {
						val = val.replace(RE_NEWLINES, NEWLINE);
					}
				} else {
					// remove surrounding whitespace
					val = val.trim();
				}

				obj[key] = val;
			} else if (debug) {
				log(
					`did not match key and value when parsing line ${idx + 1}: ${line}`
				);
			}
		});

	return obj;
}
