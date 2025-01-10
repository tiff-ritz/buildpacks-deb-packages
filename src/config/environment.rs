use std::collections::HashMap;
use std::fs;
use std::path::Path;
use toml_edit::{DocumentMut, Value};

#[derive(Debug, Default)]
pub(crate) struct Environment {
    variables: HashMap<String, String>,
    commands: HashMap<String, Vec<String>>,
}

impl Environment {
    pub(crate) fn new(variables: HashMap<String, String>, commands: HashMap<String, Vec<String>>) -> Self {
        Environment { variables, commands }
    }

    /// Load environment variables from the project.toml file based on package names.
    pub(crate) fn load_from_toml(file_path: &Path, install_dir: &str) -> Self {
        let mut env = Environment::default();
        if let Ok(contents) = fs::read_to_string(file_path) {
            let doc = contents.parse::<DocumentMut>().unwrap();
            if let Some(array_of_tables) = doc
                .as_table()
                .get("com")
                .and_then(|item| item.as_table()?.get("heroku"))
                .and_then(|item| item.as_table()?.get("buildpacks"))
                .and_then(|item| item.as_table()?.get("deb-packages"))
                .and_then(|item| item.as_table()?.get("install"))
                .and_then(|item| item.as_array())
            {
                for table in array_of_tables.iter() {
                    if let Some(env_table) = table
                        .as_inline_table()
                        .and_then(|t| t.get("env"))
                        .and_then(|e| e.as_inline_table())
                    {
                        for (key, value) in env_table.iter() {
                            if let Some(value_str) = value.as_str() {
                                let value_with_install_dir = value_str.replace("{install_dir}", install_dir);
                                env.variables.insert(key.to_string(), value_with_install_dir);
                            }
                        }
                    }

                    // New code to load commands
                    if let Some(commands_array) = table
                        .as_inline_table()
                        .and_then(|t| t.get("commands"))
                        .and_then(|c| c.as_array())
                    {
                        let package_name = table
                            .as_inline_table()
                            .and_then(|t| t.get("name"))
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string();
                        
                        let commands: Vec<String> = commands_array
                            .iter()
                            .filter_map(Value::as_str)
                            .map(String::from)
                            .collect();

                        env.commands.insert(package_name, commands);
                    }                    
                }
            }
        }
        env
    }

   /// Apply environment variables to the current process.
    pub(crate) fn apply(&self) {
        for (key, value) in &self.variables {
            std::env::set_var(key, value);
        }
    }

    /// Check if a package is present in the environment configuration
    pub(crate) fn has_package(&self, package_name: &str) -> bool {
        self.commands.contains_key(package_name)
    }

    /// Get environment variables as a `HashMap`.
    pub(crate) fn get_variables(&self) -> &HashMap<String, String> {
        &self.variables
    }

    /// Get commands as a `HashMap`.
    pub(crate) fn get_commands(&self) -> &HashMap<String, Vec<String>> {
        &self.commands
    }    
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::env;
    use tempfile::tempdir;
    // use std::collections::HashMap;

