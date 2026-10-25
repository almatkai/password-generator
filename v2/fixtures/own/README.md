# Password Generator for Tezbar

Generate cryptographically random passwords and memorable passphrases from Tezbar on Windows or macOS.

## Commands

- **Generate Random Password** validates a length from 5 to 64, guarantees the selected character groups, and copies the result.
- **Copy Random Password** creates and copies a password without opening a view.
- **Generate Memorable Password** expands word, number, symbol, and random-character tokens using the 2,048-word BIP39 English list.

## Compatibility

- Windows: built and exercised in Tezbar on Windows 11.
- macOS: uses only cross-platform Node.js and Raycast APIs and retains the upstream macOS-compatible command design.
- Linux: not currently declared.

## Local verification

```powershell
pnpm install
pnpm typecheck
pnpm test
```

Build and run each command with Tezbar's extension builder and runtime harness before publishing.

## Credits

Adapted for Tezbar from Raycast's MIT-licensed `password-generator` extension at upstream commit `6734d85992290066fd31a7f5bee3cca3d3545a1a`. Original author and contributors remain credited in `package.json`.
