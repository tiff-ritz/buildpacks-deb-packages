# Using `inspect_package.sh` in `scripts/inspect_package.sh`

The `inspect_package.sh` script is a utility for inspecting the contents of a Debian package for a given distribution and architecture.

## How `inspect_package.sh` Works

The `inspect_package.sh` script downloads and extracts the contents of a specified Debian package for inspection.

### Definition

The `inspect_package.sh` script performs the following steps:
1. Downloads the specified Debian package.
2. Extracts the package contents into a temporary directory.
3. Saves the extracted contents for inspection.

### Usage

To use the `inspect_package.sh` script, provide the package name, distribution, and architecture as arguments:

```shell
bash scripts/inspect_package.sh <package_name> <jammy | noble> <linux/arm64 | linux/amd64>
```

### Example Usage

```shell
bash scripts/inspect_package.sh wget jammy linux/amd64
```

### Example Output

```shell
Downloading Debian package wget from ubuntu:22.04...
Saved to /path/to/target/tmp/debian_packages/ubuntu_22.04_linux_amd64/wget
```

## How to Use `inspect_package.sh`

1. **Specify Arguments**: Provide the package name, distribution, and architecture as arguments.
2. **Run the Script**: Execute the `inspect_package.sh` script.
3. **Check the Output**: The extracted package contents will be saved in the specified directory.

---

# Using `scan_ubuntu_repo.sh` in `scripts/scan_ubuntu_repo.sh`

The `scan_ubuntu_repo.sh` script analyzes Debian packages in the Ubuntu repository for post-installation scripts.

## How `scan_ubuntu_repo.sh` Works

The `scan_ubuntu_repo.sh` script recursively scans the Ubuntu repository, downloads Debian packages, and checks for post-installation scripts.

### Definition

The `scan_ubuntu_repo.sh` script performs the following steps:
1. Scans the Ubuntu repository for Debian packages.
2. Downloads each package and extracts its control.tar.gz file.
3. Checks for post-installation scripts and logs the results.

### Usage

To use the `scan_ubuntu_repo.sh` script, run the following command:

```shell
bash scripts/scan_ubuntu_repo.sh
```

### Example Output

```shell
Analyzing wget_1.20.3-1ubuntu1_amd64.deb...
Saved to /path/to/results/packages_with_scripts.txt
```

## How to Use `scan_ubuntu_repo.sh`

1. **Run the Script**: Execute the `scan_ubuntu_repo.sh` script.
2. **Check the Output**: The analysis results will be saved in the `results` directory.
