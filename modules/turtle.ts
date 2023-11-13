import { HTMLElementScope } from "../src/defaultModules/web.js";
import { error } from "../src/error.js";
import {
	FCallableAny,
	FNode,
	FNodeAny,
	FNodeBlock
} from "../src/interfaces.js";
import { Scope } from "../src/scope.js";

const degToRad = (degrees: number) => Math.PI * (degrees / 180);
const radToDeg = (radians: number) => 180 * (radians / Math.PI);

export class TurtleScope extends Scope implements FNodeBlock {
	canvas: CanvasRenderingContext2D;
	type: "Block" = "Block";
	subType: "TurtleScope" = "TurtleScope";
	body: FNode[] = [];

	// NEED THIS TO SAVE OBJECT PROPERLY.
	scope?: Scope = this;
	returnSelf = true;

	x: number = 0;
	y: number = 0;
	w: number = 0;
	h: number = 0;
	angle: number = 0;

	drawMode: "none" | "fill" | "stroke" = "none";
	strokeSize: number = 1;
	lineCap: CanvasLineCap = "square";
	lineJoin: CanvasLineJoin = "miter";
	strokeColor: string = "black";
	fillColor: string = "black";
	path: Path2D;

	constructor(canvas: HTMLCanvasElement, parent: Scope = null) {
		super(parent);

		this.canvas = canvas.getContext("2d");
		this.w = canvas.width;
		this.h = canvas.height;
		this.applyFunctions();
	}

	private applyFunctions() {
		const self = this;
		const ctx: CanvasRenderingContext2D = this.canvas;
		const set = (
			name: string | string[],
			run: (...params: any[]) => void | FNodeAny
		) => {
			if (Array.isArray(name))
				name.forEach((id) => this.localFunctions.set(id, { type: "js", run }));
			else this.localFunctions.set(name, { type: "js", run });
		};

		set("clear", () => {
			ctx.clearRect(0, 0, self.w, self.h);
		});

		set("x", () => {
			return { type: "NumberLiteral", value: self.x };
		});
		set("y", () => {
			return { type: "NumberLiteral", value: self.y };
		});
		set("angle", () => {
			return { type: "NumberLiteral", value: radToDeg(self.angle) };
		});

		set("goto", (x, y, data) => {
			if (x?.type !== "NumberLiteral")
				error(`Expected a number, instead got a ${x.type}.`, "Type");
			if (y?.type !== "NumberLiteral")
				error(`Expected a number, instead got a ${y.type}.`, "Type");

			self.goto(x.value, y.value);
		});
		set("forward", (distance, data) => {
			if (distance?.type !== "NumberLiteral")
				error(`Expected a number, instead got a ${distance.type}.`, "Type");

			self.forward(distance.value);
		});

		set(["right", "turn"], (turn, data) => {
			if (turn?.type !== "NumberLiteral")
				error(`Expected a number, instead got a ${turn.type}.`, "Type");

			self.angle += degToRad(turn.value);
		});
		set("left", (turn, data) => {
			if (turn?.type !== "NumberLiteral")
				error(`Expected a number, instead got a ${turn.type}.`, "Type");

			self.angle -= degToRad(turn.value);
		});
		set("angle", (turn, data) => {
			if (turn?.type !== "NumberLiteral")
				error(`Expected a number, instead got a ${turn.type}.`, "Type");

			self.angle = degToRad(turn.value);
		});

		set("size", (size, data) => {
			if (size?.type !== "NumberLiteral")
				error(`Expected a number, instead got a ${size.type}.`, "Type");

			self.strokeSize = size.value;
		});
		set("cap", (cap, data) => {
			if (cap?.type !== "StringLiteral")
				error(`Expected a number, instead got a ${cap.type}.`, "Type");
			if (!["round", "square", "butt"].includes(cap.value))
				error(
					`Expected either 'butt', 'round', or 'square'. Instead got "${cap.value}".`,
					"Type"
				);

			self.lineCap = cap.value;
		});
		set("join", (join, data) => {
			if (join?.type !== "StringLiteral")
				error(`Expected a number, instead got a ${join.type}.`, "Type");
			if (!["round", "bevel", "miter"].includes(join.value))
				error(
					`Expected either 'miter', 'round', or 'bevel'. Instead got "${join.value}".`,
					"Type"
				);

			self.lineJoin = join.value;
		});
		set("color", (stroke, fill, data = null) => {
			if (stroke?.type !== "StringLiteral")
				error(
					`Expected a string for the stroke color, instead got a ${stroke.type}.`,
					"Type"
				);
			if (fill?.type != null && fill.type !== "StringLiteral")
				error(
					`Expected a string for the fill color, instead got a ${fill.type}.`,
					"Type"
				);

			self.strokeColor = stroke.value;
			if (data != null) self.fillColor = fill.value;
		});

		set("start", (pathType, data) => {
			if (pathType?.type !== "StringLiteral")
				error(`Expected a string, instead got a ${pathType.type}.`, "Type");
			if (!["none", "fill", "stroke"].includes(pathType.value))
				error(
					`Expected either 'none', 'fill', or 'stroke'. Instead got "${pathType.value}".`,
					"Type"
				);

			self.start(pathType.value);
		});
		set("end", () => self.end());

		set("image", (element) => {
			if (
				element?.subType !== "HTMLElementScope" ||
				element?.htmlEl?.constructor !== HTMLImageElement
			)
				error(
					`Expected an <img> element. Instead, got a ${element.type}`,
					"Type"
				);
			self.image(element.htmlEl);
		});
	}

	goto(x: number, y: number): void {
		this.x = x;
		this.y = y;

		if (this.path != null)
			if (this.drawMode === "none") {
				this.path.moveTo(x, y);
			} else {
				this.path.lineTo(x, y);
			}
	}

	forward(distance: number): void {
		this.goto(
			this.x + Math.sin(this.angle) * distance,
			this.y + Math.cos(this.angle) * distance
		);
	}

	image(img: CanvasImageSource) {
		this.canvas.drawImage(img, this.x, this.y);
	}

	start(drawMode: "none" | "fill" | "stroke") {
		this.drawMode = drawMode;
		this.path = new Path2D();
		this.path.moveTo(this.x, this.y);
	}

	end() {
		[
			this.canvas.fillStyle,
			this.canvas.strokeStyle,
			this.canvas.lineWidth,
			this.canvas.lineCap,
			this.canvas.lineJoin
		] = [
			this.fillColor,
			this.strokeColor,
			this.strokeSize,
			this.lineCap,
			this.lineJoin
		];

		if (this.drawMode === "stroke") this.canvas.stroke(this.path);
		else if (this.drawMode === "fill") {
			this.canvas.fill(this.path);
			this.canvas.stroke(this.path);
		}
	}
}

const scope = new Scope();

scope.localFunctions.set("new", {
	type: "js",
	run(canvas: HTMLElementScope, data) {
		if (
			canvas?.subType !== "HTMLElementScope" ||
			canvas.htmlEl.constructor !== HTMLCanvasElement
		) {
			error(`To create a new turtle, I need a <canvas> element.`, "Turtle");
			return;
		}

		let turtle = new TurtleScope(canvas.htmlEl);
		return turtle;
	}
});

export default scope;