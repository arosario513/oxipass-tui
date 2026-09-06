# oxipass-tui

[![CI](https://github.com/arosario513/oxipass-tui/actions/workflows/ci.yml/badge.svg)](https://github.com/arosario513/oxipass-tui/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/arosario513/oxipass-tui?logo=github)](https://github.com/arosario513/oxipass-tui/releases/latest)
[![AUR](https://img.shields.io/aur/version/oxipass-tui-bin?logo=archlinux&logoColor=white)](https://aur.archlinux.org/packages/oxipass-tui-bin)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A terminal-based password manager inspired by KeePassXC, written in Rust. All data is stored in a single encrypted local file, which means no cloud, no network, no dependencies on system keyring or clipboard daemons. **You control everything.**

![screenshot](https://github.com/user-attachments/assets/52d26af1-2ed1-4c8a-b900-1b6a763946a0)

![password-generator](https://github.com/user-attachments/assets/412ddf22-170e-4714-a124-b0d898f4c94d)

![edit](https://github.com/user-attachments/assets/081805aa-b4e0-4811-ba4e-0a930d6b263d)

## Features

- **Entry types:** Login, Payment, Note
- **TOTP:** add an `otpauth://` URI or a base32 secret to any Login; the live 6-digit code and its countdown show in the preview pane and can be copied from the copy picker. Stored the same way KeePassXC stores it (the `otp` field), so it carries across on import/export.
- **Encryption:** AES-256-GCM with Argon2 key derivation, deflate-compressed before encryption
- **Keyfile support:** optional `.opkey` file as a second factor, the vault cannot be opened without both the master password and the keyfile
- **Password generator:** configurable length and character sets, entropy display, zxcvbn strength scoring shown in both the generator and the preview pane
- **Clipboard:** OSC 52 terminal escape sequence which works in any modern terminal without X11/Wayland libraries
- **Copy picker:** press `c` to choose any field to copy, not just the primary secret
- **Multiline notes** on all entry types; `Enter` inserts a newline, `Alt+Enter` submits
- **Live search** across all entry fields
- **Preview pane** with inline reveal toggle for secrets

## Build

Requires a Rust toolchain.

```bash
cargo build --release
# binary at target/release/oxipass-tui
```

## Usage

```bash
# Create a new vault
oxipass-tui new ~/passwords

# Open an existing vault
oxipass-tui open ~/passwords.opdb

# Create a new vault protected by a keyfile
oxipass-tui keygen ~/my.opkey
oxipass-tui new ~/passwords -k ~/my.opkey

# Open a keyfile-protected vault
oxipass-tui open ~/passwords.opdb -k ~/my.opkey
```

Vaults are saved as `<name>.opdb`. Keyfiles are saved as `<name>.opkey`.

## Key bindings

### Normal

| Key            | Action                           |
| -------------- | -------------------------------- |
| `j` / `↓`      | Move down                        |
| `k` / `↑`      | Move up                          |
| `r`            | Reveal / hide secrets in preview |
| `c`            | Open copy picker                 |
| `/`            | Search                           |
| `a`            | Add entry                        |
| `e`            | Edit selected entry              |
| `d`            | Delete selected entry            |
| `g`            | Open password generator          |
| `q` / `Ctrl+C` | Quit                             |

### Add / Edit form

| Key               | Action                                                               |
| ----------------- | -------------------------------------------------------------------- |
| `Tab` / `↓`       | Next field                                                           |
| `Shift+Tab` / `↑` | Previous field                                                       |
| `Enter`           | Next field / confirm on last (or insert newline in multiline fields) |
| `Alt+Enter`       | Submit form from a multiline field                                   |
| `Ctrl+G`          | Open password generator (on password field)                          |
| `Esc`             | Cancel                                                               |

### Password generator

| Key                   | Action                                          |
| --------------------- | ----------------------------------------------- |
| `j` / `k`             | Decrease / increase length                      |
| `r` / `Space`         | Regenerate                                      |
| `u` / `l` / `d` / `s` | Toggle Uppercase / Lowercase / Digits / Symbols |
| `c`                   | Copy generated password                         |
| `Enter`               | Use password (in form) / close (standalone)     |
| `Esc`                 | Cancel                                          |

## File format

### Vault (`.opdb`)

```
[salt: 16 bytes][nonce: 12 bytes][ciphertext + GCM tag]
```

The plaintext is JSON, deflate-compressed and then AES-256-GCM encrypted. The 32-byte key is derived from the master password (concatenated with the keyfile bytes if present) using Argon2. The vault is fully re-encrypted on every save. A wrong master password causes AES-GCM authentication to fail. No plaintext is ever exposed.

### Keyfile (`.opkey`)

```json
{
  "version": "1.0.0",
  "hash": "ac7fef9b",
  "data": "<base64-encoded 32 random bytes>"
}
```

The `hash` field is the first 4 bytes of SHA-256 over the raw key bytes, encoded as lowercase hex. It is used only to detect a corrupted keyfile, not for authentication.
