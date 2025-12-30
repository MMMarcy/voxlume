# Project Overview

This is a full-stack audiobook web application named Voxlume. The backend is built with Rust using the Axum framework, and the frontend is a single-page application compiled to WebAssembly (WASM) using the Leptos framework. The application uses PostgreSQL for storing primary data.
Data is ingested through the `cli` binary.

Database schema is available at `model/migrations`, while the rust representation is at `model/entities_lib`.

## Building and Running

### Prerequisites

- **Rust:**
- **Cargo-Leptos:** `cargo install cargo-leptos`
- **Docker or Podman:** for running the databases.

### Building and Running the Application

1.  **Start the databases:**
    The project uses docker compose to manage the databases.

    ```bash
    docker compose up -d
    ```

2.  **Run the application:**

    ```bash
    RUST_LOG=DEBUG cargo leptos watch --hot-reload -- --environment=dev --gemini-api-key=$GEMINI_API_KEY
    ```

```

## Development Conventions

The project is structured as a multi-crate Rust workspace.
The `app` crate contains the main application logic, like views, routing, ui components and such.
The `shared` crate contains code shared between the `app`, `server`, and `cli`.
The `model` crate contains the data models, database migrations, and entity definitions.
```

## Testing

To test, please run the following command:

```bash
cargo test -p cli -p shared
```

## Checking changes

You should run

```bash
cargo check -p features hydrate,ssr
```

to check if any change works.
