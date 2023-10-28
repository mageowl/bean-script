export async function loadModules() {
    const moduleDefinitions = await fetch("./fmodules.json").then((d) => d.json());
    let modules = {};
    for (let [key, value] of Object.entries(moduleDefinitions)) {
        let moduleSource = value;
        switch (moduleSource?.type) {
            // prettier-ignore
            case "file/javascript":
                {
                    const scope = (await import(location.href + moduleSource.path))?.default;
                    modules[key] = scope;
                }
                break;
        }
    }
    return modules;
}
