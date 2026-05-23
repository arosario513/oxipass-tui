# oxipass-tui

A terminal-based password manager inspired by KeePassXC, written in Rust. All data is stored in a single encrypted local file, which means no cloud, no network, no dependencies on system keyring or clipboard daemons. **You control everything.**

![screenshot](https://github.com/user-attachments/assets/17fc5a67-9c9c-49b8-9fa7-2155e8b03f89)

![password-generator](https://github.com/user-attachments/assets/144afc3d-0fcd-4120-8723-8fda419d39c7)

## Features

- **Entry types:** Login, Payment, Note
- **Encryption:** AES-256-GCM with Argon2 key derivation, deflate-compressed before encryption
- **Password generator:** configurable length and character sets, entropy display, zxcvbn strength scoring
- **Clipboard:** OSC 52 terminal escape sequence which works in any modern terminal without X11/Wayland libraries
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
```

Vaults are saved as `<name>.opdb`.

## Key bindings

### Normal

| Key            | Action                           |
| -------------- | -------------------------------- |
| `j` / `↓`      | Move down                        |
| `k` / `↑`      | Move up                          |
| `r`            | Reveal / hide secrets in preview |
| `c`            | Copy primary secret to clipboard |
| `/`            | Search                           |
| `a`            | Add entry                        |
| `e`            | Edit selected entry              |
| `d`            | Delete selected entry            |
| `g`            | Open password generator          |
| `q` / `Ctrl+C` | Quit                             |

### Add / Edit form

| Key               | Action                                      |
| ----------------- | ------------------------------------------- |
| `Tab` / `↓`       | Next field                                  |
| `Shift+Tab` / `↑` | Previous field                              |
| `Enter`           | Next field / confirm on last                |
| `Ctrl+G`          | Open password generator (on password field) |
| `Esc`             | Cancel                                      |

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

```
[salt: 16 bytes][nonce: 12 bytes][ciphertext + GCM tag]
```

The plaintext is JSON, deflate-compressed and then AES-256-GCM encrypted. The 32-byte key is derived from the master password using Argon2. The vault is fully re-encrypted on every save. A wrong master password causes AES-GCM authentication to fai. No plaintext is ever exposed.
