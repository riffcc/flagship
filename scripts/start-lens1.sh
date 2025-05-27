#!/bin/bash

export VITE_SITE_ID=lens1
export VITE_SITE_NAME="Lens 1" 
export VITE_BOOTSTRAPPERS=/dns4/4032881a26640025f9a4253104b7aaf6d4b55599.peerchecker.com/tcp/4003/wss/p2p/12D3KooWH1PL9hcnciEw8EUQshpGx3kRbPLoSepx3PQDESXkk34c
export VITE_PORT=5175

echo "Starting Lens 1"
echo "Site ID: $VITE_SITE_ID"
echo "Bootstrapper: $VITE_BOOTSTRAPPERS"

pnpm watch:web 