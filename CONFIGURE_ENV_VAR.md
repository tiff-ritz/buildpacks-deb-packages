# Using Configurable Environment Variables in `src/install_packages.rs`

The `project.toml` file plays a crucial role during the build process by specifying environment variables for packages using the `env` key. This document explains how the environment variable handling works, how to add environment variables to a package, and what happens if a package is skipped.

## How It Works

1. **Reading `project.toml`**:
   - The `project.toml` file is read in the `install_packages` function located in `src/install_packages.rs`.
   - The `Environment::load_from_toml` method from `src/config/environment.rs` is used to load environment variables from the `project.toml` file.

2. **Loading Environment Variables**:
   - The `Environment::load_from_toml` method parses the `project.toml` file and extracts environment variables defined in the `env` key of each package entry.
   - The extracted environment variables are stored in a `HashMap` within the `Environment` struct.

3. **Passing Environment Variables**:
   - The `env` object containing the environment variables is passed to the `configure_layer_environment` function.
   - The `Environment::get_variables` method is used to retrieve the environment variables from the `env` object.

4. **Configuring Layer Environment**:
   - The `configure_layer_environment` function applies the environment variables to the build environment using the `LayerEnv` struct.
   - The environment variables are added to the layer environment, ensuring they are available during the build process.

## Adding Environment Variables to a Package

To add environment variables to a package in the `project.toml` file, follow these steps:

1. **Open the `project.toml` file**:
   - Navigate to the root of your project and open the `project.toml` file.

2. **Locate the package entry**:
   - Find the package entry for which you want to add environment variables.

3. **Add the `env` key**:
   - Add the `env` key to the package entry and define the environment variables as key-value pairs.

### Example

```toml
[com.heroku.buildpacks.deb-packages]
install = [
  {
    name = "git",
    env = {
      "GIT_EXEC_PATH" = "{install_dir}/usr/lib/git-core",
      "GIT_TEMPLATE_DIR" = "{install_dir}/usr/share/git-core/templates"
    }
  },
  {
    name = "ghostscript",
    env = {
      "GS_LIB" = "{install_dir}/var/lib/ghostscript"
    }
  }
]
```

In this example:
- The `git` package has two environment variables: `GIT_EXEC_PATH` and `GIT_TEMPLATE_DIR`.
- The `ghostscript` package has one environment variable: `GS_LIB`.

## Handling Skipped Packages

If a package is skipped during the installation process, the environment variables defined for that package will still be processed and applied. This ensures that the environment variables are consistently set, even if the package is not installed.

### Example

If the package `git` is skipped, the environment variables `GIT_EXEC_PATH` and `GIT_TEMPLATE_DIR` will still be set based on the values defined in the `project.toml` file.

```rust
fn configure_layer_environment(
    install_path: &Path,
    multiarch_name: &MultiarchName,
    package_env_vars: &HashMap<String, HashMap<String, String>>,
    packages_to_install: &[RepositoryPackage],
    skipped_packages: &[RequestedPackage],
    env: &Environment,
) -> LayerEnv {

    let mut layer_env = LayerEnv::new();

    let bin_paths = [
        install_path.join("bin"),
        install_path.join("usr/bin"),
        install_path.join("usr/sbin"),
    ];
    prepend_to_env_var(&mut layer_env, "PATH", &bin_paths);

    // Load and apply environment variables from the project.toml file
    for (key, value) in env.get_variables() {
        prepend_to_env_var(&mut layer_env, key, vec![value.clone()]);
    }
    
    // ... rest of function
}
```

This approach ensures consistency in the environment configuration, even when certain packages are not installed.

## Conclusion

The `project.toml` file is essential for configuring environment variables for packages during the build process. By following the steps outlined in this document, you can easily add environment variables to packages and ensure they are correctly applied, even if the package is skipped. This functionality provides flexibility and consistency in managing environment variables for your build process.
```` ▋