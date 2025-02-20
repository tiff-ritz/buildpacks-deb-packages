# Using `SPECIAL_CASE_MAP` in `src/determine_packages_to_install.rs`

The `SPECIAL_CASE_MAP` constant is a new feature in the `buildpacks-deb-packages` project located in `src/determine_packages_to_install.rs`. This document explains its functionality, how to add entries to it, and its impact on the package installation process.

## How `SPECIAL_CASE_MAP` Works

`SPECIAL_CASE_MAP` is a constant that defines special cases where additional packages should be installed before the requested package. This is useful for handling dependencies that are not automatically resolved by the package manager.

### Definition

The `SPECIAL_CASE_MAP` constant is defined as a slice of tuples, where each tuple contains the name of a special case package and a slice of additional packages that need to be installed with it.

```rust
const SPECIAL_CASE_MAP: &[(&str, &[&str])] = &[
    ("portaudio19-dev", &["libportaudio2"]),
    ("7zip", &["7zip-standalone"]),
    ("enchant-2", &["libenchant-2-dev"]),
    // add more mappings here
];
```

### Usage

The `SPECIAL_CASE_MAP` is converted into a `HashMap` for easy lookup during the package installation process. This conversion happens within the `determine_packages_to_install` function.

```rust
let special_case_map: HashMap<&str, Vec<&str>> = SPECIAL_CASE_MAP
    .iter()
    .cloned()
    .map(|(special, additionals)| (special, additionals.to_vec()))
    .collect();
```

### Applying Special Cases

During the package installation process, the `visit` function checks if a requested package is a special case. If it is, the additional packages specified in `SPECIAL_CASE_MAP` are also marked for installation.

```rust
if let Some(additional_packages) = special_case_map.get(package) {
    for &additional_package in additional_packages {
        if should_visit_dependency(additional_package, system_packages, packages_marked_for_install) {
            visit(
                additional_package,
                skip_dependencies,
                force_if_installed_on_system,
                system_packages,
                package_index,
                packages_marked_for_install,
                visit_stack,
                package_notifications,
                special_case_map,
            )?;
        }
    }
}
```

## How to Add to `SPECIAL_CASE_MAP`

To add a new special case package and its additional packages to `SPECIAL_CASE_MAP`, follow these steps:

1. **Identify the Special Case**: Determine the package name and the additional packages that need to be installed with it.

2. **Add an Entry**: Add a new entry to the `SPECIAL_CASE_MAP` constant. The entry should be a tuple where the first element is the special case package name and the second element is a slice of additional package names.

Example: Adding a new special case package `example-package` with additional packages `example-dependency1` and `example-dependency2`.

```rust
const SPECIAL_CASE_MAP: &[(&str, &[&str])] = &[
    ("portaudio19-dev", &["libportaudio2"]),
    ("7zip", &["7zip-standalone"]),
    ("example-package", &["example-dependency1", "example-dependency2"]),
];
```

3. **Testing**: Ensure the new special case package and its additional packages are correctly applied during the installation process by running the relevant tests.

## Conclusion

The `SPECIAL_CASE_MAP` constant is a powerful tool for managing special case packages that require additional packages to be installed. By following the guidelines provided, you can easily add new special cases and ensure their additional packages are correctly applied. This approach ensures that the package installation process is consistent and correctly configured.