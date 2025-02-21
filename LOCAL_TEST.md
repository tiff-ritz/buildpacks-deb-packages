# Local Testing Guide

Follow these steps to test the project locally:

## Prerequisites

Ensure the following tools are installed:
- [Rust and Cargo](https://www.rust-lang.org/tools/install)
- Docker (if applicable for your tests)
- `libcnb-cargo`: Install using `cargo install libcnb-cargo`

## Building the Project

- Run `cargo build` to build the project.
- Run `cargo test` to execute automated tests.
- Run `cargo test --test integration_test` to execute integration tests.
- Run `cargo libcnb package` to build an image of the buildpack. The output will show how to use the generated image.

## Helpful scripts

- [scripts/inspect_package.sh](INSPECT_PACKAGE.md)
- [scripts/extract_keys.sh](EXTRACT_KEYS.md)
- [scripts/download_package_indicies.sh](PACKAGE_INDICIES.md)