import { FTokenType } from "./enums";
import { Scope, Slot } from "./scope";

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
	yieldFunction: FNodeAny | null;
}

export interface FNodeValue extends FNode {
	value: any;
}

export interface FNodeMemory extends FNode {
	type: "MemoryLiteral";
	slot: Slot;
}

export interface FNodeBlock extends FNode {
	type: "Program" | "Block" | "ParameterBlock";
	body: FNodeAny[];
	scope?: Scope;
}

export type FNodeAny =
	| FNode
	| FNodeBlock
	| FNodeFunctionCall
	| FNodeValue
	| FNodeMemory
	| Scope
	| null;

export interface FCallData {
	scope?: Scope;
	parameters?: FNodeBlock[];
	yieldFunction?: FNode | null;
	returnScope?: Scope;
}

export interface FCallable {
	type: "js" | "custom";
}

export interface FJSCallable extends FCallable {
	type: "js";
	run(...params): FNodeAny | void;
}

export interface FUserCallable extends FCallable {
	type: "custom";
	run: FNodeAny;
	scope?: Scope;
}

export type FCallableAny = FJSCallable | FUserCallable;

export interface FModuleSource {
	type: "file/javascript" | "file/f-script" | "github";
	path: string;
}
