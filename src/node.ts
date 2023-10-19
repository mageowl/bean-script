import { argv } from "node:process";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { readFile } from "node:fs/promises";

import * as defaultModules from "./defaultModules/main.js";
import { executer } from "./executer.js";
import { lexer } from "./lexer.js";
import { parser } from "./parser.js";
import server from "./defaultModules/server.js";

const __dirname = dirname(fileURLToPath(import.meta.url));
const target = argv[2];
readFile(target, "utf-8").then((text) => {
	executer(parser(lexer(text)), {
		...defaultModules,
		server
	});
});
