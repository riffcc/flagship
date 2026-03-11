import { ref, computed } from 'vue'
import * as ed25519 from '@noble/ed25519'

/**
 * Identity management using REAL ed25519 cryptography
 * Uses @noble/ed25519 for proper cryptographic operations
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

// Generate a random 32-byte seed for ed25519
function generateSeed(): Uint8Array {
  return crypto.getRandomValues(new Uint8Array(32))
}

class Identity {
  private seed: Uint8Array | null = null
  private privateKey: Uint8Array | null = null
  private publicKey: Uint8Array | null = null
  private _publicKeyString = ref<string>('')
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

    // Generate proper ed25519 keypair from seed
    // The seed IS the private key for ed25519
    this.privateKey = this.seed

    // Derive public key from private key using @noble/ed25519
    this.publicKey = await ed25519.getPublicKeyAsync(this.privateKey)

    // Format public key as ed25519p/{hex}
    this._publicKeyString.value = `ed25519p/${bytesToHex(this.publicKey)}`
    this._isInitialized.value = true

    console.log('[Identity] Initialized with public key:', this._publicKeyString.value)
  }

  // Sign a message using REAL ed25519 signatures
  async sign(message: string): Promise<string> {
    if (!this.privateKey) {
      throw new Error('Identity not initialized')
    }

    // Convert message to bytes
    const messageBytes = new TextEncoder().encode(message)

    // Sign using @noble/ed25519 (proper ed25519 signature)
    const signature = await ed25519.signAsync(messageBytes, this.privateKey)

    // Return signature as hex
    return bytesToHex(signature)
  }

  // Verify a signature (useful for testing)
  async verify(message: string, signature: string, publicKey?: string): Promise<boolean> {
    const messageBytes = new TextEncoder().encode(message)
    const signatureBytes = hexToBytes(signature)

    // Use provided public key or our own
    let pubKeyBytes: Uint8Array
    if (publicKey) {
      // Strip ed25519p/ prefix if present
      const pubKeyHex = publicKey.startsWith('ed25519p/')
        ? publicKey.slice(9)
        : publicKey
      pubKeyBytes = hexToBytes(pubKeyHex)
    } else if (this.publicKey) {
      pubKeyBytes = this.publicKey
    } else {
      throw new Error('No public key available for verification')
    }

    return await ed25519.verifyAsync(signatureBytes, messageBytes, pubKeyBytes)
  }

  get publicKeyComputed() {
    return computed(() => this._publicKeyString.value)
  }

  get isInitialized() {
    return computed(() => this._isInitialized.value)
  }

  // Clear identity (for testing or logout)
  clearIdentity() {
    localStorage.removeItem(STORAGE_KEY)
    this.seed = null
    this.privateKey = null
    this.publicKey = null
    this._publicKeyString.value = ''
    this._isInitialized.value = false
    console.log('[Identity] Identity cleared')
  }

  // Export seed for backup (show as hex)
  exportSeed(): string | null {
    if (!this.seed) return null
    return bytesToHex(this.seed)
  }

  // Import seed from backup
  async importSeed(seedHex: string) {
    try {
      this.seed = hexToBytes(seedHex)
      localStorage.setItem(STORAGE_KEY, seedHex)

      // Seed is the private key for ed25519
      this.privateKey = this.seed
      this.publicKey = await ed25519.getPublicKeyAsync(this.privateKey)
      this._publicKeyString.value = `ed25519p/${bytesToHex(this.publicKey)}`
      this._isInitialized.value = true

      console.log('[Identity] Imported identity with public key:', this._publicKeyString.value)
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
    publicKey: identityInstance.publicKeyComputed,
    isInitialized: identityInstance.isInitialized,
    sign: (message: string) => identityInstance.sign(message),
    verify: (message: string, signature: string, publicKey?: string) =>
      identityInstance.verify(message, signature, publicKey),
    clearIdentity: () => identityInstance.clearIdentity(),
    exportSeed: () => identityInstance.exportSeed(),
    importSeed: (seed: string) => identityInstance.importSeed(seed),
  }
}
