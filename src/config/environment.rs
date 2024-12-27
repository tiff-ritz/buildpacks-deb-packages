use std::collections::HashMap;
use std::fs;
use std::path::Path;
use toml_edit::{DocumentMut};

#[derive(Debug, Default)]
pub(crate) struct Environment {
    variables: HashMap<String, String>,
}

impl Environment {
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
    
    /// Get environment variables as a `HashMap`.
    pub(crate) fn get_variables(&self) -> &HashMap<String, String> {
        &self.variables
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_load_from_toml() {
        let toml_content = r#"
            schema-version = "0.2"

            [com.heroku.buildpacks.deb-packages]
            install = [
                { name = "git", env = { "GIT_EXEC_PATH" = "{install_dir}/usr/lib/git-core", "GIT_TEMPLATE_DIR" = "{install_dir}/usr/lib/git-core/templates" } },
                { name = "babeld" },
                { name = "ghostscript", skip_dependencies = true, force = true, env = { "GS_LIB" = "{install_dir}/var/lib/ghostscript", "GS_FONTPATH" = "{install_dir}/var/lib/ghostscript/fonts" } },
            ]
        "#;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("project.toml");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(toml_content.as_bytes()).unwrap();

        let env = Environment::load_from_toml(&file_path, "/build");
        let variables = env.get_variables();

        // Print the values of the variables
        // println!("GIT_EXEC_PATH: {:?}", variables.get("GIT_EXEC_PATH"));
        // println!("GIT_TEMPLATE_DIR: {:?}", variables.get("GIT_TEMPLATE_DIR"));
        // println!("GS_LIB: {:?}", variables.get("GS_LIB"));
        // println!("GS_FONTPATH: {:?}", variables.get("GS_FONTPATH"));

        assert_eq!(variables.get("GIT_EXEC_PATH"), Some(&"/build/usr/lib/git-core".to_string()));
        assert_eq!(variables.get("GIT_TEMPLATE_DIR"), Some(&"/build/usr/lib/git-core/templates".to_string()));
        assert_eq!(variables.get("GS_LIB"), Some(&"/build/var/lib/ghostscript".to_string()));
        assert_eq!(variables.get("GS_FONTPATH"), Some(&"/build/var/lib/ghostscript/fonts".to_string()));
    }

    #[test]
    fn test_apply() {
        let mut env = Environment::default();
        env.variables.insert("TEST_VAR".to_string(), "test_value".to_string());
        env.apply();

        assert_eq!(env::var("TEST_VAR").unwrap(), "test_value");
    }
}
