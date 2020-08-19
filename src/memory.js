function generateSlotID(usedIDs) {
	const binary = (n) => n.toString(2);
	const decimal = (n) => parseInt(n, 2);

	return binary(
		Math.max(...(usedIDs.length ? usedIDs.map(decimal) : [-1])) + 1,
		2
	).padStart(4, "0");
}

export class Memory {
	#slotLookup = {};
	#memory = {};

	getID(slotName) {
		let id = this.#slotLookup[slotName];

		if (!id) {
			let slot = new Slot(generateSlotID(Object.keys(this.#slotLookup)));
			id = this.#slotLookup[slotName] = slot.id;
			this.#memory[slot.id] = slot;
		}

		return id;
	}

	get(slotID) {
		return this.#memory[slotID];
	}

	set(slotID, value) {
		this.#memory[slotID].set(value);
	}

	free(slotID) {
		delete this.#memory[slotID];
	}
}

class Slot {
	#value;
	id;
	constructor(id) {
		this.id = id;
	}

	get() {
		return this.#value;
	}

	set(value) {
		this.#value = value;
	}
}
