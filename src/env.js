import { parse } from "../dotenv/parse.js";

const parsedENVs = {};
const defaultENV = {
	process: "detect"
};

/**
 * Gets ENV data from file path.
 *
 * @export
 * @param {string} path
 */
export async function getENVData(path) {
	let dirname = path.split("/").slice(0, -1).join("/") + "/";
	if (parsedENVs[dirname]) {
		return parsedENVs[dirname];
	} else {
		const ENV = (
			await fetch(dirname + "fscript.env").catch(() => {
				console.warn(
					`Could not find a 'fscript.env' file in parent directory of ${path}. Defaulting enviroment varibles...`
				);
			})
		)?.text();

		if (!ENV) return defaultENV;

		const data = {
			...defaultENV,
			...Object.fromEntries(
				Object.entries(parse(await ENV)).map(([k, v]) => [k.toLowerCase(), v])
			)
		};

		parsedENVs[dirname] = data;
		return data;
	}
}
