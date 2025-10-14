# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 🚨 CRITICAL BUILD CONFIGURATION 🚨

### API URL Rules (READ CAREFULLY)

**PRODUCTION BUILDS:**
```bash
VITE_API_URL="https://api.global.riff.cc/api/v1" pnpm build
```
- Use `api.global.riff.cc` for ALL production deployments
- Use when building Docker images for production
- Use when deploying to live infrastructure

**DEV/TEST BUILDS:**
```bash
VITE_API_URL="https://api.palace.riff.cc/api/v1" pnpm build
```
- Use `api.palace.riff.cc` for ALL dev/test environments
- Use in Docker Compose test clusters (even with 50+ nodes)
- Use for local development and testing

**THIS HAS BEEN FORGOTTEN 10 TIMES. DO NOT FORGET IT AGAIN.**

The frontend MUST be built with the correct API URL:
- **Production:** `https://api.global.riff.cc/api/v1`
- **Dev/Test:** `https://api.palace.riff.cc/api/v1`

This is not negotiable. This is not optional. This is ALWAYS required.

## Project Overview

Flagship is Riff.CC's decentralized media platform for watching, sharing, and curating legally free content. It uses peer-to-peer technology with PeerBit for metadata and IPFS for content/data distribution, and can run as both an Electron desktop app and a web application.

## Key Commands

### Development
- `pnpm install` - Install dependencies (uses pnpm workspaces)
- `pnpm watch` - Run Electron app in development mode
- `pnpm watch:web` - Run web version with hot reload (port 5175)
- `pnpm watch:web:stub` - Run web version with stub data

### Building
- `pnpm build` - Build all packages (main, preload, renderer)
- `pnpm compile` - Build Electron app for distribution
- `pnpm compile:web` - Build web version for production

### Testing
- `pnpm test` - Run all tests
- `pnpm test:e2e` - Run E2E tests with Playwright
- `pnpm test:main` - Test main process
- `pnpm test:preload` - Test preload scripts
- `pnpm test:renderer` - Test renderer/frontend
- Run single test: `pnpm test -- path/to/test.spec.ts`

### Code Quality
- `pnpm lint` - Run ESLint
- `pnpm format` - Format code with Prettier
- `pnpm typecheck` - TypeScript type checking for all packages

## High-Level Architecture

### Monorepo Structure
- `/packages/main/` - Electron main process (system operations, window management)
- `/packages/preload/` - Secure bridge between main and renderer processes
- `/packages/renderer/` - Vue 3 SPA (works in both Electron and web)

### Technology Stack
- **Frontend**: Vue 3 + TypeScript + Vuetify 3
- **State Management**: TanStack Query (Vue Query) + Vue Composables
- **Build Tool**: Vite with multiple plugins
- **P2P Layer**: PeerBit (metadata) and IPFS (content/data)
- **Desktop**: Electron v34

### Key Architectural Patterns

1. **Service Layer Abstraction**
   - `LensService` provides core functionality with two implementations:
     - Browser implementation for web builds
     - Electron implementation using IPC for desktop
   - Service is injected as Vue plugin and accessed via composables

2. **State Management**
   - No traditional store (Vuex/Pinia)
   - Async state handled by TanStack Query
   - Local state managed through Vue Composables in `/packages/renderer/src/composables/`

3. **Dual Build Support**
   - Single codebase supports both Electron and web builds
   - Environment variables control build targets
   - Service layer abstracts platform differences

