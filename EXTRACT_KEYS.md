# Using `extract_keys.sh` in `scripts/extract_keys.sh`

The `extract_keys.sh` script is a utility in the `buildpacks-deb-packages` project. This document explains its functionality, how to use it, and its expected output.

## How `extract_keys.sh` Works

The `extract_keys.sh` script extracts the PGP certificate stored in the Apt keyring as ASCII-armored text for each target architecture and distribution specified in the `buildpack.toml` file.

### Definition

The `extract_keys.sh` script performs the following steps:
1. Iterates through each target architecture and distribution specified in `buildpack.toml`.
2. Runs a Docker container for each target and extracts the PGP key.
3. Saves the extracted key with an MD5 signature to identify differences.

### Usage

To use the `extract_keys.sh` script, run the following command:

```shell
bash scripts/extract_keys.sh
```

Ensure that Docker is installed and running on your system.

### Example Output

```shell
Extracting keyring from ubuntu:22.04 (linux/amd64)...
Saved to /path/to/keys/ubuntu_22.04_linux_amd64.<checksum>.asc
```

## How to Use `extract_keys.sh`

1. **Ensure Docker is Running**: Make sure Docker is installed and running on your system.
2. **Run the Script**: Execute the `extract_keys.sh` script.
3. **Check the Output**: The extracted keys will be saved in the `keys` directory with MD5 signatures.
