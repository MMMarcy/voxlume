# Voxlume

A full-stack audiobook web application built with Rust and Leptos, featuring a Python-based data ingestion pipeline.

Voxlume is a web application for discovering and managing audiobooks. The backend is built with Rust using the Axum framework, and the frontend is a single-page application compiled to WebAssembly (WASM) using the Leptos framework. The application uses PostgreSQL for storing primary data and Neo4j for graph-based data relationships.

Data is ingested into the application via a separate Python-based CLI tool. This tool scrapes audiobook information from `audiobookbay.is`, processes it using a Gemini AI model, and adds it to a PostgreSQL message queue (PGMQ) for the main application to consume.

## 🚀 Getting Started

### Prerequisites

- **Rust:** `nightly-2025-05-04`
- **Cargo-Leptos:** `cargo install cargo-leptos`
- **Docker:** for running the databases.
- **Python 3.11+**
- **uv:** `pip install uv`

### Building and Running the Application

1.  **Start the databases:**
    ```bash
    docker-compose up -d
    ```
    (Note: A `docker-compose.yml` file may need to be created from the terraform configuration. See `terraform/` directory.)

2.  **Run the application:**
    ```bash
    cargo leptos watch
    ```

3.  Open your browser to `http://0.0.0.0:3000`.

### Running the Python CLI

1. **Install dependencies:**
   ```bash
   cd python_cli
   uv pip install -r requirements.txt
   ```
   (Note: a `requirements.txt` may need to be generated from `pyproject.toml`)

2. **Run the CLI:**
   ```bash
   python src/python_cli/main.py --backfill_job=yes
   ```

## 📂 Project Structure

- `app/`: Contains the main application logic, tying the frontend and server together.
- `frontend/`: The frontend Leptos application (compiled to WASM).
- `server/`: The backend Axum server.
- `shared/`: Rust code shared between the `frontend` and `server`.
- `model/`: Contains the data models, database migrations, and entity definitions.
- `python_cli/`: A Python-based data ingestion pipeline.
- `style/`: SASS/SCSS stylesheets.
- `public/`: Static assets (images, fonts, etc.).
- `terraform/`: Infrastructure as Code (IaC) for managing cloud resources.
- `obsidian/`: Project notes and documentation.

## 💻 Technology Stack

- **Framework:** [Leptos](https://leptos.dev/)
- **Backend:** [Axum](https://github.com/tokio-rs/axum)
- **Frontend:** Rust compiled to [WebAssembly](https://webassembly.org/)
- **Databases:**
  - [PostgreSQL](https://www.postgresql.org/)
  - [Neo4j](https://neo4j.com/)
- **Styling:** [SASS](https://sass-lang.com/) with [Bulma](https://bulma.io/)
- **Infrastructure:** [Terraform](https://www.terraform.io/)
- **AI:** [Google Gemini](https://ai.google.dev/)

## ✨ Features

- **Audiobook Discovery:** Browse and search for audiobooks.
- **User Authentication:** Register, login, and logout functionality.
- **Theme Toggler:** Switch between light and dark modes.
- **Data Ingestion:** Automated pipeline for scraping and processing audiobook data.
