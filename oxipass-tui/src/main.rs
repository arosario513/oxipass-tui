use clap::{Parser, Subcommand};
use oxipass_tui::core::{KeyFile, Vault, VaultError};
use oxipass_tui::tui;
use std::path::{Path, PathBuf};

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

fn load_keyfile(path: &Path) -> Result<Vec<u8>, VaultError> {
    let path = path.with_extension("opkey");
    let kf = KeyFile::load(&path)?;
    Ok(kf.key_bytes()?.to_vec())
}

fn open_vault(
    path: &Path,
    keyfile: Option<&Path>,
) -> Result<(Vault, String, Option<Vec<u8>>), VaultError> {
    let password = rpassword::prompt_password("Master password: ")?;
    let kf_bytes = keyfile.map(load_keyfile).transpose()?;
    let vault = Vault::load(path, &password, kf_bytes.as_deref())?;
    Ok((vault, password, kf_bytes))
}

fn create_vault(
    path: &Path,
    keyfile: Option<&Path>,
) -> Result<(Vault, String, Option<Vec<u8>>), VaultError> {
    if path.exists() {
        return Err(VaultError::VaultExists(path.to_path_buf()));
    }
    let password = rpassword::prompt_password("New master password: ")?;
    let confirm = rpassword::prompt_password("Confirm master password: ")?;
    if password != confirm {
        return Err(VaultError::PasswordMismatch);
    }
    let kf_bytes = keyfile.map(load_keyfile).transpose()?;
    let vault = Vault::new();
    vault.save(path, &password, kf_bytes.as_deref())?;
    println!("{} Vault created at {}", OK, path.display());
    Ok((vault, password, kf_bytes))
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
            let (vault, password, kf_bytes) = create_vault(&path, keyfile.as_deref())?;
            tui::run(vault, path, password, kf_bytes)?;
            Ok(())
        }
        Command::Open { path, keyfile } => {
            let path = path.with_extension("opdb");
            match open_vault(&path, keyfile.as_deref()) {
                Ok((vault, password, kf_bytes)) => tui::run(vault, path, password, kf_bytes)?,
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
