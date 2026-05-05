# Product Requirements Document (PRD): Rust Local Draft App

## 1. Overview
A fast, lightweight desktop draft/note-taking application designed for quick capture of plain text, markdown, and code snippets. The application will run in the background as a system tray utility and can be instantly summoned via a global keyboard shortcut.

## 2. Tech Stack
* **Core/Backend:** Rust
* **Framework:** Tauri
* **Frontend:** Vue 3 (TypeScript)
* **Styling:** Vanilla CSS (Focus on rich aesthetics and consistent spacing without utility overhead)
* **Storage:** Local File System

## 3. Core Features

### 3.1 Interface & Access
* **GUI via Tauri:** A clean, modern graphical user interface using Vue 3.
* **Tray Application:** Runs quietly in the system tray.
* **Global Hotkey:** Can be summoned or dismissed instantly from anywhere in the OS using a configurable keyboard shortcut (e.g., `Cmd/Ctrl + Space`).
* **Default View (Instant Scratchpad):** Activating the hotkey immediately presents a clean, empty scratchpad ready for typing, minimizing friction between thought and capture.

### 3.2 Content & Editor
* **Supported Formats:** Plain Text, Markdown, and Code Snippets.
* **Editor Core:** A robust, styled `textarea` for maximum performance and simplicity during the initial build, to be iterated upon later.
* **Editor Features:** Fast rendering; basic markdown formatting shortcuts.

### 3.3 Storage & Syncing
* **Local Files First:** All notes are saved as raw `.md` or `.txt` files in a designated local directory.
* **No Vendor Lock-in:** Data is easily readable by any other text editor.
* **External Syncing:** The app does not handle syncing natively. Users will place the data directory inside a cloud sync folder (e.g., iCloud, Dropbox, OneDrive) for cross-device backup and sync.

### 3.4 Organization & Search
* **Hybrid Organization:**
  * **Folders:** Traditional file-tree hierarchy reflecting the local directory structure.
  * **Tags:** Support for inline tags (e.g., `#idea`, `#todo`) or frontmatter metadata.
  * **Scratchpad Mode:** The default quick-entry mode for transient thoughts before they are organized into files/folders.
* **Full-Text Search:** High-performance search that scans the contents of all files in the designated directory, not just filenames.

## 4. Future Considerations / Phase 2
* Upgrading the editor core (e.g., CodeMirror) for syntax highlighting and live markdown preview.
* Customizing the frontend theme (Light/Dark mode).
* Advanced markdown plugins (Math, Mermaid charts).
* Export options (PDF, HTML).
