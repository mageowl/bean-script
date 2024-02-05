import { argv } from "node:process";
import { readFile } from "node:fs/promises";
import { executer } from "./executer.js";
import { lexer } from "./lexer.js";
import { parser } from "./parser.js";
throw new Error("FScript node is currently not working.");
const target = argv[2];
readFile(target, "utf-8").then((text) => {
    executer(parser(lexer(text)), {});
});
