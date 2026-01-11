---
title: Mobile App
description: Secure wallet and signing approvals for Tana
sidebar:
  order: 1
---

Tana Mobile is a React Native application built with Expo that provides secure wallet storage, balance visibility, and transaction signing for the Tana blockchain.

## Features

- **Secure Key Storage** - Ed25519 wallet keys stored entirely on-device using secure storage
- **Balance Visibility** - View your current ledger balances linked to wallet addresses
- **Transaction Signing** - Authorize external requests (web, CLI) by signing challenge payloads
- **Automatic Registration** - Wallets are automatically registered on the blockchain when created
- **Multi-Platform** - iOS, Android, and Web support via Expo

## Getting Started

### Installation

```bash
cd mobile
npm install
npm run start  # Opens Expo Dev Tools
```

Launch directly on a platform:
```bash
npm run ios      # iOS simulator
npm run android  # Android emulator
npm run web      # Web browser
```

### Environment Configuration

By default, the app connects to your local development ledger at `http://localhost:8080`.

Make sure your local ledger service is running:

```bash
# Start the app (defaults to localhost:8080)
npm start
```

Then press `i` for iOS simulator or `a` for Android emulator.

**To connect to production:**

```bash
# Option 1: Use the convenience script
npm run start:prod

# Option 2: Set TANA_LEDGER_URL manually
TANA_LEDGER_URL=https://mainnet.tana.network npm start
```

See [Environment Variables](/contributing/env-vars/) for more configuration options.

## Using the App

### Creating a Wallet

1. **Open the Vault screen** - Your main wallet dashboard
2. **Tap "Add Wallet"**
3. **Choose an option:**
   - **Generate New** - Creates a new Ed25519 keypair
   - **Import Existing** - Import from private key
4. **Enter details:**
   - Display name (your public blockchain name)
   - Username (unique blockchain handle, must start with @)
5. **Tap "Generate & Register" or "Import & Register"**

Your wallet is automatically registered on the blockchain with your public key!

### Blockchain Registration

Every wallet creation automatically registers on the blockchain:
- Username must be unique (app will error if taken)
- Display name and username are required fields
- Backend creates a `user_creation` transaction immediately
- **Registration completes when the next block is produced**

**Transaction Flow:**
- Wallet is saved locally immediately with a pre-assigned user ID
- The user creation transaction is marked as "pending"
- User appears on blockchain after block production
- In production, blocks are produced automatically (e.g., every 6 seconds)

### Viewing Balances

From the Vault screen:
- See all your wallets and their balances
- Tap a wallet to view details
- Copy public keys or export private keys (securely!)

### Signing Transactions

The approval screen accepts QR codes or manual challenges:
1. Scan a QR code or paste a challenge
2. Choose which wallet to sign with
3. Review the transaction details
4. Approve to generate an Ed25519 signature

Your signature can then be used to authorize transactions on the blockchain.

## Security

- **Wallet metadata** stored in `AsyncStorage`
- **Private keys** stored in secure storage (`expo-secure-store`)
- **Keys never leave your device** - all signing happens locally
- **Export keys before uninstalling** - secure storage is cleared on uninstall

:::caution
Always back up your private keys! If you lose your device and haven't exported your keys, they cannot be recovered.
:::

## Troubleshooting

### White Screen on Launch

If you see a white screen, ensure all dependencies are installed:

```bash
npx expo-doctor  # Check for issues
npx expo install --fix  # Fix package versions
```

Common fixes:
- Install `expo-font`: `npx expo install expo-font`
- Remove `@types/react-native` if installed (types are included with react-native)

### Xcode Version

Expo SDK 51+ requires Xcode ≤16.2.0. If you have a newer version:
- Downgrade Xcode for iOS builds
- Use the Expo Go app for testing instead of building locally

### Connection Issues

If the app can't connect to the ledger:
- Verify the ledger is running: `tana status`
- Check the `TANA_LEDGER_URL` environment variable
- For local development on a physical device, use your computer's IP address instead of `localhost`

## Technical Details

### Project Structure

```
mobile/
├── App.tsx                     # Entry point with navigation
├── app.config.js               # Expo configuration + env vars
├── src/
│   ├── components/
│   │   └── KeyCard.tsx         # Wallet summary display
│   ├── contexts/
│   │   └── KeyStoreContext.tsx # Wallet vault + secure storage
│   ├── lib/
│   │   └── ledgerApi.ts        # REST API client
│   ├── screens/
│   │   ├── AddKeyScreen.tsx    # Generate/import wallets
│   │   ├── LoginApprovalScreen.tsx # Challenge signing
│   │   └── VaultScreen.tsx     # Wallet list, balances
│   └── utils/
│       └── crypto.ts           # Ed25519 cryptography
```

### Compatibility

- React Native 0.81 / Expo SDK 54
- TypeScript strict mode enabled
- Ed25519 via `@noble/ed25519`
- QR scanning via `expo-camera`

## Future Enhancements

- Biometric authentication for transaction signing
- QR-powered challenges for easier approvals
- Push/deep link handling for automatic approval requests
- Encrypted backup/export flows for device migration
- Multi-network support with configurable endpoints

## Related Documentation

- [CLI Commands](/tana-cli/commands/) - Command-line interface
- [API Reference](/tana-api/intro/) - Ledger API endpoints
- [Environment Variables](/contributing/env-vars/) - Configuration options
