use crate::config::Config;
use crate::model::request_script;
use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::PathBuf;

pub async fn run_script_mode(config: &Config, prompt_text: &str, output: Option<&str>) -> Result<()> {
    if prompt_text.trim().is_empty() {
        println!(
            "{}",
            "Script mode requires a prompt (e.g. qwen-cli --script -o clean.sh \"describe the script\")".red()
        );
        return Ok(());
    }

    let script = request_script(config, prompt_text).await?;

    let filename = output.unwrap_or("generated_script.sh");
    let path = PathBuf::from(filename);

    fs::write(&path, script).with_context(|| format!("Failed to write script to {:?}", path))?;

    // Try to make it executable (best-effort)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms)?;
    }

    println!(
        "{} {:?}",
        "Script written to".green().bold(),
        path.as_os_str()
    );
    println!("{}", "Review it carefully before running:".yellow());
    println!("  {}", format!("bash {:?}", path.as_os_str()).yellow());

    Ok(())
}
