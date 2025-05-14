#!/usr/bin/env node

import fs from 'fs';
import path from 'path';
import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';

import { createPeerbit } from './peerbit';
import { CONFIG_FILE_NAME, DEFAULT_PEERBIT_DIR } from './consts';
import type { PeerbitConfig, PossiblyIncompletePeerbitConfig } from './types';

const ensurePeerbitDir = (dir: string): void => {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
};

const getPeerbitConfigPath = (dir: string): string => {
  return path.join(dir, CONFIG_FILE_NAME);
};

const readPeerbitConfig = (configPath: string): PossiblyIncompletePeerbitConfig => {
  if (!fs.existsSync(configPath)) {
    return {};
  }

  try {
    const configContent = fs.readFileSync(configPath, 'utf-8');
    return JSON.parse(configContent);
  } catch (error) {
    console.error(`Error reading config file: ${error}`);
    return {};
  }
};

const writePeerbitConfig = (configPath: string, config: PeerbitConfig): void => {
  try {
    fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
  } catch (error) {
    console.error(`Error writing config file: ${error}`);
  }
};

const generateSiteId = (): string => {
  return `site_${Math.random().toString(36).substring(2, 15)}`;
};

const main = async () => {
  const argv = await yargs(hideBin(process.argv))
    .command('config', 'Configure PeerBit', (yargs) => {
      return yargs.option('dir', {
        describe: 'Directory to store PeerBit data',
        type: 'string',
        default: path.join(process.cwd(), DEFAULT_PEERBIT_DIR),
      });
    })
    .command('start', 'Start PeerBit node', (yargs) => {
      return yargs.option('dir', {
        describe: 'Directory to store PeerBit data',
        type: 'string',
        default: path.join(process.cwd(), DEFAULT_PEERBIT_DIR),
      });
    })
    .command('add-release', 'Add a release', (yargs) => {
      return yargs
        .option('dir', {
          describe: 'Directory to store PeerBit data',
          type: 'string',
          default: path.join(process.cwd(), DEFAULT_PEERBIT_DIR),
        })
        .option('name', {
          describe: 'Release name',
          type: 'string',
          demandOption: true,
        })
        .option('file', {
          describe: 'File path',
          type: 'string',
          demandOption: true,
        })
        .option('author', {
          describe: 'Author name',
          type: 'string',
          demandOption: true,
        })
        .option('category', {
          describe: 'Content category',
          type: 'string',
          demandOption: true,
        })
        .option('thumbnail', {
          describe: 'Thumbnail path',
          type: 'string',
        })
        .option('cover', {
          describe: 'Cover image path',
          type: 'string',
        })
        .option('metadata', {
          describe: 'Metadata JSON string',
          type: 'string',
        });
    })
    .command('trust-site', 'Trust a site', (yargs) => {
      return yargs
        .option('dir', {
          describe: 'Directory to store PeerBit data',
          type: 'string',
          default: path.join(process.cwd(), DEFAULT_PEERBIT_DIR),
        })
        .option('site-id', {
          describe: 'Site ID to trust',
          type: 'string',
          demandOption: true,
        })
        .option('site-name', {
          describe: 'Site name',
          type: 'string',
          demandOption: true,
        });
    })
    .command('untrust-site', 'Untrust a site', (yargs) => {
      return yargs
        .option('dir', {
          describe: 'Directory to store PeerBit data',
          type: 'string',
          default: path.join(process.cwd(), DEFAULT_PEERBIT_DIR),
        })
        .option('site-id', {
          describe: 'Site ID to untrust',
          type: 'string',
          demandOption: true,
        });
    })
    .command('list-releases', 'List releases', (yargs) => {
      return yargs.option('dir', {
        describe: 'Directory to store PeerBit data',
        type: 'string',
        default: path.join(process.cwd(), DEFAULT_PEERBIT_DIR),
      });
    })
    .command('list-trusted-sites', 'List trusted sites', (yargs) => {
      return yargs.option('dir', {
        describe: 'Directory to store PeerBit data',
        type: 'string',
        default: path.join(process.cwd(), DEFAULT_PEERBIT_DIR),
      });
    })
    .demandCommand(1, 'You need to specify a command')
    .help()
    .argv;

  const command = argv._[0];
  const dir = argv.dir as string;

  ensurePeerbitDir(dir);

  const configPath = getPeerbitConfigPath(dir);

  switch (command) {
    case 'config': {
      const existingConfig = readPeerbitConfig(configPath);
      
      const siteId = existingConfig.siteId || generateSiteId();
      
      const config: PeerbitConfig = {
        siteId,
        ...existingConfig,
      };
      
      writePeerbitConfig(configPath, config);
      
      console.log(`PeerBit configured with site ID: ${config.siteId}`);
      console.log(`Config saved to: ${configPath}`);
      break;
    }
    
    case 'start': {
      const config = readPeerbitConfig(configPath);
      
      if (!config.siteId) {
        console.error('No site ID found. Please run "peerbit config" first.');
        process.exit(1);
      }
      
      console.log(`Starting PeerBit with site ID: ${config.siteId}`);
      
      const { peerbit } = await createPeerbit({ siteId: config.siteId });
      
      console.log('PeerBit started successfully');
      
      process.on('SIGINT', async () => {
        console.log('Shutting down PeerBit...');
        await peerbit.close();
        process.exit(0);
      });
      
      await new Promise(() => {});
      break;
    }
    
    case 'add-release': {
      const config = readPeerbitConfig(configPath);
      
      if (!config.siteId) {
        console.error('No site ID found. Please run "peerbit config" first.');
        process.exit(1);
      }
      
      const { peerbit } = await createPeerbit({ siteId: config.siteId });
      
      await peerbit.addRelease({
        contentName: argv.name as string,
        file: argv.file as string,
        author: argv.author as string,
        category: argv.category as string,
        thumbnail: argv.thumbnail as string,
        cover: argv.cover as string,
        metadata: argv.metadata ? JSON.parse(argv.metadata as string) : undefined,
      });
      
      console.log('Release added successfully');
      
      await peerbit.close();
      break;
    }
    
    case 'trust-site': {
      const config = readPeerbitConfig(configPath);
      
      if (!config.siteId) {
        console.error('No site ID found. Please run "peerbit config" first.');
        process.exit(1);
      }
      
      const { peerbit } = await createPeerbit({ siteId: config.siteId });
      
      await peerbit.trustSite({
        siteId: argv['site-id'] as string,
        siteName: argv['site-name'] as string,
      });
      
      console.log(`Site ${argv['site-id']} trusted successfully`);
      
      await peerbit.close();
      break;
    }
    
    case 'untrust-site': {
      const config = readPeerbitConfig(configPath);
      
      if (!config.siteId) {
        console.error('No site ID found. Please run "peerbit config" first.');
        process.exit(1);
      }
      
      const { peerbit } = await createPeerbit({ siteId: config.siteId });
      
      await peerbit.untrustSite({
        siteId: argv['site-id'] as string,
      });
      
      console.log(`Site ${argv['site-id']} untrusted successfully`);
      
      await peerbit.close();
      break;
    }
    
    case 'list-releases': {
      const config = readPeerbitConfig(configPath);
      
      if (!config.siteId) {
        console.error('No site ID found. Please run "peerbit config" first.');
        process.exit(1);
      }
      
      const { peerbit } = await createPeerbit({ siteId: config.siteId });
      
      const forgetFn = await peerbit.listenForReleases({
        f: (releases) => {
          console.log('Releases:');
          releases.forEach((release) => {
            console.log(`- ${release.release.release.contentName} (${release.release.id})`);
          });
          
          setTimeout(async () => {
            await forgetFn();
            await peerbit.close();
            process.exit(0);
          }, 1000);
        },
      });
      
      await new Promise((resolve) => setTimeout(resolve, 5000));
      break;
    }
    
    case 'list-trusted-sites': {
      const config = readPeerbitConfig(configPath);
      
      if (!config.siteId) {
        console.error('No site ID found. Please run "peerbit config" first.');
        process.exit(1);
      }
      
      const { peerbit } = await createPeerbit({ siteId: config.siteId });
      
      const forgetFn = await peerbit.followTrustedSites({
        f: (sites) => {
          console.log('Trusted sites:');
          sites.forEach((site) => {
            console.log(`- ${site.data.siteName} (${site.data.siteId})`);
          });
          
          setTimeout(async () => {
            await forgetFn();
            await peerbit.close();
            process.exit(0);
          }, 1000);
        },
      });
      
      await new Promise((resolve) => setTimeout(resolve, 5000));
      break;
    }
    
    default:
      console.error(`Unknown command: ${command}`);
      process.exit(1);
  }
};

main().catch((error) => {
  console.error(`Error: ${error}`);
  process.exit(1);
});
