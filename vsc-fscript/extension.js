const vscode = require("vscode");
const fs = require("fs-extra");
const path = require("path");
const dotenv = require("dotenv");

function activate(context) {
	context.subscriptions.push(
		vscode.commands.registerCommand("fscript.packageENV", async (uri) => {
			let ENVPath;
			if (!uri) {
				let directory = null;
				if (vscode.workspace.workspaceFolders.length > 1) {
					vscode.window.showWorkspaceFolderPick().then((workspace) => {
						directory = workspace.uri.fsPath;
					});
				} else directory = vscode.workspace.workspaceFolders[0].uri.fsPath;
				let files = await fs.readdir(directory);
				let foundPath = files.filter(
					(f) => path.basename(f) === "fscript.env"
				)[0];

				if (foundPath) {
					ENVPath = path.join(directory, foundPath);
				} else {
					vscode.window.showErrorMessage(
						"Could not find 'fscript.env' file in selected directory."
					);
				}
			} else ENVPath = uri.fsPath;

			if (ENVPath === undefined) return;

			if (path.extname(ENVPath) === ".env") {
				fs.readFile(ENVPath, "utf8").then((data) => {
					const env = Object.fromEntries(
						Object.entries(dotenv.parse(data)).map(([key, value]) => [
							key.toLowerCase(),
							value
						])
					);
					const { version } = env;
					delete env.version;
					delete env.export;

					const json = {
						name: path.dirname(ENVPath),
						version,
						env,
						files: []
					};

					fs.readdir(path.dirname(ENVPath)).then((files) => {
						json.files = files
							.map((v) =>
								path.extname(v) === ".func"
									? Buffer.from(
											fs.readFileSync(path.join(path.dirname(ENVPath), v))
									  ).toString("base64")
									: null
							)
							.filter((v) => v != null);

						vscode.window
							.showInputBox({
								placeHolder: "Path to package file (.fpack)"
							})
							.then((packagePath) => {
								console.log(
									path.join(
										vscode.workspace.workspaceFolders[0].uri.fsPath,
										packagePath
									)
								);
								fs.writeFile(
									path.join(
										vscode.workspace.workspaceFolders[0].uri.fsPath,
										packagePath
									),
									JSON.stringify(json)
								);
							});
					});
				});
			}
		})
	);
}

function deactivate() {}

module.exports = { activate, deactivate };
