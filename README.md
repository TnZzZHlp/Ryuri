<center>
<h1>
Ryuri
</h1>
</center>

<center>

[[简体中文](https://github.com/tnzzzhlp/ryuri/blob/main/docs/zh-CN.README.md)]

</center>

<div style="text-align: center;">
  <img src="frontend/public/ryuri.svg" alt="Logo" width="100"/>
</div>

Ryuri is a self-hosted comic and manga reader server designed for simplicity and compatibility. It allows you to manage your digital collection, read directly in the browser and sync.

<details>

<summary>Preview</summary>

![alt text](img/image-1.png)
![alt text](img/image-2.png)
![alt text](img/image-3.png)

</details>

## Features

-   **Self-Hosted Library**: Organize your comics, manga, and ebooks (supports zip, rar, cbz, cbr, epub).
-   **Web Reader**: A modern, responsive web interface for reading on any device.
-   **Komga Compatibility**: Implements the Komga API, allowing you to use clients like [Mihon](https://github.com/mihonapp/mihon).
-   **Progress Tracking**: Automatically tracks your reading progress across devices.
-   **Content Scanning**: Efficiently scans your folders to update your library.

## Tech Stack

### Backend

-   **Language**: Rust
-   **Framework**: [Axum](https://github.com/tokio-rs/axum)
-   **Database**: SQLite (via [SQLx](https://github.com/launchbadge/sqlx))
-   **Key Crates**: `tokio`, `tower-http`, `zip`, `rar`, `epub`

### Frontend

-   **Framework**: [Vue.js 3](https://vuejs.org/)
-   **Build Tool**: [Vite](https://vitejs.dev/)
-   **Language**: TypeScript
-   **State Management**: [Pinia](https://pinia.vuejs.org/)
-   **Styling**: [Tailwind CSS](https://tailwindcss.com/)
-   **UI Components**: [shadcn-vue](https://www.shadcn-vue.com/)

## Getting Started

### Prerequisites

-   [Rust](https://www.rust-lang.org/tools/install) (latest stable)
-   [Node.js](https://nodejs.org/) (v18+ recommended)
-   [pnpm](https://pnpm.io/) (optional, but recommended for frontend)

### Development Setup

1.  **Clone the repository:**

    ```bash
    git clone https://github.com/yourusername/Ryuri.git
    cd Ryuri
    ```

2.  **Setup Backend:**

    ```bash
    cd backend
    # Create a .env file if necessary (check .env.example if available)
    cargo run
    ```

    The backend server usually runs on `http://localhost:3000` (check `main.rs` or logs to confirm).

3.  **Setup Frontend:**

    ```bash
    cd frontend
    pnpm install
    pnpm dev
    ```

    The frontend development server will start, usually proxied to the backend.

    The server will start, usually on `http://localhost:3000`.

### Running Binaries from Releases

If you prefer not to build from source, you can download pre-compiled binaries from the GitHub Releases page.

1.  **Download the Binary:**
    Go to the [GitHub Releases page](https://github.com/TnZzZHlp/Wyuri/releases) and download the appropriate binary for your operating system and architecture (e.g., `ryuri-linux-amd64` or `ryuri-windows-amd64.exe`).

2.  **Set Environment Variables and Run:**
    Navigate to the directory where you downloaded the binary. Set the necessary environment variables and run the executable. A webpage will be available immediately upon startup, defaulting to listening on port 3000.

    **Linux/macOS:**

    ```bash
    chmod +x ryuri-linux-amd64
    ./ryuri-linux-amd64
    ```

    **Windows (PowerShell):**

    ```powershell
    .\ryuri-windows-amd64.exe
    ```

    **Important Environment Variables:**

    -   `DATABASE_URL`: Specifies the database connection string, e.g., `sqlite:ryuri.db?mode=rwc`.
    -   `JWT_SECRET`: **Crucial for security.** Replace with a strong, random string.
    -   `HOST`: (Optional) The host address to bind to (default: `0.0.0.0`).
    -   `PORT`: (Optional) The port to listen on (default: `3000`).
    -   `JWT_EXPIRATION_HOURS`: (Optional) How long JWT tokens are valid (default: `24`).

    The server will start, usually on `http://localhost:3000`.

### Building and Running from Source

To run Ryuri directly from the compiled binary, follow these steps:

1.  **Prerequisites:**
    Ensure you have [Rust](https://www.rust-lang.org/tools/install) (latest stable) and [Node.js](https://nodejs.org/) (v18+ recommended) with [pnpm](https://pnpm.io/) installed.

2.  **Clone the repository:**

    ```bash
    git clone https://github.com/yourusername/Ryuri.git
    cd Ryuri
    ```

3.  **Build Backend (Release Mode):**
    First, build the frontend so its assets can be embedded in the backend binary.

    ```bash
    cd frontend
    pnpm install
    pnpm build
    cd ..
    ```

    Then, navigate to the backend directory and build the application in release mode.

    ```bash
    cd backend
    cargo build --release
    cd ..
    ```

    The compiled binary will be located at `backend/target/release/backend`.

4.  **Run the Binary:**
    You need to set environment variables for the database and JWT secret before running the application.

    **Linux/macOS:**

    ```bash
    export DATABASE_URL="sqlite:ryuri.db?mode=rwc"
    export JWT_SECRET="your_secure_random_string_here"
    ./backend/target/release/backend
    ```

    **Windows (PowerShell):**

    ```powershell
    $env:DATABASE_URL="sqlite:ryuri.db?mode=rwc"
    $env:JWT_SECRET="your_secure_random_string_here"
    .\backend\target\release\backend.exe
    ```

    **Important Environment Variables:**

    -   `DATABASE_URL`: Specifies the database connection string. For SQLite, `sqlite:ryuri.db?mode=rwc` will create a `ryuri.db` file in the current directory.
    -   `JWT_SECRET`: **Crucial for security.** Replace `"your_secure_random_string_here"` with a long, random, and unique string. Without a persistent secret, user sessions will be invalidated on every restart.
    -   `HOST`: (Optional) The host address to bind to (default: `0.0.0.0`).
    -   `PORT`: (Optional) The port to listen on (default: `3000`).
    -   `JWT_EXPIRATION_HOURS`: (Optional) How long JWT tokens are valid (default: `24`).

    The server will start, usually on `http://localhost:3000`.

### Deployment (Docker)

You can easily deploy Ryuri using the official Docker image.

1.  **Pull the image:**

    ```bash
    docker pull ghcr.io/tnzzzhlp/wyuri:latest
    ```

    _(Note: Replace `tnzzzhlp/wyuri` with your specific image path if different)_

2.  **Run the container:**

    ```bash
    docker run -d \
      -p 3000:3000 \
      --name ryuri \
      -v ./ryuri-data:/app/data \
      -v /path/to/your/comics:/comics \
      -e DATABASE_URL="sqlite:/app/data/ryuri.db?mode=rwc" \
      -e JWT_SECRET="change-this-to-a-secure-secret" \
      ghcr.io/tnzzzhlp/wyuri:latest
    ```

    **Environment Variables:**

    -   `DATABASE_URL`: Connection string for the database. Ensure it points to a location inside the volume (e.g., `/app/data`).
    -   `JWT_SECRET`: (Optional) A secure random string for signing authentication tokens (default: random string generated on each startup).
    -   `JWT_EXPIRATION_HOURS`: (Optional) Token expiration time in hours (default: 24).

    **Volumes:**

    -   `/app/data`: Persistent storage for the database.
    -   `/comics`: (Example) Mount your local comic directories here to add them to your Ryuri library.

## API Documentation

Ryuri provides a REST API for all frontend operations. Additionally, it exposes a Komga-compatible API layer under `/komga`.

-   **Standard API**: Used by the web frontend for library management, reading, and settings.
-   **Limited Komga API support**: Support the API required for the Komga plugin in the Mihon APP.

## License

[MIT](LICENSE)
