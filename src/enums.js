export const operator = {
    PAREN: {
        START: "(",
        END: ")"
    },
    BRACE: {
        START: "{",
        END: "}"
    },
    ARROW: "->",
    COLON: ":",
    COMMA: ",",
    ACCESS: ".",
    PARENT: ".."
};
export var FTokenType;
(function (FTokenType) {
    FTokenType["STRING"] = "lit.string";
    FTokenType["NUMBER"] = "lit.number";
    FTokenType["BOOLEAN"] = "lit.bool";
    FTokenType["MEMORY"] = "lit.memory";
    FTokenType["VALUE"] = "lit.value";
    FTokenType["NULL"] = "lit.null";
    FTokenType["OPERATOR"] = "operator";
    FTokenType["NEWLINE"] = "newline";
})(FTokenType || (FTokenType = {}));
