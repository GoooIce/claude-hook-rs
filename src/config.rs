//! Configuration loading and management

use crate::types::{Config, ConfigError, CONFIG_FILE_NAMES, BACKUP_SUFFIX};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Finds the first available configuration file in the search order.
///
/// Searches for configuration files in the order defined by CONFIG_FILE_NAMES.
/// This provides backward compatibility by checking the new file name first,
/// then falling back to the legacy file name.
///
/// # Returns
/// * `Ok(PathBuf)` - Path to the found configuration file
/// * `Err(ConfigError::NotFound)` - If no configuration file is found
pub fn find_config_file() -> Result<PathBuf, ConfigError> {
    for filename in CONFIG_FILE_NAMES {
        let path = PathBuf::from(filename);
        if path.exists() {
            return Ok(path);
        }
    }
    Err(ConfigError::NotFound(
        "No configuration file found. Searched for: .claude.toml, .claude-hook-advisor.toml".to_string()
    ))
}

/// Loads configuration using the new file discovery mechanism.
///
/// This function automatically searches for configuration files in the
/// preferred order and loads the first one found.
pub fn load_config_auto() -> Result<Config> {
    match find_config_file() {
        Ok(config_path) => load_config_from_path(&config_path),
        Err(ConfigError::NotFound(_)) => {
            // No config file found - return empty config with a warning
            eprintln!("ℹ️  No configuration file found. Run with --init-config to create one.");
            Ok(Config {
                commands: HashMap::new(),
                semantic_directories: HashMap::new(),
            })
        }
        Err(e) => Err(e.into()),
    }
}

/// Checks if configuration migration is needed.
///
/// Returns the path to the old configuration file if it exists and
/// the new configuration file does not exist.
pub fn needs_migration() -> Option<PathBuf> {
    let old_config = PathBuf::from(".claude-hook-advisor.toml");
    let new_config = PathBuf::from(".claude.toml");

    if old_config.exists() && !new_config.exists() {
        Some(old_config)
    } else {
        None
    }
}

/// Migrates configuration from old file name to new file name.
///
/// Creates a backup of the original file before migration.
/// Validates the new configuration after migration.
pub fn migrate_config() -> Result<PathBuf, ConfigError> {
    let old_path = PathBuf::from(".claude-hook-advisor.toml");
    let new_path = PathBuf::from(".claude.toml");
    let backup_path = PathBuf::from(format!("{}{}", old_path.display(), BACKUP_SUFFIX));

    // Verify old config exists and new config doesn't
    if !old_path.exists() {
        return Err(ConfigError::MigrationFailed(
            "Old configuration file does not exist".to_string()
        ));
    }

    if new_path.exists() {
        return Err(ConfigError::MigrationFailed(
            "New configuration file already exists".to_string()
        ));
    }

    // Create backup
    fs::copy(&old_path, &backup_path).map_err(|e|
        ConfigError::BackupFailed(format!("Failed to create backup: {}", e))
    )?;

    // Copy to new location
    fs::copy(&old_path, &new_path).map_err(|e|
        ConfigError::MigrationFailed(format!("Failed to copy to new location: {}", e))
    )?;

    // Validate new configuration
    load_config_from_path(&new_path).map_err(|e| {
        // If validation fails, remove the new file and keep backup
        let _ = fs::remove_file(&new_path);
        ConfigError::MigrationFailed(format!("New configuration validation failed: {}", e))
    })?;

    // If everything succeeded, remove the original file
    fs::remove_file(&old_path).map_err(|e| {
        ConfigError::MigrationFailed(format!("Failed to remove original file: {}", e))
    })?;

    Ok(new_path)
}

/// Loads configuration from a specific path.
///
/// # Arguments
/// * `config_path` - Path to the configuration file
///
/// # Returns
/// * `Ok(Config)` - Loaded configuration
/// * `Err` - If file cannot be read or parsed
pub fn load_config_from_path(config_path: &Path) -> Result<Config> {
    let content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

    Ok(config)
}

