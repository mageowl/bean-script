import { operator, literal } from "./enums.js";

function chunk(code) {
	let chunks = [];
	let currentChunk = "";
	let comment = false;
	let blockComment = false;
	let i = 0;

	let inString = false;

	function split() {
		chunks.push(currentChunk);
		currentChunk = "";
	}

	for (const char of code) {
		if ((char == " " || char == "	" || char == "\n") && !inString) {
			split();
			if (char == "\n") {
				comment = false;
			}
		} else if (char == "*") {
			if (code[i - 1] != "*") {
				if (code[i + 1] == "*") {
					blockComment = !blockComment;
				} else comment = true;
			}
		} else if (comment || blockComment) {
			// nothing.
		} else if (char == '"') {
			if (!inString) {
				split();
				currentChunk += char;
				inString = true;
			} else {
				currentChunk += char;
				inString = false;
				split();
			}
		} else if (char == "-" && code[i + 1] == ">" && !inString) {
			split();
			currentChunk += char;
		} else if (char == ">" && code[i - 1] == "-" && !inString) {
			currentChunk += char;
			split();
		} else if (
			(Object.values(operator)
				.flatMap((o) => (typeof o == "object" ? Object.values(o) : o))
				.includes(char) ||
				char == ";") &&
			!inString
		) {
			split();
			currentChunk += char;
			split();
		} else if (char == "<" && !inString) {
			split();
			currentChunk += char;
		} else if (char == ">" && !inString) {
			currentChunk += char;
			split();
		} else currentChunk += char;

		i++;
	}

	split();

	return chunks.map((x) => x.trim()).filter((x) => x.length);
}

export function lexer(code) {
	let chunks = chunk(code);
	let tokens = [];

	chunks.forEach((chunk) => {
		switch (true) {
			case /^[\d.]+$/g.test(chunk):
				tokens.push({ type: literal.NUMBER, value: parseInt(chunk) });
				break;

			case /^"[^"]*"$/g.test(chunk):
				tokens.push({ type: literal.STRING, value: chunk.slice(1, -1) });
				break;

			case /^true|false$/g.test(chunk):
				tokens.push({ type: literal.BOOLEAN, value: chunk == "true" });
				break;

			case /^<[^>]+>$/g.test(chunk):
				tokens.push({ type: literal.MEMORY, value: chunk.slice(1, -1) });
				break;

			case Object.values(operator)
				.flatMap((o) => (typeof o == "object" ? Object.values(o) : o))
				.includes(chunk):
				tokens.push({ type: "operator", value: chunk.trim() });
				break;

			case chunk == ";":
				tokens.push({ type: "newline" });
				break;

			default:
				tokens.push({ type: literal.VALUE, value: chunk.trim() });
		}
	});

	return tokens;
}
