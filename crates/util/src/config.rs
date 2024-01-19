use std::{fs, path::PathBuf};

const DEFAULT_CONFIG_FILE_PATH: &str = "/etc/security/authramp.conf";

#[derive(Debug)]
pub struct Config {
    // Directory where tally information is stored.
    pub tally_dir: PathBuf,
    // Number of allowed free authentication attempts before applying delays.
    pub free_tries: i32,
    // Base delay applied to each authentication failure.
    pub base_delay_seconds: i32,
    // Multiplier for the delay calculation based on the number of failures.
    pub ramp_multiplier: i32,
    // Even lock out root user
    pub even_deny_root: bool,
}

impl Default for Config {
    /// Creates a default 'Config' struct. Default configruation values are set here.
    fn default() -> Self {
        Config {
            tally_dir: PathBuf::from("/var/run/authramp"),
            free_tries: 6,
            base_delay_seconds: 30,
            ramp_multiplier: 50,
            even_deny_root: false,
        }
    }
}

impl Config {
    /// Loads configuration config from an INI file, returning a `Config` instance.
    ///
    /// # Arguments
    ///
    /// * `config_file`: An optional `PathBuf` specifying the path to the INI file. If
    ///   not provided, the default configuration file path is used.
    ///
    /// # Returns
    ///
    /// A `Config` instance populated with values from the configuration file, or the
    /// default values if the file is not present or cannot be loaded.
    #[must_use]
    pub fn load_file(path: Option<&str>) -> Config {
        // Read TOML file using the toml crate
        let content =
            fs::read_to_string(PathBuf::from(path.unwrap_or(DEFAULT_CONFIG_FILE_PATH))).ok();

        // Parse TOML content into a TomlTable
        let toml_table: Option<toml::value::Table> =
            content.and_then(|c| toml::de::from_str(&c).ok());

        // Extract the "Config" section from the TOML table
        let config = toml_table.and_then(|t| t.get("Settings").cloned());

        // Map the config to the Config struct
        config
            .map(|s| Config {
                tally_dir: s
                    .get("tally_dir")
                    .and_then(|val| val.as_str().map(PathBuf::from))
                    .unwrap_or_else(|| Config::default().tally_dir),
                free_tries: s
                    .get("free_tries")
                    .and_then(toml::Value::as_integer)
                    .map_or_else(|| Config::default().free_tries, |val| val as i32),
                base_delay_seconds: s
                    .get("base_delay_seconds")
                    .and_then(toml::Value::as_integer)
                    .map_or_else(|| Config::default().base_delay_seconds, |val| val as i32),
                ramp_multiplier: s
                    .get("ramp_multiplier")
                    .and_then(toml::Value::as_float)
                    .map_or_else(|| Config::default().ramp_multiplier, |val| val as i32),
                even_deny_root: s
                    .get("even_deny_root")
                    .and_then(toml::Value::as_bool)
                    .unwrap_or_else(|| Config::default().even_deny_root),
            })
            .unwrap_or_default()
    }
}

// Unit Tests
#[cfg(test)]
mod tests {
    use tempdir::TempDir;

    use super::*;

    #[test]
    fn test_default_config() {
        let default_config = Config::default();
        assert_eq!(default_config.tally_dir, PathBuf::from("/var/run/authramp"));
        assert_eq!(default_config.free_tries, 6);
        assert_eq!(default_config.base_delay_seconds, 30);
        assert_eq!(default_config.ramp_multiplier, 50);
        assert!(!default_config.even_deny_root);
    }

    #[test]
    fn test_build_config() {
        let temp_dir = TempDir::new("test_build_settings_from_toml").unwrap();
        let conf_file_path = temp_dir.path().join("config.conf");

        // Create a TOML file with settings
        let toml_content = r#"
        [Settings]
        tally_dir = "/tmp/tally_dir"
        free_tries = 10
        base_delay_seconds = 15
        ramp_multiplier = 20.0
        even_deny_root = true
    "#;
        std::fs::write(&conf_file_path, toml_content).unwrap();

        // Build settings from TOML
        let config = Config::load_file(Some(conf_file_path.to_str().unwrap()));

        // Validate the result
        assert_eq!(config.free_tries, 10);
        assert_eq!(config.base_delay_seconds, 15);
        assert_eq!(config.ramp_multiplier, 20);
        assert!(config.even_deny_root);
    }
}
