import { field, variant } from "@dao-xyz/borsh";
import { type PeerId } from "@libp2p/interface";
import { PublicSignKey, getPublicKeyFromPeerId, sha256Sync } from "@peerbit/crypto";
import { type CanPerformOperations, Documents } from "@peerbit/document";
import { Compare, IntegerCompare, Or, SearchRequest, StringMatch, And } from "@peerbit/document";
import { Program } from "@peerbit/program";
import { type ReplicationOptions } from "@peerbit/shared-log";
import {
	IdentityGraph,
	TrustedNetwork,
	createIdentityGraphStore,
	getFromByTo,
	getPathGenerator,
} from "@peerbit/trusted-network";
import { concat } from "uint8arrays";
import { ACCESS_TYPE_PROPERTY, Access, AccessType, UserRole, ROLE_PROPERTY } from "./access.js";
import { PublicKeyAccessCondition, AnyAccessCondition } from "./condition.js";

@variant("identity_acl")
export class IdentityAccessController extends Program {
	@field({ type: Documents })
	access: Documents<Access>;

	@field({ type: IdentityGraph })
	identityGraphController: IdentityGraph;

	@field({ type: TrustedNetwork })
	trustedNetwork: TrustedNetwork;

	// Helper to ensure rootTrust is PublicSignKey if needed for direct use
	get rootTrustKey(): PublicSignKey | undefined {
		if (this.trustedNetwork.rootTrust instanceof PublicSignKey) {
			return this.trustedNetwork.rootTrust;
		} else if (this.trustedNetwork.rootTrust) { // PeerId case
			try {
				return getPublicKeyFromPeerId(this.trustedNetwork.rootTrust);
			} catch (e) {
				console.error("Failed to get PublicKey from rootTrust PeerId:", e);
				return undefined;
			}
		}
		return undefined;
	}

	constructor(opts: {
		id?: Uint8Array;
		rootTrust: PublicSignKey | PeerId; // rootTrust will be the initial admin
		trustedNetwork?: TrustedNetwork;
	}) {
		super();
		if (!opts.trustedNetwork && !opts.rootTrust) {
			throw new Error("Expecting either TrustedNetwork or rootTrust");
		}
		this.access = new Documents({
			id: opts.id && sha256Sync(concat([opts.id, new Uint8Array([0])])),
		});

		this.trustedNetwork = opts.trustedNetwork
			? opts.trustedNetwork
			: new TrustedNetwork({
				id: opts.id && sha256Sync(concat([opts.id, new Uint8Array([1])])),
				rootTrust: opts.rootTrust,
			});
		this.identityGraphController = new IdentityGraph({
			relationGraph: createIdentityGraphStore(
				opts.id && sha256Sync(concat([opts.id, new Uint8Array([2])])),
			),
		});
	}

	async getRole(publicKey: PublicSignKey): Promise<UserRole | undefined> {
		// We need to find an Access entry where its AccessCondition is a PublicKeyAccessCondition
		// that matches the given publicKey.
		// This is a bit tricky with current search capabilities if PublicKey is nested.
		// A more direct way would be to add publicKey as a top-level indexed field in Access if this is slow.
		// For now, iterate and check.
		const allAccessEntries = await this.access.index.search(new SearchRequest({ query: [] })); // Fetch all
		for (const entry of allAccessEntries) {
			if (entry instanceof Access) {
				if (entry.accessCondition instanceof PublicKeyAccessCondition) {
					if (entry.accessCondition.key.equals(publicKey)) {
						return entry.role; // Return the role of the first matching entry
					}
				}
			}
		}
		return undefined;
	}

