# Voxlume

A full-stack audiobook web application built with Rust and Leptos, featuring a Rust-based data ingestion pipeline.

Voxlume is a web application for discovering and managing audiobooks. The backend is built with Rust using the Axum framework, and the frontend is a single-page application compiled to WebAssembly (WASM) using the Leptos framework. The application uses PostgreSQL for storing primary data.

Data is ingested into the application via a separate Rust-based CLI tool. This tool scrapes audiobook information from `audiobookbay.is`, processes it using a Gemini AI model, and adds it to a PostgreSQL message queue (PGMQ) for the main application to consume.

## Getting Started

### Prerequisites

- **Rust:** `nightly`
- **Cargo-Leptos:** `cargo install cargo-leptos`
- **Docker:** for running the databases.

### Building and Running the Application

1.  **Start the databases:**

    ```bash
    docker-compose up -d
    ```

2.  **Run the application:**

    ```bash
    cargo leptos watch
    ```

3.  Open your browser to `http://0.0.0.0:3000`.

### Running the CLI

You can run the CLI using cargo:

```bash
cargo run -p cli -- --help
```

## Project Structure

- `app/`: Contains the main application logic, tying the frontend and server together.
- `frontend/`: The frontend Leptos application (compiled to WASM).
- `server/`: The backend Axum server.
- `shared/`: Rust code shared between the `frontend` and `server`.
- `model/`: Contains the data models, database migrations, and entity definitions.
- `cli/`: A Rust-based data ingestion pipeline and utilities.
- `style/`: SASS/SCSS stylesheets.
- `public/`: Static assets (images, fonts, etc.).

## Features

- **Audiobook Discovery:** Browse and search for audiobooks.
- **User Authentication:** Register, login, and logout functionality.
- **Theme Toggler:** Switch between light and dark modes.
- **Data Ingestion:** Automated pipeline for scraping and processing audiobook data.

## 📄 License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.
