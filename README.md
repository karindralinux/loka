# Loka

A modern, lightweight, and high-performance PostgreSQL client built for developers who value speed and aesthetics.

Loka combines the power of **Rust** and **Svelte 5** to provide a seamless database management experience within a native desktop environment.

## ✨ Features

- **🚀 Native Performance**: Built with Tauri and Rust for a footprint that's as small as it is fast.
- **⚡ Reactive UI**: Powered by Svelte 5 runes for a truly fluid and responsive interface.
- **📊 Intelligent Data Grid**:
  - Full CRUD support directly from the grid.
  - **Staging Workflow**: Batch your edits and review changes before committing to the database.
  - Column resizing and persistent layout.
  - Efficient pagination for large datasets.
- **📁 Workspace Management**:
  - Multi-tab support to work on multiple tables simultaneously.
  - Schema-aware table browsing with real-time filtering.
  - Primary key detection for safe data manipulation.
- **🎨 Premium Aesthetics**: A meticulously crafted dark theme with vibrant accents and smooth animations.

## 🛠 Tech Stack

- **Frontend**: [Svelte 5](https://svelte.dev/) (Runes), [Vite](https://vitejs.dev/)
- **Desktop Bridge**: [Tauri 2.0](https://tauri.app/)
- **Backend**: [Rust](https://www.rust-lang.org/)
- **Database**: [PostgreSQL](https://www.postgresql.org/)

## 🚀 Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v20+)
- [Rust](https://www.rust-lang.org/tools/install)
- [npm](https://www.npmjs.com/) / [pnpm](https://pnpm.io/)

### Development

1. **Install dependencies**:
   ```bash
   npm install
   ```

2. **Run in development mode**:
   ```bash
   npm run tauri dev
   ```

### Building

To create a production-ready desktop application:

```bash
npm run tauri build
```

## 🗺 Roadmap

Loka aims to become a universal database client. Future goals include:

- [ ] **Expanded Database Support**:
  - [ ] **MySQL / MariaDB** integration.
  - [ ] **SQL Server (MSSQL)** support.
  - [ ] **SQLite** for local development.
  - [ ] **Redis** key-value browsing.
- [ ] **SQL Workbench**: A robust editor for custom queries with syntax highlighting and autocomplete.
- [ ] **Data Visualization**: Generate ER diagrams and schema maps.
- [ ] **Import/Export**: Support for CSV, JSON, and SQL dump formats.

## 📜 License

MIT
