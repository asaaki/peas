use crate::config::{PeasConfig, PeasSettings};
use anyhow::Result;
use colored::Colorize;

pub fn handle_init(prefix: String, id_length: usize, peas_path: Option<String>) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let config_path = cwd.join(".peas.toml");

    if config_path.exists() {
        anyhow::bail!("Project already initialized at {}", config_path.display());
    }

    let data_dir = peas_path.unwrap_or_else(|| ".peas".to_string());

    let config = PeasConfig {
        peas: PeasSettings {
            path: data_dir.clone(),
            prefix,
            id_length,
            default_status: "todo".to_string(),
            default_type: "task".to_string(),
            frontmatter: "toml".to_string(),
        },
        tui: Default::default(),
    };

    // Create data directory
    let data_path = cwd.join(&data_dir);
    std::fs::create_dir_all(&data_path)?;

    // Save config
    config.save(&config_path)?;

    println!(
        "{} peas project in {}",
        "Initialized".green(),
        cwd.display()
    );
    println!("  Config: {}", config_path.display());
    println!("  Data:   {}", data_path.display());

    Ok(())
}
