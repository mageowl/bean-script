import { executer } from "./executer.js";
import { lexer } from "./lexer.js";
import { parser } from "./parser.js";
export default function run(code, options) {
    return executer(parser(lexer(code)), options);
}
