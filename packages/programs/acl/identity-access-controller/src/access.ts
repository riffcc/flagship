import { field, option, serialize, variant, vec } from "@dao-xyz/borsh";
import { toBase64 } from "@peerbit/crypto";
import { AccessCondition } from "./condition.js";

export enum AccessType {
	Any = 0,
	Read = 1,
	Write = 2,
}

export enum UserRole {
	ADMIN = "ADMIN",
	WRITER = "WRITER",
	GUEST = "GUEST", // Or NONE, or undefined if a guest is someone with no specific Access entry
}

@variant(0)
export class AccessData {}

export const ACCESS_TYPE_PROPERTY = "accessTypes";
export const ROLE_PROPERTY = "role"; // For consistency if we need to query by role

@variant(0)
export class Access extends AccessData {
	@field({ type: option("string") })
	id: string;

	// Keep accessTypes for now, it might be useful for finer-grained permissions
	// beyond roles, or can be removed if roles are sufficient.
	@field({ type: vec("u16") })
	[ACCESS_TYPE_PROPERTY]: AccessType[];

	@field({ type: AccessCondition })
	accessCondition: AccessCondition<any>;

	// New role field
	@field({ type: "string" }) // Storing enum as string
	[ROLE_PROPERTY]: UserRole;

	constructor(options?: {
		accessTypes: AccessType[];
		accessCondition: AccessCondition<any>;
		role: UserRole; // Add role to constructor
	}) {
		super();
		if (options) {
			this.accessTypes = options.accessTypes;
			this.accessCondition = options.accessCondition;
			this.role = options.role; // Assign role
			this.initialize();
		}
	}

	calculateId(): string {
		if (!this.accessTypes || !this.accessCondition || !this.role) {
			throw new Error("Not initialized");
		}
		const a = new Access();
		a.accessCondition = this.accessCondition;
		a.accessTypes = this.accessTypes;
		a.role = this.role; // Include role in ID calculation
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
