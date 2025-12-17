# Wyuri

Wyuri is a self-hosted comic and manga reader server designed for simplicity and compatibility. It allows you to manage your digital collection, read directly in the browser, and sync with external clients.

## Features

- **Self-Hosted Library**: Organize your comics, manga, and ebooks (supports zip, rar, cbz, cbr, epub).
- **Web Reader**: A modern, responsive web interface for reading on any device.
- **Komga Compatibility**: Implements the Komga API, allowing you to use clients like [Mihon](https://github.com/mihonapp/mihon).
- **Progress Tracking**: Automatically tracks your reading progress across devices.
- **Content Scanning**: Efficiently scans your folders to update your library.

## Tech Stack

### Backend
- **Language**: Rust
- **Framework**: [Axum](https://github.com/tokio-rs/axum)
- **Database**: SQLite (via [SQLx](https://github.com/launchbadge/sqlx))
- **Key Crates**: `tokio`, `tower-http`, `zip`, `rar`, `epub`

### Frontend
- **Framework**: [Vue.js 3](https://vuejs.org/)
- **Build Tool**: [Vite](https://vitejs.dev/)
- **Language**: TypeScript
- **State Management**: [Pinia](https://pinia.vuejs.org/)
- **Styling**: [Tailwind CSS](https://tailwindcss.com/)
- **UI Components**: [shadcn-vue](https://www.shadcn-vue.com/)

## Getting Started

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Node.js](https://nodejs.org/) (v18+ recommended)
- [pnpm](https://pnpm.io/) (optional, but recommended for frontend)

### Development Setup

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/yourusername/wyuri.git
    cd wyuri
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

## API Documentation

Wyuri provides a REST API for all frontend operations. Additionally, it exposes a Komga-compatible API layer under `/komga`.

- **Standard API**: Used by the web frontend for library management, reading, and settings.
- **Komga API**: See [KOMGA_API.md](backend/KOMGA_API.md) for details on supported endpoints for third-party clients.

## License

[MIT](LICENSE)
