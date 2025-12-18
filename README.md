# greego

**greego** is an AI-native graph knowledge base designed for seamless information organization and workflow automation. Built with a high-performance backend and a modern, interactive graph UI, greego allows users to visualize connections between ideas, chat nodes, and external resources.

## 🚀 Features

-   **Interactive Graph UI**: Powered by [React Flow](https://reactflow.dev/), providing a fluid and intuitive canvas for organizing nodes.
-   **AI-Native Workflow**: Specialized node types for chat-based interactions, allowing for a conversational approach to knowledge management.
-   **High Performance**: A background event loop in Rust handles complex database operations concurrently, ensuring a lag-free UI experience.
-   **Embedded SurrealDB**: Uses [SurrealDB](https://surrealdb.com/) for multi-model graph data storage, supporting both in-memory and persistent modes.
-   **Modern Aesthetics**: A premium, dark-mode-first design with Tailwind CSS, featuring subtle animations and a clean workspace.

## 🛠️ Tech Stack

-   **Frontend**: React, TypeScript, Vite, Tailwind CSS, Zustand, @xyflow/react.
-   **Backend**: Tauri (v2), Rust, Tokio (Async Runtime).
-   **Database**: SurrealDB (Embedded).

## 📂 Project Structure

-   `src/`: React frontend application.
    -   `components/`: UI components like `CanvasBoard`, `Sidebar`, and specialized `Nodes`.
    -   `services/`: Backend API integration via Tauri's `invoke`.
-   `src-tauri/`: Rust backend application.
    -   `src/db/`: SurrealDB schema and high-performance event loop implementation.
    -   `src/commands/`: Tauri command handlers.

## 🏁 Getting Started

### Prerequisites

-   [Node.js](https://nodejs.org/) (v18+)
-   [Rust](https://www.rust-lang.org/) (latest stable)
-   [Tauri Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)

### Installation

1.  Clone the repository:
    ```bash
    git clone https://github.com/yourusername/greego.git
    cd greego
    ```

2.  Install dependencies:
    ```bash
    npm install
    ```

3.  Run in development mode:
    ```bash
    npm run tauri dev
    ```

### Building

To build the production-ready application:
```bash
npm run tauri build
```

## 📄 License

This project is licensed under the [MIT License](LICENSE).
