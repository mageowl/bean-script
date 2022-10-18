import { FTokenType } from "./enums";
import { Scope } from "./scope";

export interface FToken {
	type: FTokenType;
	value?: any;
}

export type FNodeType =
	| "FunctionCall"
	| "Block"
	| "Program"
	| "ParameterBlock"
	| "NeedOperator"
	| "MemoryLiteral"
	| "NewLine"
	| "StringLiteral"
	| "NeedOperator"
	| "MemoryLiteral"
	| "NumberLiteral"
	| "BooleanLiteral"
	| "NullLiteral";

export interface FNode {
	type: FNodeType;
}

export interface FNodeFunctionCall extends FNode {
	type: "FunctionCall";
	name: String;
	parameters: FNodeBlock[];
	yieldFunction: FNode | null;
}

export interface FNodeValue extends FNode {
	value: any;
}

export interface FNodeBlock extends FNode {
	type: "Program" | "Block" | "ParameterBlock";
	body: FNodeAny[];
}

export type FNodeAny =
	| FNode
	| FNodeBlock
	| FNodeFunctionCall
	| FNodeValue
	| null;

export interface FCallData {
	scope?: Scope;
	parameters?: FNodeBlock[];
	yieldFunction?: FNode | null;
	returnScope?: Scope;
}
