import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const sourcePaths = [
  {
    path: ['node_modules', '@peerbit', 'any-store-opfs', 'dist', 'peerbit'],
    type: 'directory',
  },
  {
    path: ['node_modules', '@peerbit', 'indexer-sqlite3', 'dist', 'peerbit'],
    type: 'directory',
  },
  {
    path: ['node_modules', '@peerbit', 'riblt', 'dist', 'rateless_iblt_bg.wasm'],
    type: 'file',
  },
];

async function copyFiles() {
  try {
    // Destination folder relative to scripts folder
    const destFolder = path.join(__dirname, '..', 'packages', 'renderer', 'public', 'peerbit');

    // Create destination folder if it doesn't exist
    await fs.mkdir(destFolder, { recursive: true });

    for (const source of sourcePaths) {
      const sourcePath = path.join(__dirname, '..', ...source.path);
      const destPath = source.type === 'file'
        ? path.join(destFolder, path.basename(sourcePath))
        : destFolder;

      try {
        if (source.type === 'file') {
          // Copy single file
          await fs.copyFile(sourcePath, destPath);
          console.log(`Copied file: ${sourcePath} to ${destPath}`);
        } else {
          // Copy directory recursively
          await fs.cp(sourcePath, destFolder, { recursive: true });
          console.log(`Copied directory: ${sourcePath} to ${destFolder}`);
        }
      } catch (error) {
        console.error(`Error copying ${sourcePath}:`, error.message);
      }
    }

    console.log('File copying completed successfully!');
  } catch (error) {
    console.error('Error:', error.message);
    process.exit(1);
  }
}

copyFiles();
