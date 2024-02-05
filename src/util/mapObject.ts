export default function mapObject(object: Object, callback: ([key, value]) => [any, any]) {
	return Object.fromEntries(Object.entries(object).map(callback))
}
