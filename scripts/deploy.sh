#!/bin/bash

# Deployment script for GoQuant Upgrade System

set -e

NETWORK=${1:-devnet}

echo "Deploying to $NETWORK..."

# Set Solana cluster
solana config set --url $NETWORK

# Build program
cd programs/upgrade-manager
anchor build

# Deploy
anchor deploy --provider.cluster $NETWORK

echo "Deployment complete!"
echo "Program ID: $(solana address -k target/deploy/upgrade_manager-keypair.json)"