	async setRole(targetPublicKey: PublicSignKey, role: UserRole, actorPublicKey: PublicSignKey): Promise<void> {
		const actorRole = await this.getRole(actorPublicKey);
		if (actorRole !== UserRole.ADMIN) {
			throw new Error(`Actor ${actorPublicKey.toString()} with role ${actorRole} is not authorized to set roles. Admin role required.`);
		}

		// Remove existing role for this targetPublicKey if any
		// This is to ensure a user has only one role assigned directly via PublicKeyAccessCondition
		const existingEntries = await this.access.index.search(new SearchRequest({ query: [] }));
		for (const entry of existingEntries) {
			if (entry instanceof Access && entry.accessCondition instanceof PublicKeyAccessCondition && entry.accessCondition.key.equals(targetPublicKey)) {
				await this.access.del(entry.id);
			}
		}
		
		let accessTypesForRole: AccessType[];
		switch (role) {
			case UserRole.ADMIN:
			case UserRole.WRITER:
				accessTypesForRole = [AccessType.Read, AccessType.Write];
				break;
			case UserRole.GUEST:
				accessTypesForRole = [AccessType.Read];
				break;
			default:
				accessTypesForRole = [];
		}

		const newAccessEntry = new Access({
			accessCondition: new PublicKeyAccessCondition({ key: targetPublicKey }),
			role: role,
			accessTypes: accessTypesForRole
		});
		// newAccessEntry.initialize(); // ID is calculated in constructor if options are passed
		await this.access.put(newAccessEntry);
	}

	async hasPermission(publicKey: PublicSignKey, requiredRoles: UserRole[]): Promise<boolean> {
		const userRole = await this.getRole(publicKey);
		return userRole ? requiredRoles.includes(userRole) : false;
	}

	async canRead(_obj: any, s: PublicSignKey | undefined): Promise<boolean> {
		if (!s) {
			return false;
		}
		if (await this.trustedNetwork.isTrusted(s)) {
			return true;
		}
		// Check direct role-based permission
		if (await this.hasPermission(s, [UserRole.ADMIN, UserRole.WRITER, UserRole.GUEST])) {
			return true;
		}
		
		// Fallback to checking generic Read/Any access types with conditions,
		// potentially for AnyAccessCondition or other non-role specific rules.
		const canReadCheck = async (key: PublicSignKey) => {
			const accessReadOrAny = await this.access.index.search(
				new SearchRequest({
					query: [
						new Or([
							new IntegerCompare({ key: ACCESS_TYPE_PROPERTY, compare: Compare.Equal, value: AccessType.Any }),
							new IntegerCompare({ key: ACCESS_TYPE_PROPERTY, compare: Compare.Equal, value: AccessType.Read }),
						]),
					],
				})
			);
			for (const access of accessReadOrAny) {
				if (access instanceof Access) {
					if (access.accessTypes.find((x) => x === AccessType.Any || x === AccessType.Read) !== undefined) {
						if (await access.accessCondition.allowed(key)) {
							// If this rule is not role-specific (e.g. AnyAccessCondition), it should grant access.
							// If it IS role-specific (PublicKeyAccessCondition), hasPermission would have caught it.
							// This mainly helps with global read rules like "Any logged in user can read" if condition allows.
							if (!(access.accessCondition instanceof PublicKeyAccessCondition)) {
								return true;
							}
						}
					}
				}
			}
			return false;
		};

		if (await canReadCheck(s)) {
			return true;
		}

		// Check trusted graph (indirect trust)
		for await (const trustedByKey of getPathGenerator(s, this.identityGraphController.relationGraph, getFromByTo)) {
			if (await this.hasPermission(trustedByKey.from, [UserRole.ADMIN, UserRole.WRITER, UserRole.GUEST])) return true; // Role-based for trusted
			if (await canReadCheck(trustedByKey.from)) return true; // Condition-based for trusted
		}

		return false;
	}

