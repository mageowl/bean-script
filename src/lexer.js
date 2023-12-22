import { operator, FTokenType } from "./enums.js";
function chunk(code) {
    let chunks = [];
    let currentChunk = "";
    let comment = false;
    let blockComment = false;
    let i = 0;
    let inString = false;
    let inMemory = false;
    const isDigit = (char) => "1234567890".includes(char) && char.length === 1;
    function split() {
        chunks.push(currentChunk);
        currentChunk = "";
    }
    for (const char of code) {
        if ((char === " " || char === "	" || char === "\n") && !inString) {
            split();
            if (char === "\n") {
                comment = false;
            }
        }
        else if (char === "*") {
            if (code[i + 1] === "*" && code[i - 1] !== "*") {
                blockComment = !blockComment;
            }
            else if (code[i - 1] !== "*")
                comment = true;
        }
        else if (comment || blockComment) {
            // nothing.
        }
        else if (char === '"') {
            if (!inString) {
                split();
                currentChunk += char;
                inString = true;
            }
            else {
                currentChunk += char;
                inString = false;
                split();
            }
        }
        else if (char === "-" && code[i + 1] === ">" && !inString) {
            split();
            currentChunk += char;
        }
        else if (char === ">" && code[i - 1] === "-" && !inString) {
            currentChunk += char;
            split();
        }
        else if (char === "." && code[i + 1] === "." && !inString) {
            split();
            currentChunk += char;
        }
        else if (char === "." && code[i - 1] === "." && !inString) {
            currentChunk += char;
            split();
        }
        else if ((Object.values(operator)
            .flatMap((o) => (typeof o === "object" ? Object.values(o) : o))
            .includes(char) ||
            char === ";") &&
            !inString &&
            !(isDigit(code[i - 1]) && isDigit(code[i + 1])) &&
            !(char === "." && inMemory)) {
            split();
            currentChunk += char;
            split();
        }
        else if (char === "<" && !inString) {
            split();
            currentChunk += char;
            inMemory = true;
        }
        else if (char === ">" && !inString) {
            inMemory = false;
            currentChunk += char;
            split();
        }
        else
            currentChunk += char;
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
            case /^-?[\d.]*\d$/.test(chunk):
                tokens.push({ type: FTokenType.NUMBER, value: parseFloat(chunk) });
                break;
            case /^"[^"]*"$/.test(chunk):
                tokens.push({ type: FTokenType.STRING, value: chunk.slice(1, -1) });
                break;
            case /^true|false$/.test(chunk):
                tokens.push({ type: FTokenType.BOOLEAN, value: chunk === "true" });
                break;
            case /^null$/.test(chunk):
                tokens.push({ type: FTokenType.NULL });
                break;
            case /^<[^>]+>$/.test(chunk):
                tokens.push({ type: FTokenType.MEMORY, value: chunk.slice(1, -1) });
                break;
            case Object.values(operator)
                .flatMap((o) => (typeof o === "object" ? Object.values(o) : o))
                .includes(chunk):
                tokens.push({ type: FTokenType.OPERATOR, value: chunk.trim() });
                break;
            case chunk === ";":
                tokens.push({ type: FTokenType.NEWLINE });
                break;
            default:
                tokens.push({ type: FTokenType.VALUE, value: chunk.trim() });
        }
    });
    return tokens;
}
