use clap::{Parser, Subcommand};
use oxipass_tui::core::{KeyFile, MasterKey, Vault, VaultError};
use oxipass_tui::tui;
use std::path::{Path, PathBuf};
use zeroize::Zeroizing;

const OK: &str = "[\x1b[32m+\x1b[0m]";
const WARN: &str = "[\x1b[33m*\x1b[0m]";
const ERR: &str = "[\x1b[31m!\x1b[0m]";

#[derive(Parser)]
#[command(name = "oxipass", about = "A local password manager")]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a new vault
    New {
        /// Path to the vault file
        path: PathBuf,
        /// Optional key file (.opkey)
        #[arg(long, short)]
        keyfile: Option<PathBuf>,
    },
    /// Open an existing vault
    Open {
        /// Path to the vault file
        path: PathBuf,
        /// Optional key file (.opkey)
        #[arg(long, short)]
        keyfile: Option<PathBuf>,
    },
    /// Generate a new key file
    Keygen {
        /// Path to write the key file (.opkey will be appended if missing)
        path: PathBuf,
    },
}

fn load_keyfile(path: &Path) -> Result<Zeroizing<[u8; 32]>, VaultError> {
    let path = path.with_extension("opkey");
    KeyFile::load(&path)?.key_bytes()
}

// Password and keyfile bytes are wiped when these return; only the derived
// MasterKey is carried into the TUI.
fn open_vault(path: &Path, keyfile: Option<&Path>) -> Result<(Vault, MasterKey), VaultError> {
    let password = Zeroizing::new(rpassword::prompt_password("Master password: ")?);
    let kf_bytes = keyfile.map(load_keyfile).transpose()?;
    Vault::load(path, &password, kf_bytes.as_ref().map(|k| k.as_slice()))
}

fn create_vault(path: &Path, keyfile: Option<&Path>) -> Result<(Vault, MasterKey), VaultError> {
    if path.exists() {
        return Err(VaultError::VaultExists(path.to_path_buf()));
    }
    let password = Zeroizing::new(rpassword::prompt_password("New master password: ")?);
    let confirm = Zeroizing::new(rpassword::prompt_password("Confirm master password: ")?);
    if password != confirm {
        return Err(VaultError::PasswordMismatch);
    }
    let kf_bytes = keyfile.map(load_keyfile).transpose()?;
    let key = MasterKey::new(&password, kf_bytes.as_ref().map(|k| k.as_slice()))?;
    let vault = Vault::new();
    vault.save(path, &key)?;
    println!("{} Vault created at {}", OK, path.display());
    Ok((vault, key))
}

fn main() -> Result<(), VaultError> {
    let args = Args::parse();

    match args.command {
        Command::Keygen { path } => {
            let path = path.with_extension("opkey");
            if path.exists() {
                eprintln!("{} File already exists: {}", ERR, path.display());
                std::process::exit(1);
            }
            let kf = KeyFile::generate()?;
            kf.save(&path)?;
            println!("{} Key file generated: {}", OK, path.display());
            println!(
                "{} Keep this file safe, losing it means losing access to any vault it protects.",
                WARN
            );
            Ok(())
        }
        Command::New { path, keyfile } => {
            let path = path.with_extension("opdb");
            let (vault, key) = create_vault(&path, keyfile.as_deref())?;
            tui::run(vault, path, key)?;
            Ok(())
        }
        Command::Open { path, keyfile } => {
            let path = path.with_extension("opdb");
            match open_vault(&path, keyfile.as_deref()) {
                Ok((vault, key)) => tui::run(vault, path, key)?,
                Err(VaultError::Crypto(_)) if keyfile.is_none() => {
                    eprintln!(
                        "{} Failed to open vault. Wrong password, or this vault requires a key file (-k).",
                        ERR
                    );
                    std::process::exit(1);
                }
                Err(e) => return Err(e),
            }
            Ok(())
        }
    }
}
