import { field, option, serialize, variant, vec } from "@dao-xyz/borsh";
import { toBase64 } from "@peerbit/crypto";
import { AccessCondition } from "./condition.js";

export enum AccessType {
	Any = 0,
	Read = 1,
	Write = 2,
}

@variant(0)
export class AccessData {}

export const ACCESS_TYPE_PROPERTY = "accessTypes";
@variant(0)
export class Access extends AccessData {
	@field({ type: option("string") })
	id: string;

	@field({ type: vec("u16") }) // TODO we can not use u8 because sqlite will index this as a bytearray in that case
	[ACCESS_TYPE_PROPERTY]: AccessType[];

	@field({ type: AccessCondition })
	accessCondition: AccessCondition<any>;

	constructor(options?: {
		accessTypes: AccessType[];
		accessCondition: AccessCondition<any>;
	}) {
		super();
		if (options) {
			this.accessTypes = options.accessTypes;
			this.accessCondition = options.accessCondition;
			this.initialize();
		}
	}

	calculateId(): string {
		if (!this.accessTypes || !this.accessCondition) {
			throw new Error("Not initialized");
		}
		const a = new Access();
		a.accessCondition = this.accessCondition;
		a.accessTypes = this.accessTypes;
		return toBase64(serialize(a));
	}

	initialize(): this {
		this.id = this.calculateId();
		return this;
	}

	assertId() {
		const calculatedId = this.calculateId();
		if (this.id !== calculatedId) {
			throw new Error(
				`Invalid id, got ${this.id} but expected ${calculatedId}`,
			);
		}
	}
}
