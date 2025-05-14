#!/bin/bash

export VITE_SITE_ID=lens2
export VITE_SITE_NAME="Lens 2" 
export VITE_BOOTSTRAPPERS=/dns4/65da3760cb3fd2926532310b0650ddca4f88ebd5.peerchecker.com/tcp/4003/wss/p2p/12D3KooWMQTwyWnvKyFPjs72bbrDMUDM7pmtF328X7iTfWws3A18
export VITE_PORT=5176

echo "Starting Lens 2"
echo "Site ID: $VITE_SITE_ID"
echo "Bootstrapper: $VITE_BOOTSTRAPPERS"

PORT=5176 pnpm watch:web 