/// Loads configuration from a TOML file path (legacy function for compatibility).
///
/// If the config file doesn't exist, returns an empty configuration and logs
/// a warning to stderr. This allows the tool to work gracefully without config.
/// 
/// # Arguments
/// * `config_path` - Path to the .claude-hook-advisor.toml file
/// 
/// # Returns
/// * `Ok(Config)` - Loaded configuration or empty config if file not found
/// * `Err` - If file exists but cannot be read or parsed
#[deprecated(note = "Use load_config_auto() or load_config_from_path() instead")]
#[allow(dead_code)]
pub fn load_config(config_path: &str) -> Result<Config> {
    if !Path::new(config_path).exists() {
        // Log warning to stderr when config file is not found
        eprintln!("Warning: Config file '{config_path}' not found. No command mappings will be applied.");
        return Ok(Config {
            commands: HashMap::new(),
            semantic_directories: HashMap::new(),
        });
    }

    let content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config file: {config_path}"))?;

    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {config_path}"))?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_config_loading_missing_file() {
        // Test loading non-existent config file
        let result = load_config("non-existent-file.toml");
        assert!(result.is_ok()); // Should return empty config
        let config = result.unwrap();
        assert!(config.commands.is_empty());
    }

    #[test]
    fn test_find_config_file_new() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".claude.toml");

        // Create new config file
        fs::write(&config_path, "[commands]\nnpm = \"bun\"").unwrap();

        // Test that it finds the new config file
        let result = find_config_file_in_dir(temp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), config_path);
    }

    #[test]
    fn test_find_config_file_fallback_to_old() {
        let temp_dir = TempDir::new().unwrap();
        let old_config_path = temp_dir.path().join(".claude-hook-advisor.toml");

        // Create only old config file
        fs::write(&old_config_path, "[commands]\nnpm = \"bun\"").unwrap();

        // Test that it finds the old config file as fallback
        let result = find_config_file_in_dir(temp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), old_config_path);
    }

    #[test]
    fn test_find_config_file_priority() {
        let temp_dir = TempDir::new().unwrap();
        let new_config_path = temp_dir.path().join(".claude.toml");
        let old_config_path = temp_dir.path().join(".claude-hook-advisor.toml");

        // Create both config files
        fs::write(&old_config_path, "[commands]\nnpm = \"bun\"").unwrap();
        fs::write(&new_config_path, "[commands]\nnpm = \"bunx\"").unwrap();

        // Test that it prefers the new config file
        let result = find_config_file_in_dir(temp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), new_config_path);
    }

    #[test]
    fn test_needs_migration() {
        let temp_dir = TempDir::new().unwrap();
        let old_config = temp_dir.path().join(".claude-hook-advisor.toml");
        let new_config = temp_dir.path().join(".claude.toml");

        // Test when only old config exists
        fs::write(&old_config, "[commands]\nnpm = \"bun\"").unwrap();
        let result = needs_migration_in_dir(temp_dir.path());
        assert!(result.is_some());
        assert_eq!(result.unwrap(), old_config);

        // Test when new config exists
        fs::write(&new_config, "[commands]\nnpm = \"bun\"").unwrap();
        let result = needs_migration_in_dir(temp_dir.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_migration() {
        let temp_dir = TempDir::new().unwrap();
        let old_config = temp_dir.path().join(".claude-hook-advisor.toml");
        let new_config = temp_dir.path().join(".claude.toml");

        // Create old config
        fs::write(&old_config, "[commands]\nnpm = \"bun\"").unwrap();

        // Perform migration
        let result = migrate_config_in_dir(temp_dir.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), new_config);

        // Verify results
        assert!(!old_config.exists());
        assert!(new_config.exists());

        // Verify backup was created
        let backup = temp_dir.path().join(".claude-hook-advisor.toml.backup");
        assert!(backup.exists());
    }

    #[test]
    fn test_load_config_auto() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join(".claude.toml");

        // Create config file
        fs::write(&config_path, "[commands]\nnpm = \"bun\"").unwrap();

        // Test auto loading
        let result = load_config_auto_in_dir(temp_dir.path());
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.commands.get("npm"), Some(&"bun".to_string()));
    }

    // Helper functions for testing with different directories
    fn find_config_file_in_dir(dir: &std::path::Path) -> Result<std::path::PathBuf, ConfigError> {
        for filename in CONFIG_FILE_NAMES {
            let path = dir.join(filename);
            if path.exists() {
                return Ok(path);
            }
        }
        Err(ConfigError::NotFound("No config found".to_string()))
    }

    fn needs_migration_in_dir(dir: &std::path::Path) -> Option<std::path::PathBuf> {
        let old_config = dir.join(".claude-hook-advisor.toml");
        let new_config = dir.join(".claude.toml");

        if old_config.exists() && !new_config.exists() {
            Some(old_config)
        } else {
            None
        }
    }

    fn migrate_config_in_dir(dir: &std::path::Path) -> Result<std::path::PathBuf, ConfigError> {
        let old_path = dir.join(".claude-hook-advisor.toml");
        let new_path = dir.join(".claude.toml");
        let backup_path = dir.join(format!("{}{}", old_path.display(), BACKUP_SUFFIX));

        // Create backup
        fs::copy(&old_path, &backup_path)?;

        // Copy to new location
        fs::copy(&old_path, &new_path)?;

        // Validate new configuration
        load_config_from_path(&new_path)?;

        // Remove original
        fs::remove_file(&old_path)?;

        Ok(new_path)
    }

    fn load_config_auto_in_dir(dir: &std::path::Path) -> Result<Config> {
        match find_config_file_in_dir(dir) {
            Ok(config_path) => load_config_from_path(&config_path),
            Err(ConfigError::NotFound(_)) => Ok(Config {
                commands: HashMap::new(),
                semantic_directories: HashMap::new(),
            }),
            Err(e) => Err(e.into()),
        }
    }
}