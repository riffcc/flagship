// Special plugin to handle Peerbit SQLite worker files
export function sqliteWorkerPlugin() {
  return {
    name: 'sqlite-worker-plugin',
    
    // Transform import paths to the correct location
    resolveId(source) {
      if (source === 'public/peerbit/sqlite3.worker.min.js') {
        return '/peerbit/sqlite3.worker.min.js';
      }
      return null;
    }
  };
} 