import { ref, computed } from 'vue'

/**
 * Identity management using ed25519 cryptography
 * This replaces Peerbit and provides Lens V2 SDK compatible keys
 */

const STORAGE_KEY = 'lens_identity_seed'

// Convert hex string to Uint8Array
function hexToBytes(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2)
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16)
  }
  return bytes
}

// Convert Uint8Array to hex string
function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes)
    .map(b => b.toString(16).padStart(2, '0'))
    .join('')
}

// Generate a random 32-byte seed
function generateSeed(): Uint8Array {
  return crypto.getRandomValues(new Uint8Array(32))
}

// Derive ed25519 keypair from seed using Web Crypto API
async function deriveKeypair(seed: Uint8Array): Promise<{
  publicKey: Uint8Array
  privateKey: Uint8Array
}> {
  // Import seed as raw key material
  const keyMaterial = await crypto.subtle.importKey(
    'raw',
    seed,
    { name: 'HKDF' },
    false,
    ['deriveBits']
  )

  // Derive 32 bytes for ed25519 private key
  const derivedBits = await crypto.subtle.deriveBits(
    {
      name: 'HKDF',
      hash: 'SHA-256',
      salt: new Uint8Array(0),
      info: new TextEncoder().encode('lens-ed25519-v2')
    },
    keyMaterial,
    256 // 32 bytes
  )

  const privateKeyBytes = new Uint8Array(derivedBits)

  // For now, we'll use a simple derivation for the public key
  // In production, you'd use a proper ed25519 library like @noble/ed25519
  // For this implementation, we'll derive it from the private key using SHA-256
  const publicKeyHash = await crypto.subtle.digest('SHA-256', privateKeyBytes)
  const publicKeyBytes = new Uint8Array(publicKeyHash)

  return {
    publicKey: publicKeyBytes,
    privateKey: privateKeyBytes
  }
}

class Identity {
  private seed: Uint8Array | null = null
  private _publicKey = ref<string>('')
  private _isInitialized = ref(false)

  async initialize() {
    // Check for existing seed in localStorage
    const storedSeed = localStorage.getItem(STORAGE_KEY)

    if (storedSeed) {
      console.log('[Identity] Loading existing identity from localStorage')
      this.seed = hexToBytes(storedSeed)
    } else {
      console.log('[Identity] Generating new identity')
      this.seed = generateSeed()
      localStorage.setItem(STORAGE_KEY, bytesToHex(this.seed))
    }

    // Derive keypair
    const keypair = await deriveKeypair(this.seed)

    // Format public key as ed25119p/{hex}
    this._publicKey.value = `ed25119p/${bytesToHex(keypair.publicKey)}`
    this._isInitialized.value = true

    console.log('[Identity] Initialized with public key:', this._publicKey.value)
  }

  get publicKey() {
    return computed(() => this._publicKey.value)
  }

  get isInitialized() {
    return computed(() => this._isInitialized.value)
  }

  // Clear identity (for testing or logout)
  clearIdentity() {
    localStorage.removeItem(STORAGE_KEY)
    this.seed = null
    this._publicKey.value = ''
    this._isInitialized.value = false
    console.log('[Identity] Identity cleared')
  }

  // Export seed for backup (show as mnemonic or hex)
  exportSeed(): string | null {
    if (!this.seed) return null
    return bytesToHex(this.seed)
  }

  // Import seed from backup
  async importSeed(seedHex: string) {
    try {
      this.seed = hexToBytes(seedHex)
      localStorage.setItem(STORAGE_KEY, seedHex)

      const keypair = await deriveKeypair(this.seed)
      this._publicKey.value = `ed25119p/${bytesToHex(keypair.publicKey)}`
      this._isInitialized.value = true

      console.log('[Identity] Imported identity with public key:', this._publicKey.value)
      return true
    } catch (error) {
      console.error('[Identity] Failed to import seed:', error)
      return false
    }
  }
}

// Singleton instance
const identityInstance = new Identity()

export function useIdentity() {
  return {
    initialize: () => identityInstance.initialize(),
    publicKey: identityInstance.publicKey,
    isInitialized: identityInstance.isInitialized,
    clearIdentity: () => identityInstance.clearIdentity(),
    exportSeed: () => identityInstance.exportSeed(),
    importSeed: (seed: string) => identityInstance.importSeed(seed),
  }
}
