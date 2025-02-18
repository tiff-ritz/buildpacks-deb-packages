# Using `PACKAGE_ENV_VARS` in `src/install_packages.rs`

The `PACKAGE_ENV_VARS` constant is a new feature of the Debian Buildpack in `src/install_packages.rs`. This document explains how it works, how to add to the constant, and what happens if a package is skipped. This information is intended for other programmers who might need to extend or maintain the codebase.

## How `PACKAGE_ENV_VARS` Works

`PACKAGE_ENV_VARS` is a constant that defines environment variables required by specific packages. When a package is installed, its associated environment variables are set to ensure the package functions correctly.

### Definition

The `PACKAGE_ENV_VARS` constant is defined as a slice of tuples, where each tuple contains the name of a package and a slice of key-value pairs representing the environment variables and their respective values.

```rust
const PACKAGE_ENV_VARS: &[(&str, &[(&str, &str)])] = &[
    ("git", &[("GIT_EXEC_PATH", "{install_dir}/usr/lib/git-core"), ("GIT_TEMPLATE_DIR", "{install_dir}/usr/share/git-core/templates")]),
    ("ghostscript", &[("GS_LIB", "{install_dir}/var/lib/ghostscript")]),
    // Add more package mappings here
];
```

### Usage

The `package_env_vars` function converts this constant into a `HashMap` for easy lookup. It is used in the `install_packages` function to set environment variables for the installed packages.

```rust
fn package_env_vars() -> HashMap<&'static str, HashMap<&'static str, &'static str>> {
    let mut map = HashMap::new();
    for &(package, vars) in PACKAGE_ENV_VARS.iter() {
        let mut var_map = HashMap::new();
        for &(key, value) in vars.iter() {
            var_map.insert(key, value);
        }
        map.insert(package, var_map);
    }
    map
}
```

During the package installation process, the environment variables from `PACKAGE_ENV_VARS` are applied to the layer environment.

### Applying Environment Variables

After the packages are installed, the environment variables are applied using the `configure_layer_environment` function. This function replaces placeholders like `{install_dir}` with the actual installation directory path.

```rust
let install_dir = install_layer.path().to_string_lossy().to_string();
let package_env_vars: HashMap<String, HashMap<String, String>> = package_env_vars()
    .into_iter()
    .map(|(k, v)| {
        (
            k.to_string(),
            v.into_iter()
                .map(|(k, v)| (k.to_string(), v.replace("{install_dir}", &install_dir)))
                .collect(),
        )
    })
    .collect();
```

## How to Add to `PACKAGE_ENV_VARS`

To add a new package and its environment variables to `PACKAGE_ENV_VARS`, follow these steps:

1. **Identify the Package**: Determine the package name and the required environment variables.

2. **Add an Entry**: Add a new entry to the `PACKAGE_ENV_VARS` constant. The entry should be a tuple where the first element is the package name and the second element is a slice of key-value pairs representing the environment variables.

Example: Adding a new package `example-package` with environment variables `EXAMPLE_VAR1` and `EXAMPLE_VAR2`.

```rust
const PACKAGE_ENV_VARS: &[(&str, &[(&str, &str)])] = &[
    ("git", &[("GIT_EXEC_PATH", "{install_dir}/usr/lib/git-core"), ("GIT_TEMPLATE_DIR", "{install_dir}/usr/share/git-core/templates")]),
    ("ghostscript", &[("GS_LIB", "{install_dir}/var/lib/ghostscript")]),
    ("example-package", &[("EXAMPLE_VAR1", "{install_dir}/usr/lib/example"), ("EXAMPLE_VAR2", "{install_dir}/usr/share/example")]),
];
```

3. **Testing**: Ensure the new package and its environment variables are correctly applied during the installation process by running the relevant tests.

## Handling Skipped Packages

When a package is skipped, its environment variables are still processed and applied if they are listed in the `PACKAGE_ENV_VARS` constant. This ensures that even if the package is not installed, any pre-existing environment variables are correctly set.  To accomplish this `determine_packages_to_install` was modified to return a list of skipped packages.

### Example

If the package `git` is skipped, the environment variables `GIT_EXEC_PATH` and `GIT_TEMPLATE_DIR` will still be set based on the values defined in `PACKAGE_ENV_VARS`.

```rust
for skipped_package in skipped_packages {
    if let Some(vars) = package_env_vars.get(skipped_package.name.as_str()) {
        for (key, value) in vars {
            prepend_to_env_var(&mut layer_env, key, vec![value.to_string()]);
        }
    }
}
```

This approach ensures consistency in the environment configuration, even when certain packages are not installed.

## Conclusion

The `PACKAGE_ENV_VARS` constant is a powerful tool for managing environment variables for specific packages during the installation process. By following the guidelines provided, you can easily add new packages and ensure their environment variables are correctly applied. Additionally, the handling of skipped packages ensures that the environment remains consistent and correctly configured.