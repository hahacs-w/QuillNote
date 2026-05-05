# QuillNote 📝

QuillNote（羽毛笔记）意喻编辑笔记草稿的过程像羽毛一样优雅飘逸， 是一款基于 **Tauri (v2)** 、 **Vue 3**、**rust** 构建的现代化、轻量级桌面笔记应用程序。它提供了流畅的富文本编辑体验，支持焦点模式、标签和文件夹管理，以及强大的页面内书签和重点标记功能。

## ✨ 功能特性 (Features)

- **📁 强大的组织结构**: 支持无限制的文件夹和标签管理，可通过拖拽轻松移动笔记。
- **📑 子页面 (Sub-pages) 支持**: 在主笔记下可以创建多个子页面，方便管理复杂长篇内容，支持拖拽排序。
- **🎯 专注模式 (Focus Mode)**: 一键隐藏侧边栏和工具栏，沉浸式无干扰写作。
- **🔍 全局与局部搜索**: 
  - 全局搜索：快速检索所有笔记标题和内容。
  - 局部搜索：在当前页面中高亮显示匹配项，支持快捷键（`Ctrl/Cmd + F`）。
- **⭐ 重点高亮与轨迹 (Track & Highlights)**:
  - 选择文本一键标记为“重要（★）”或“轨迹（📍）”。
  - **只显示重要模式**: 过滤掉普通文本，快速回顾笔记核心内容。
- **🔖 快捷书签 (Bookmarks)**: 使用 `Ctrl/Cmd + Shift + [0-9]` 设置书签，`Ctrl/Cmd + [0-9]` 快速跳转。
- **🔗 外链管理**: 支持在笔记中嵌入外部链接或本地文件路径，并集中管理。
- **📌 窗口置顶**: 支持将应用窗口钉在桌面最上层。
- **📤 多格式导出**: 一键将笔记导出为 TXT, HTML, Word (.docx) 甚至是 PDF 格式。

## 🛠️ 技术栈 (Tech Stack)

- **前端 (Frontend)**:
  - [Vue 3](https://vuejs.org/) (Composition API, `<script setup>`)
  - [Vite](https://vitejs.dev/) - 下一代前端构建工具
  - [TypeScript](https://www.typescriptlang.org/)
  - [Tiptap](https://tiptap.dev/) - 强大的 Headless 富文本编辑器
- **桌面端 (Desktop)**:
  - [Tauri v2](https://v2.tauri.app/) - 使用 Web 技术构建更小、更快、更安全的桌面应用
  - [Rust](https://www.rust-lang.org/) - Tauri 后端语言

## 🚀 快速开始 (Getting Started)

### 环境要求 (Prerequisites)

在开始之前，请确保您的机器上已经安装了以下环境：

- [Node.js](https://nodejs.org/) (推荐 v18 或以上版本)
- [Rust](https://www.rust-lang.org/tools/install) (及相关的 C++ 构建工具)
- Tauri 相关的[系统依赖](https://v2.tauri.app/start/prerequisites/)

### 安装与运行 (Installation & Run)

1. **克隆仓库 (Clone the repository)**
   ```bash
   git clone https://github.com/yourusername/quillnote.git
   cd quillnote
   ```

2. **安装前端依赖 (Install dependencies)**
   ```bash
   npm install
   ```

3. **开发模式运行 (Run in development mode)**
   ```bash
   npm run tauri dev
   ```

4. **构建生产版本 (Build for production)**
   ```bash
   npm run tauri build
   ```
   构建完成后，安装包将生成在 `src-tauri/target/release/bundle/` 目录下。

## 📂 项目结构 (Project Structure)

```text
quillnote/
├── src/                  # Vue 3 前端代码
│   ├── components/       # Vue 组件 (Editor, Settings 等)
│   ├── assets/           # 静态资源
│   ├── types.ts          # TypeScript 类型定义
│   ├── App.vue           # 根组件，包含主布局和核心逻辑
│   └── main.ts           # 前端入口文件
├── src-tauri/            # Tauri & Rust 后端代码
│   ├── src/              # Rust 源码 (fs_cmds, db_cmds, config 等)
│   ├── tauri.conf.json   # Tauri 配置文件
│   └── Cargo.toml        # Rust 依赖配置
├── package.json          # Node.js 项目配置
└── vite.config.ts        # Vite 构建配置
```

## 📝 快捷键 (Shortcuts)

| 快捷键 (Shortcut) | 描述 (Description) |
| :--- | :--- |
| `Ctrl/Cmd + F` | 在当前页面打开局部搜索 |
| `Ctrl/Cmd + G` | 局部搜索跳至下一个结果 (`Shift` 上一个) |
| `Ctrl/Cmd + Shift + [0-9]` | 设置页面内书签 |
| `Ctrl/Cmd + [0-9]` | 跳转至对应的书签 |
| `Meta + ←/→` | 在子页面标签之间切换 |

## 📄 许可证 (License)

本项目采用 [MIT License](LICENSE) 开源许可证。