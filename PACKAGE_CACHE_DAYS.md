# Using `PACKAGE_CACHE_DAYS` Environment Variable in `buildpacks-deb-packages`

The `PACKAGE_CACHE_DAYS` environment variable is a configurable option in the `buildpacks-deb-packages` project. This document explains its functionality, what it does, and how to use it effectively.

## How `PACKAGE_CACHE_DAYS` Works

The `PACKAGE_CACHE_DAYS` environment variable defines the number of days for which package cache is considered valid. This is useful for controlling how frequently the package cache is refreshed, ensuring that the most up-to-date packages are used during the build process.

### Definition

`PACKAGE_CACHE_DAYS` is an environment variable that specifies the number of days to retain the package cache before it is considered stale and needs to be reloaded.

```rust
pub(crate) fn get_package_cache_days() -> u64 {
    Env::from_current()
        .get("PACKAGE_CACHE_DAYS")
        .and_then(|value| value.to_str().and_then(|s| s.parse::<u64>().ok()))
        .unwrap_or(7)
}
```

### Usage

By default, the package cache is considered valid for 7 days. You can override this default behavior by setting the `PACKAGE_CACHE_DAYS` environment variable to a different number of days.

#### Setting `PACKAGE_CACHE_DAYS`

To set the `PACKAGE_CACHE_DAYS` environment variable, you can define it in your build environment. For example, you can set it in your shell before running the build command:

```shell
export PACKAGE_CACHE_DAYS=14
```

Alternatively, you can set it directly in the `project.toml` file under the `[env]` section if your build system supports it:

```toml
[env]
PACKAGE_CACHE_DAYS = "14"
```

### Forcing Cache Reload

To force the cache to be reloaded, you can set the `PACKAGE_CACHE_DAYS` environment variable to `0`. This will invalidate the cache immediately, causing the build process to refresh the package cache.

#### Example

```shell
export PACKAGE_CACHE_DAYS=0
```

By setting `PACKAGE_CACHE_DAYS` to `0`, the cache will be considered stale, and the build process will fetch the latest packages from the repository.

## How to Use `PACKAGE_CACHE_DAYS`

1. **Determine the Cache Duration**: Decide how long you want the package cache to be valid. The default is 7 days, but you can set it to any number of days that suits your needs.

2. **Set the Environment Variable**: Set the `PACKAGE_CACHE_DAYS` environment variable to the desired number of days. This can be done in your shell or in your `project.toml` file.

3. **Force Cache Reload (Optional)**: If you need to force the cache to be reloaded, set the `PACKAGE_CACHE_DAYS` environment variable to `0`.

### Example Scenarios

- **Default Cache Duration**: If you are okay with the default 7-day cache duration, you do not need to set the `PACKAGE_CACHE_DAYS` environment variable.
- **Custom Cache Duration**: If you want the cache to be valid for 14 days, set `PACKAGE_CACHE_DAYS` to `14`.
- **Immediate Cache Invalidation**: To force a cache reload, set `PACKAGE_CACHE_DAYS` to `0`.

## Conclusion

The `PACKAGE_CACHE_DAYS` environment variable provides flexibility in managing the package cache duration. By adjusting this variable, you can control how frequently the package cache is refreshed, ensuring that your build process uses the most up-to-date packages.