    #[test]
    fn test_load_from_toml() {
        let toml_content = r#"
            schema-version = "0.2"

            [com.heroku.buildpacks.deb-packages]
            install = [
                { name = "git", env = { "GIT_EXEC_PATH" = "{install_dir}/usr/lib/git-core", "GIT_TEMPLATE_DIR" = "{install_dir}/usr/lib/git-core/templates" }, commands = ["echo 'Git installed successfully'", "git --version"] },
                { name = "babeld" },
                { name = "ghostscript", skip_dependencies = true, force = true, env = { "GS_LIB" = "{install_dir}/var/lib/ghostscript", "GS_FONTPATH" = "{install_dir}/var/lib/ghostscript/fonts" }, commands = ["echo 'Ghostscript installed successfully'", "gs --version"] },
            ]
        "#;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("project.toml");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(toml_content.as_bytes()).unwrap();

        let env = Environment::load_from_toml(&file_path, "/build");
        let variables = env.get_variables();
        let commands = env.get_commands();

        assert_eq!(variables.get("GIT_EXEC_PATH"), Some(&"/build/usr/lib/git-core".to_string()));
        assert_eq!(variables.get("GIT_TEMPLATE_DIR"), Some(&"/build/usr/lib/git-core/templates".to_string()));
        assert_eq!(variables.get("GS_LIB"), Some(&"/build/var/lib/ghostscript".to_string()));
        assert_eq!(variables.get("GS_FONTPATH"), Some(&"/build/var/lib/ghostscript/fonts".to_string()));

        // Verify commands
        let mut all_commands: Vec<String> = commands.values().cloned().flatten().collect();
        all_commands.sort();
        let mut expected_commands = vec![
            "echo 'Git installed successfully'".to_string(),
            "git --version".to_string(),
            "echo 'Ghostscript installed successfully'".to_string(),
            "gs --version".to_string(),
        ];
        expected_commands.sort();
        assert_eq!(all_commands, expected_commands);
    }

    #[test]
    fn test_load_from_toml_invalid() {
        let toml_content = r#"
            schema-version = "0.2"

            [com.heroku.buildpacks.deb-packages]
            install = [
                { name = 123 },
            ]
        "#;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("project.toml");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(toml_content.as_bytes()).unwrap();

        let env = Environment::load_from_toml(&file_path, "/build");
        let variables = env.get_variables();
        let commands = env.get_commands();

        // Verify that no variables or commands are loaded
        assert!(variables.is_empty());
        assert!(commands.is_empty());
    }

    #[test]
    fn test_apply() {
        let mut env = Environment::default();
        env.variables.insert("TEST_VAR".to_string(), "test_value".to_string());
        env.apply();

        assert_eq!(env::var("TEST_VAR").unwrap(), "test_value");
    }

    #[test]
    fn test_get_variables() {
        let mut env = Environment::default();
        env.variables.insert("VAR1".to_string(), "value1".to_string());
        env.variables.insert("VAR2".to_string(), "value2".to_string());

        let variables = env.get_variables();
        assert_eq!(variables.get("VAR1"), Some(&"value1".to_string()));
        assert_eq!(variables.get("VAR2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_get_commands() {
        let toml_content = r#"
            schema-version = "0.2"

            [com.heroku.buildpacks.deb-packages]
            install = [
                { name = "git", commands = ["echo 'Git installed successfully'", "git --version"]},
                { name = "babeld" },
                { name = "ghostscript", skip_dependencies = true, force = true, commands = ["echo 'Ghostscript installed successfully'", "gs --version"]},
            ]
        "#;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("project.toml");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(toml_content.as_bytes()).unwrap();

        let env = Environment::load_from_toml(&file_path, "/build");
        let commands = env.get_commands();

        // Extract the values from the commands HashMap and flatten them into a Vec
        let mut all_commands: Vec<String> = commands.values().cloned().flatten().collect();
        all_commands.sort();

        let mut expected_commands = vec![
            "echo 'Git installed successfully'".to_string(),
            "git --version".to_string(),
            "echo 'Ghostscript installed successfully'".to_string(),
            "gs --version".to_string(),
        ];
        expected_commands.sort();

        assert_eq!(all_commands, expected_commands);
    }

    #[test]
    fn test_has_package() {
        let toml_content = r#"
            schema-version = "0.2"

            [com.heroku.buildpacks.deb-packages]
            install = [
                { name = "git", commands = ["echo 'Git installed successfully'", "git --version"] },
                { name = "babeld" },
                { name = "ghostscript", skip_dependencies = true, force = true, commands = ["echo 'Ghostscript installed successfully'", "gs --version"] }
            ]
        "#;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("project.toml");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(toml_content.as_bytes()).unwrap();

        let env = Environment::load_from_toml(&file_path, "/build");

        assert!(env.has_package("git"));
        assert!(env.has_package("ghostscript"));
        assert!(!env.has_package("nonexistent-package"));
    }    
}
