# Using `download_package_indices.sh` in `scripts/download_package_indices.sh`

The `download_package_indices.sh` script checks out the contents of the package indices for a given distribution.

## How `download_package_indices.sh` Works

The `download_package_indices.sh` script downloads and extracts package indices for a specified distribution and architecture.

### Definition

The `download_package_indices.sh` script performs the following steps:
1. Downloads the package indices for the specified distribution and architecture.
2. Extracts the indices using the `lz4` compression tool.
3. Saves the extracted indices for inspection.

### Usage

To use the `download_package_indices.sh` script, provide the distribution and architecture as arguments:

```shell
bash scripts/download_package_indices.sh <jammy | noble> <linux/arm64 | linux/amd64>
```

### Example Usage

```shell
bash scripts/download_package_indices.sh jammy linux/amd64
```

### Example Output

```shell
Extracting package indices from ubuntu:22.04...
Saved to /path/to/target/tmp/package_indices/ubuntu_22.04_linux_amd64
```

## How to Use `download_package_indices.sh`

1. **Specify Arguments**: Provide the distribution and architecture as arguments.
2. **Run the Script**: Execute the `download_package_indices.sh` script.
3. **Check the Output**: The extracted package indices will be saved in the specified directory.