export default function toFString(node) {
	if (!node) return;
	if (node?.toFString != null) return node.toFString();
	switch (node.type) {
		case "NumberLiteral":
			return node.value.toString();
		case "StringLiteral":
			return node.value;

		case "BooleanLiteral":
			return node.value.toString();

		case "MemoryLiteral":
			return `<${node.value}>`;

		case "Block":
			return "[block]";

		default:
			return "[null]";
	}
}
