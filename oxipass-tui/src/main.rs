use clap::{Parser, Subcommand};
use oxipass_tui::core::{Vault, VaultError};
use oxipass_tui::tui;
use std::path::{Path, PathBuf};

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
    },
    /// Open an existing vault
    Open {
        /// Path to the vault file
        path: PathBuf,
    },
}

fn open_vault(path: &Path) -> Result<(Vault, String), VaultError> {
    let password = rpassword::prompt_password("Master password: ")?;
    let vault = Vault::load(path, &password)?;
    Ok((vault, password))
}

fn create_vault(path: &Path) -> Result<(Vault, String), VaultError> {
    if path.exists() {
        return Err(VaultError::VaultExists(path.to_path_buf()));
    }
    let password = rpassword::prompt_password("New master password: ")?;
    let confirm = rpassword::prompt_password("Confirm master password: ")?;
    if password != confirm {
        return Err(VaultError::PasswordMismatch);
    }
    let vault = Vault::new();
    vault.save(path, &password)?;
    println!("Vault created at {}", path.display());
    Ok((vault, password))
}

fn main() -> Result<(), VaultError> {
    let args = Args::parse();

    let (path, vault, password) = match args.command {
        Command::New { path } => {
            let path = path.with_extension("opdb");
            let (vault, password) = create_vault(&path)?;
            (path, vault, password)
        }
        Command::Open { path } => {
            let path = path.with_extension("opdb");
            let (vault, password) = open_vault(&path)?;
            (path, vault, password)
        }
    };

    tui::run(vault, path, password)?;
    Ok(())
}
