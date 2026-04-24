use anyhow::{Context, Result, anyhow};
use directories::ProjectDirs;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

const ENV_TOKEN: &str = "GITHUB_TOKEN";

fn token_file() -> Result<PathBuf> {
    let project_dirs = ProjectDirs::from("com", "GitNapse", "GitNapse")
        .ok_or_else(|| anyhow!("Unable to resolve project config directory"))?;
    let dir = project_dirs.config_dir();
    fs::create_dir_all(dir).with_context(|| format!("Cannot create config dir: {}", dir.display()))?;
    Ok(dir.join("token"))
}

pub fn load_token() -> Result<Option<String>> {
    if let Ok(env_token) = std::env::var(ENV_TOKEN) {
        let trimmed = env_token.trim().to_owned();
        if !trimmed.is_empty() {
            return Ok(Some(trimmed));
        }
    }

    let file = token_file()?;
    if !file.exists() {
        return Ok(None);
    }

    let token = fs::read_to_string(&file)
        .with_context(|| format!("Cannot read token file: {}", file.display()))?;
    let token = token.trim().to_owned();
    if token.is_empty() {
        return Ok(None);
    }
    Ok(Some(token))
}

pub fn save_token(token: &str) -> Result<()> {
    let token = token.trim();
    if token.is_empty() {
        return Err(anyhow!("Token is empty"));
    }

    let file = token_file()?;
    fs::write(&file, format!("{token}\n"))
        .with_context(|| format!("Cannot write token file: {}", file.display()))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&file, fs::Permissions::from_mode(0o600))
            .with_context(|| format!("Cannot set secure permissions on {}", file.display()))?;
    }

    Ok(())
}

pub fn clear_token() -> Result<()> {
    let file = token_file()?;
    if file.exists() {
        fs::remove_file(&file).with_context(|| format!("Cannot remove token file: {}", file.display()))?;
    }
    Ok(())
}

pub fn set_token_cli(token_arg: Option<String>) -> Result<()> {
    let token = match token_arg {
        Some(t) => t,
        None => {
            print!("GitHub token: ");
            io::stdout().flush().context("Cannot flush stdout")?;
            rpassword::read_password().context("Cannot read token from terminal")?
        }
    };

    save_token(&token)?;
    println!("Token saved successfully.");
    Ok(())
}

pub fn clear_token_cli() -> Result<()> {
    clear_token()?;
    println!("Stored token removed.");
    Ok(())
}

pub fn status_cli() -> Result<()> {
    let env_ok = std::env::var(ENV_TOKEN).ok().filter(|t| !t.trim().is_empty()).is_some();
    let file = token_file()?;
    let file_ok = file.exists();

    println!("Authentication status:");
    println!("- ENV {ENV_TOKEN}: {}", if env_ok { "available" } else { "missing" });
    println!(
        "- Stored token file: {} ({})",
        file.display(),
        if file_ok { "present" } else { "missing" }
    );
    Ok(())
}
