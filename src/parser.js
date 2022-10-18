import { FTokenType, operator } from "./enums.js";
export function parser(tokens) {
    let i = 0;
    let ast = {
        type: "Program",
        body: []
    };
    function next() {
        return tokens[i++];
    }
    function peek() {
        return tokens[i];
    }
    function parse(token) {
        if (token.type === FTokenType.VALUE) {
            let node = {
                type: "FunctionCall",
                name: token.value,
                parameters: [],
                yieldFunction: null
            };
            if (peek() &&
                peek().type === "operator" &&
                peek().value === operator.PAREN.START) {
                next();
                while (peek() &&
                    !(peek().type === "operator" && peek().value === operator.PAREN.END)) {
                    let parameter = { type: "ParameterBlock", body: [] };
                    if (peek().type === "operator" && peek().value === operator.COMMA)
                        next();
                    while (peek() &&
                        !(peek().type === "operator" &&
                            (peek().value === operator.COMMA ||
                                peek().value === operator.PAREN.END))) {
                        let paramNode = parse(next());
                        if (paramNode)
                            parameter.body.push(paramNode);
                    }
                    parameter.body.filter((x) => x != undefined);
                    node.parameters.push(parameter);
                }
                next();
            }
            if (peek() &&
                peek().type === "operator" &&
                peek().value === operator.ARROW) {
                next();
                node.yieldFunction = parse(next());
            }
            return node;
        }
        else if (token.type === FTokenType.STRING) {
            return { type: "StringLiteral", value: token.value };
        }
        else if (token.type === FTokenType.MEMORY &&
            token.value.startsWith("!")) {
            return { type: "NeedOperator", value: token.value.slice(1) };
        }
        else if (token.type === FTokenType.MEMORY) {
            return { type: "MemoryLiteral", value: token.value };
        }
        else if (token.type === FTokenType.NUMBER) {
            return { type: "NumberLiteral", value: token.value };
        }
        else if (token.type === FTokenType.BOOLEAN) {
            return { type: "BooleanLiteral", value: token.value };
        }
        else if (token.type === FTokenType.NULL) {
            return { type: "NullLiteral" };
        }
        else if (token.type === FTokenType.OPERATOR) {
            switch (token.value) {
                case operator.BRACE.START:
                    let body = [];
                    while (peek() &&
                        !(peek().type === "operator" && peek().value === operator.BRACE.END)) {
                        body.push(parse(next()));
                    }
                    next();
                    body.filter((x) => x != undefined);
                    return { type: "Block", body };
                default:
                    break;
            }
        }
        else if (token.type === FTokenType.NEWLINE) {
            return { type: "NewLine" };
        }
        return null;
    }
    do {
        ast.body.push(parse(next()));
    } while (i < tokens.length);
    return ast;
}
