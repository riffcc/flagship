import type {App} from 'vue';
import { Peerbit } from 'peerbit';
import { Documents } from '@peerbit/document';
import { TrustedNetwork } from '@peerbit/trusted-network';

// Libp2p specific imports
import { createLibp2p, Libp2pOptions } from 'libp2p';
import { bootstrap } from '@libp2p/bootstrap';
import { webSockets } from '@libp2p/websockets';
import * as filters from '@libp2p/websockets/filters';
import { noise } from '@chainsafe/libp2p-noise';
import { mplex } from '@libp2p/mplex';
import { yamux } from '@chainsafe/libp2p-yamux';

// Utilities
import { hrtime } from '@peerbit/time';
import { logger as peerbitLogger } from '@peerbit/logger';

const textEncoder = new TextEncoder();

export default {
  install: async (app: App) => {
    const siteId = import.meta.env.VITE_SITE_ID as string;
    const bootstrappersRaw = import.meta.env.VITE_BOOTSTRAPPERS as string | undefined;

    if (!siteId) {
      throw new Error('VITE_SITE_ID is missing');
    }

    // 1. Configure and create libp2p instance
    const libp2pConfig: Libp2pOptions = {
      addresses: {
        listen: ['/ip4/0.0.0.0/tcp/0/ws'] 
      },
      transports: [webSockets({ filter: filters.all })],
      connectionEncrypters: [noise()],
      streamMuxers: [yamux(), mplex()],
      peerDiscovery: bootstrappersRaw
        ? [bootstrap({ list: bootstrappersRaw.split(',').map(b => b.trim()) })]
        : [],
    };
    const libp2p = await createLibp2p(libp2pConfig);

    // 2. Create Peerbit client
    const peerbitClient = await Peerbit.create({ 
      libp2p: libp2p as any,
      directory: `./.peerbit/${siteId}` 
    });

    // 3. Open Documents store
    const documentsInstance = new Documents({ 
      id: textEncoder.encode(`site-${siteId}-documents`)
    });
    const documents = await peerbitClient.open(documentsInstance);

    // 4. Open TrustedNetwork program
    const trustedNetworkInstance = new TrustedNetwork({
      id: textEncoder.encode(`site-${siteId}-trusted-network`),
      rootTrust: peerbitClient.identity.publicKey 
    });
    const network = await peerbitClient.open(trustedNetworkInstance);

    // 5. Initialize utilities
    const timeUtility = hrtime(); 
    const logInstance = peerbitLogger({ module: 'peerbit-plugin' });

    // 6. Provide to Vue app
    app.config.globalProperties.$peerbit = peerbitClient;
    app.config.globalProperties.$program = peerbitClient; 
    app.config.globalProperties.$documents = documents;
    app.config.globalProperties.$network = network;
    app.config.globalProperties.$time = timeUtility;
    app.config.globalProperties.$logger = logInstance;

    app.provide('peerbit', peerbitClient);
    app.provide('program', peerbitClient);
    app.provide('documents', documents);
    app.provide('network', network);
    app.provide('time', timeUtility);
    app.provide('logger', logInstance);
  },
}; 