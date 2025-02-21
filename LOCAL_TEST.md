## Testing Locally

To test the project locally, follow these steps:

### Prerequisites

Ensure you have the following installed:
- Rust and Cargo: [Installation Guide](https://www.rust-lang.org/tools/install)
- Docker (if applicable for your tests)
- `cargo install libcnb-cargo`

### Building the Project

- `cargo build` to build the project
- `cargo test` to run the automated tests
- `cargo test --test integration_test` to run integration tests
- `cargo libcnb package` builds an image of the buildpack that can be used with an application.  The output of this command shows usage of the generated image.