import { executer } from "./executer.js";
import { FCallData } from "./interfaces.js";
import { lexer } from "./lexer.js";
import { parser } from "./parser.js";

export default function run(code: string, options: FCallData) {
	return executer(parser(lexer(code)), options);
}
