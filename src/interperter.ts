import { executer } from "./executer.js";
import { lexer } from "./lexer.js";
import { parser } from "./parser.js";

export default function run(code: string) {
	return executer(parser(lexer(code)));
}
