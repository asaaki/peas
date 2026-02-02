use crate::config::{DATA_DIR, IdMode, PeasConfig, PeasSettings};
use anyhow::Result;
use colored::Colorize;

pub fn handle_init(prefix: String, id_length: usize) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let data_path = cwd.join(DATA_DIR);
    let config_path = data_path.join("config.toml");

    // Check for both new and legacy config locations
    if config_path.exists() {
        anyhow::bail!("Project already initialized at {}", config_path.display());
    }
    for legacy in [".peas.toml", ".peas.yml", ".peas.yaml", ".peas.json"] {
        let legacy_path = cwd.join(legacy);
        if legacy_path.exists() {
            anyhow::bail!(
                "Project already initialized with legacy config at {}. Please migrate to {}/config.toml",
                legacy_path.display(),
                DATA_DIR
            );
        }
    }

    let config = PeasConfig {
        peas: PeasSettings {
            path: None,
            prefix,
            id_length,
            id_mode: IdMode::Random,
            default_status: "todo".to_string(),
            default_type: "task".to_string(),
            frontmatter: "toml".to_string(),
        },
        tui: Default::default(),
    };

    // Create data directory
    std::fs::create_dir_all(&data_path)?;

    // Save config inside .peas/
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
