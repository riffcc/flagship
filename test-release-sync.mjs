import * as ed25519 from '@noble/ed25519';

// Helper functions
function bytesToHex(bytes) {
  return Array.from(bytes)
    .map(b => b.toString(16).padStart(2, '0'))
    .join('');
}

// Generate keypair
console.log('🔐 Generating Ed25519 keypair...\n');
const privateKey = crypto.getRandomValues(new Uint8Array(32));
const publicKey = await ed25519.getPublicKeyAsync(privateKey);

const publicKeyFormatted = 'ed25119p/' + bytesToHex(publicKey);
console.log('Private Key:', bytesToHex(privateKey));
console.log('Public Key:', bytesToHex(publicKey));
console.log('Public Key (formatted):', publicKeyFormatted);

// Step 1: Authorize the public key
console.log('\n📝 Step 1: Authorizing public key as admin...');
const authResponse = await fetch('https://api.palace.riff.cc/api/v1/admin/authorize', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ publicKey: publicKeyFormatted })
});
const authResult = await authResponse.json();
console.log('Authorization result:', authResult);

// Wait for admin sync across all nodes
console.log('\n⏳ Waiting 2 seconds for admin authorization to sync across all nodes...');
await new Promise(resolve => setTimeout(resolve, 2000));

// Step 2: Create and sign release
console.log('\n📦 Step 2: Creating signed release...');
const releaseData = {
  name: "Hex Toroid P2P Sync Test",
  categoryId: "test-category",
  categorySlug: "test",
  contentCID: "QmTestHexToroidSync" + Date.now(),
  thumbnailCID: "QmThumbTest123",
  metadata: {
    description: "Real Ed25519 signed release to test hex toroid block propagation!",
    tags: ["test", "hex-toroid", "p2p", "real-signature"]
  }
};

const body = JSON.stringify(releaseData);
const timestamp = Date.now();

// Sign the message: timestamp:body
const message = timestamp + ':' + body;
const messageBytes = new TextEncoder().encode(message);
const signature = await ed25519.signAsync(messageBytes, privateKey);
const signatureHex = bytesToHex(signature);

console.log('Timestamp:', timestamp);
console.log('Signature:', signatureHex.substring(0, 32) + '...');

// Create the release
console.log('\n🚀 Creating release (will broadcast via hex toroid)...');
const releaseResponse = await fetch('https://api.palace.riff.cc/api/v1/releases', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
    'X-Public-Key': publicKeyFormatted,
    'X-Timestamp': timestamp.toString(),
    'X-Signature': signatureHex
  },
  body: body
});

const releaseResult = await releaseResponse.json();
console.log('Release creation result:', releaseResult);

if (releaseResult.success) {
  const releaseId = releaseResult.id;
  console.log('\n✅ Release created with ID:', releaseId);
  console.log('\n⏳ Waiting 3 seconds for hex toroid block propagation...');
  await new Promise(resolve => setTimeout(resolve, 3000));

  console.log('\n🔍 Checking if release synced to all 3 nodes via HAProxy round-robin...');

  // Check multiple times (HAProxy will round-robin to different nodes)
  for (let i = 0; i < 5; i++) {
    try {
      const checkResponse = await fetch('https://api.palace.riff.cc/api/v1/releases/' + releaseId);
      if (checkResponse.ok) {
        const release = await checkResponse.json();
        const nodeName = release.site_address || 'unknown';
        console.log('  Request ' + (i+1) + ': ✅ Found release "' + release.name + '" (node: ' + nodeName + ')');
      } else {
        console.log('  Request ' + (i+1) + ': ❌ Release not found (HTTP ' + checkResponse.status + ')');
      }
    } catch (e) {
      console.log('  Request ' + (i+1) + ': ❌ Error:', e.message);
    }
  }
} else {
  console.log('\n❌ Failed to create release:', releaseResult);
}