	async canPerform(operation: CanPerformOperations<Access>): Promise<boolean> {
		const keys = await operation.entry.getPublicKeys();
		if (keys.length === 0) {
			return false; // Or true? If no key, can it be performed? Assuming false.
		}

		const canPerformByKey = async (key: PublicSignKey): Promise<boolean> => {
			// Highest priority: is the key part of the trusted network?
			if (await this.trustedNetwork.isTrusted(key)) {
				return true;
			}

			// Second priority: Does the key have explicit ADMIN or WRITER role?
			if (await this.hasPermission(key, [UserRole.ADMIN, UserRole.WRITER])) {
				return true;
			}
			
			// Fallback: Check for generic Write/Any access types for this key (e.g. via AnyAccessCondition)
			// This is less likely to be used if roles are primary, but kept for compatibility.
			const accessWriteOrAny = await this.access.index.search(
				new SearchRequest({
					query: [
						new Or([
							new IntegerCompare({ key: ACCESS_TYPE_PROPERTY, compare: Compare.Equal, value: AccessType.Any }),
							new IntegerCompare({ key: ACCESS_TYPE_PROPERTY, compare: Compare.Equal, value: AccessType.Write }),
						]),
					],
				})
			);

			for (const access of accessWriteOrAny) {
				if (access instanceof Access) {
					if (access.accessTypes.find((x) => x === AccessType.Any || x === AccessType.Write) !== undefined) {
						if (await access.accessCondition.allowed(key)) {
							// If this rule is not role-specific (e.g. AnyAccessCondition), it should grant access.
							// If it IS role-specific (PublicKeyAccessCondition), hasPermission would have caught it.
							if (!(access.accessCondition instanceof PublicKeyAccessCondition)) {
								return true;
							}
						}
					}
				}
			}

			// Check indirect trust via identity graph (less common for write, but possible)
			for await (const trustedByKey of getPathGenerator(key, this.identityGraphController.relationGraph, getFromByTo)) {
				if (await this.hasPermission(trustedByKey.from, [UserRole.ADMIN, UserRole.WRITER])) return true; // Role-based for trusted
				// Add check for generic write conditions for trustedByKey.from if needed
			}
			return false;
		};

		for (const key of keys) {
			if (await canPerformByKey(key)) {
				return true; // If any key can perform, operation is allowed
			}
		}
		return false;
	}

	async open(properties?: { replicate?: ReplicationOptions }) {
		await this.identityGraphController.open({
			replicate: properties?.replicate || { factor: 1 },
			// canRead for identityGraph might need to be permissive or use its own ACL
		});

		await this.access.open({
			replicate: properties?.replicate || { factor: 1 },
			type: Access,
			canPerform: async (op) => {
				// Who can write to the ACL DB itself? ONLY ADMINS or the rootTrust initially.
				const keys = await op.entry.getPublicKeys();
				for (const key of keys) {
					if (this.rootTrustKey?.equals(key)) return true; // Root trust can always write to ACL initially
					if (await this.hasPermission(key, [UserRole.ADMIN])) return true;
				}
				return false;
			},
			index: {
				// canRead for ACL index. Who can see the ACL rules?
				// For now, let's say anyone who can read the program in general.
				// This could be more restrictive.
				canRead: this.canRead.bind(this),
			},
		});
		await this.trustedNetwork.open(properties);

		// Bootstrap initial admin
		const rootAdminKey = this.rootTrustKey;
		if (rootAdminKey) {
			const adminRole = await this.getRole(rootAdminKey);
			if (!adminRole) { // If rootTrust does not have a role yet
				console.log(`Bootstrapping initial admin: ${rootAdminKey.toString()}`);
				// We need to bypass the regular setRole's actor check for this initial setup
				const bootstrapAdminAccess = new Access({
					accessCondition: new PublicKeyAccessCondition({ key: rootAdminKey }),
					role: UserRole.ADMIN,
					accessTypes: [AccessType.Read, AccessType.Write]
				});
				// bootstrapAdminAccess.initialize(); // ID calculated in constructor
				await this.access.put(bootstrapAdminAccess);
				console.log(`Initial admin ${rootAdminKey.toString()} bootstrapped with ADMIN role.`);
			} else {
				console.log(`Root trust ${rootAdminKey.toString()} already has role: ${adminRole}. No bootstrap needed.`);
			}
		} else {
			console.warn("No PublicSignKey found for rootTrust. Initial admin cannot be bootstrapped automatically. Ensure 'rootTrust' is a PublicSignKey or a resolvable PeerId for admin bootstrapping.");
		}
	}
}
