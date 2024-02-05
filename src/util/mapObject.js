export default function mapObject(object, callback) {
    return Object.fromEntries(Object.entries(object).map(callback));
}