4. **Hybrid Data Loading Architecture (PR #70)**
   - **API-First Pre-fetching**: Attempts to load data from REST API immediately for instant UI
   - **Graceful P2P Fallback**: Falls back to Peerbit when API is unavailable
   - **Non-Blocking P2P Init**: Peerbit initializes in background without blocking UI
   - **Smart Loading Screen**: Shows appropriate loading state based on data source
   
   Implementation details:
   - Router guard in `plugins/router.ts` performs API health check
   - If healthy, pre-fetches and seeds TanStack Query cache
   - `composables/lensInitialization.ts` handles background P2P setup
   - `getApiUrl()` dynamically constructs API URL from multiaddr
   - Provides "near-instantaneous UI render" when API is available
   - Degrades gracefully to P2P-only mode when necessary

4. **Component Organization**
   ```
   packages/renderer/src/components/
   ├── account/     - Authentication and user profile
   ├── admin/       - Admin panel components
   ├── home/        - Homepage sections
   ├── layout/      - App shell (header, footer)
   ├── misc/        - Shared utilities
   └── releases/    - Media players and content UI
   ```

5. **Routing Structure**
   - `/` - Homepage with featured content
   - `/release/:id` - Individual release pages
   - `/admin` - Content management
   - `/upload` - Content upload interface

### Critical Entry Points
- Main process: `/packages/main/src/index.ts`
- Renderer: `/packages/renderer/src/index.ts`
- App root: `/packages/renderer/src/App.vue`
- Service integration: `/packages/renderer/src/plugins/lensService/`

### Development Notes
- The project uses pnpm workspaces for dependency management
- Hot module replacement is configured for rapid development
- TypeScript is used throughout for type safety
- Vite configs in each package control build behavior
- Content is distributed via P2P network with configurable replication factors

## CRITICAL DATA STRUCTURE NOTES

### Category and Release Structure
**Category object** has:
- `id` - The hash/unique identifier for the category
- `slug` - The slug identifier (e.g., 'tv-shows', 'music', 'movies')

**Release object** has:
- `categoryId` - References the category's hash ID
- `categorySlug` - References the category's slug

When filtering releases by category type, use `categorySlug` on the release, NOT `categoryId`!

### Structures System
Structures are completely generic organizational containers documented in `docs/STRUCTURES.md`. They can represent ANY hierarchical relationship - artists/albums, TV shows/seasons, book series/volumes, courses/lessons, etc. The system is designed for efficient PeerBit queries across arbitrary hierarchies using `parentId` relationships and content references via `metadata.structureId`.

## Key Implementation Details

### Series and Episodes
- Series structures should only exist when they have actual episodes
- Episodes link to series via `metadata.seriesId` matching the series structure's `id`
- Seasons are tracked via `metadata.seasonNumber` on episodes

### Important Reminders
- Check this file before making assumptions about data structures
- When something works on one page but not another, the issue is usually simple
- Focus on fixing exactly what's requested without adding complexity

## 🚨 CRITICAL: SPORE Protocol Understanding 🚨

### SPORE = Succinct Proof of RANGE Exclusion

**SPORE IS NOT JUST FOR BLOCKS. IT APPLIES TO ANY RANGE OF VALUES.**

SPORE is a XOR-based bitmap comparison technique for efficiently identifying missing elements in ANY coordinate space or ID space:

- **Block ID ranges** (UUIDs) - Find missing blocks
- **Peer slot coordinate ranges** - Find missing peers in mesh topology
- **ANY range of coordinates or IDs** - General-purpose range comparison

### How SPORE Works

1. **Bitmap Representation**: Represent a range of values as a compact bitmap
2. **XOR Comparison**: XOR two bitmaps to identify differences
3. **Range Exclusion**: The XOR result shows which values are in one set but not the other
4. **Succinct**: Extremely compact representation compared to sending full lists

### SPORE for Blocks

```rust
// WantList contains have_blocks (blocks I have)
// Compare with peer's WantList to find missing blocks
let my_blocks = wantlist.have_blocks;
let peer_blocks = peer_wantlist.have_blocks;

// XOR comparison identifies missing blocks
let missing = spore_compare(&my_blocks, &peer_blocks);
```

### SPORE for Peers

```rust
// WantList contains known_peers (peers I know about)
// Relay uses SPORE to send ONLY peers you DON'T know about
let known_peer_ids: HashSet<String> = wantlist.known_peers
    .iter()
    .map(|kp| kp.peer_id.clone())
    .collect();

// Filter peer referrals to exclude known peers (SPORE exclusion)
let peers_to_send: Vec<_> = all_peers
    .into_iter()
    .filter(|p| !known_peer_ids.contains(&p.peer_id))
    .collect();
```

### Key Implementation Files

- `/crates/lens-v2-p2p/src/spore.rs` - Core SPORE implementation with XOR bitmap comparison
- `/crates/palace/crates/consensus/peerexc/src/wantlist.rs` - WantList structure with known_peers field
- `/crates/lens-v2-node/src/routes/relay.rs` - Relay that filters peer referrals using SPORE

### Critical Rules

1. **SPORE applies to RANGES** - blocks, peers, coordinates, ANY range of values
2. **XOR-based comparison** - Efficient bitmap technique, not list filtering
3. **known_peers is FOR SPORE** - List of peer IDs for range exclusion filtering
4. **Relay filters using SPORE** - Send only unknown peers, not all peers

### DO NOT CONFUSE

- ❌ "SPORE is only for blocks" - WRONG
- ❌ "Simple list filtering" - WRONG, it's XOR-based bitmap comparison
- ✅ "SPORE = Succinct Proof of RANGE Exclusion" - CORRECT
- ✅ "Works on any range - blocks, peers, coordinates" - CORRECT

**Cost of forgetting this: $50 API credits + $500 of user time. Read the spec before implementing.**

---

## 🚨 CRITICAL: Citadel DHT Architecture 🚨

### Authoritative Specification

**READ THIS FIRST:** `/opt/castle/workspace/citadel/2025-10-12-Citadel-DHT-SPEC.md`

The Citadel DHT specification defines the complete architecture for:
- **Recursive DHT (Section 2.4)** - DHT uses itself for topology discovery
- **LazyNode Neighbor Discovery** - Query DHT network for slot ownership on-demand
- **O(1) Routing** - Constant-time routing decisions using deterministic key-to-slot mapping
- **Slot Ownership Keys** - Stored IN the DHT network, not locally
- **Hexagonal Toroidal Mesh** - 2.5D topology with 8 neighbors per node

### Critical Implementation Requirements

**From Section 2.4 of the spec (lines 252-563):**

1. **Slot ownership must be stored IN the DHT network** - Not in local storage!
   ```rust
   // Query DHT for "who owns this slot?"
   let key = slot_ownership_key(neighbor_slot);
   let ownership: SlotOwnership = self.dht.get(&key).await?;
   ```

2. **LazyNode queries the network DHT** - No neighbor caches needed
   ```rust
   pub async fn get_neighbor(&mut self, direction: Direction) -> DHTResult<PeerID> {
       let neighbor_slot = self.my_slot.neighbor(direction, &self.mesh_config);
       let key = slot_ownership_key(neighbor_slot);
       let ownership: SlotOwnership = self.dht.get(&key).await?;
       Ok(ownership.peer_id)
   }
   ```

3. **O(1) deterministic routing** - Every routing decision computed from first principles

4. **Minimal state (64 bytes!)** - No routing tables, no neighbor caches

### Verifying Correct Implementation

Run TDD tests to verify DHT implementation matches the spec:
```bash
cd /opt/castle/workspace/flagship/crates/lens-v2-node
cargo test dht_
```

**If nodes show `peer_count: 0` or fragmented mesh**, the DHT networking layer is not implemented according to the spec. Read Section 2.4 carefully!

---

## Important Instructions
- NEVER use git checkout to revert changes - this will throw away hours of work
- Always manually revert specific changes using the Edit tool
- Do what has been asked; nothing more, nothing less
- NEVER create files unless they're absolutely necessary
- ALWAYS prefer editing existing files to creating new ones
- Browser navigation rule: Unless explicitly stated, the USER navigates, Claude only screenshots - never use browser navigation tools without explicit permission