# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

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

## Important Instructions
- NEVER use git checkout to revert changes - this will throw away hours of work
- Always manually revert specific changes using the Edit tool
- Do what has been asked; nothing more, nothing less
- NEVER create files unless they're absolutely necessary
- ALWAYS prefer editing existing files to creating new ones