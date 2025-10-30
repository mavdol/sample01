# Sample01

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

An open-source desktop app for generating diverse and realistic synthetic tabular data using local LLMs.

Generate tabular data locally with complete privacy and at no cost, while supporting complex relationships and consistency between columns. All processing happens on your machine, no data leaves your computer.

## ✨ Features

### Intelligent Data Generation

- **DAG-based Pipeline**: Row generation follows a strict, deterministic pipeline where each column has its own rules
- **Column Dependencies**: Reference other columns in your rules (e.g., `@firstName @lastName` for full names)
- **Random Value Commands**: Control diversity with `@RANDOM_INT_X` and `@RANDOM_INT_X_Y` in your rules for predictable randomization
- **Strict Typing**: Each column enforces a specific type (text, int, float, JSON, etc.) for consistency and quality

### Quality & Control

- **User-Controlled Diversity**: Use `@RANDOM_INT` commands to inject randomness exactly where you need it
- **Consistent Relationships**: Column references ensure data consistency (e.g., email derived from name)
- **Local Processing**: All generation happens on your machine using llama.cpp - complete privacy, no API costs

## 🚀 Development

### Prerequisites

- Node.js
- pnpm
- Rust

### Installation

```bash
pnpm install
pnpm run tauri dev
```

> **Windows users only**: Before building, you need to install LLVM and CMake for llama-cpp to work:
>
> **Option 1 (Chocolatey):**
>
> ```bash
> choco install llvm cmake -y
> ```
>
> **Option 2 (winget):**
>
> ```bash
> winget install LLVM.LLVM
> winget install Kitware.CMake
> ```

## 📖 How to Use

> The entire flow takes place directly in the desktop app.

```
[ Download a model ] → [ Create your dataset ] → [ Select model & generate ] → [ Export to csv ]
```

## ⚠️ Limitations

- **Hardware Requirements**: Local model inference is resource-intensive. Faster generation requires more VRAM.
- **GPU Layer Allocation**: You can control the number of GPU layers allocated. Note that improper configuration may cause instability.
- **Performance**: Generation speed depends heavily on your hardware capabilities.

## 🤝 Contributing

Contributions are welcome! This project is perfect for experimentation.

### Getting Started

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Make your changes and add tests
4. Run the test suite: `cargo test`
5. Submit a pull request with a clear description

### 💡 Contribution Ideas

**🟢 Easy**

- Create or update UI themes
- Add locale support (French, Spanish, Chinese, etc.)
- Add more llama.cpp compatible models (see `src/assets/models/*.md`)

**🟡 Intermediate**

- Add more export formats (Parquet, JSON, etc.)
- Add more random command types (`@RANDOM_FLOAT`, `@RANDOM_DATE`, etc.)

**🔴 Hard**

- Add Ollama integration
- Support online LLM providers (OpenAI, Anthropic, etc.)
- Optimize inference performance

## 🙏 Credits

This project is built on top of excellent open-source tools:

- **Inference Engine**: [llama-cpp-2](https://github.com/utilityai/llama-cpp-rs) - Rust bindings for llama.cpp
- **Framework**: [Tauri](https://github.com/tauri-apps/tauri) - Build cross-platform desktop apps
- **UI**: React + TypeScript with UnoCSS

## 📝 License

This project is dual-licensed under:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

You may choose either license for your use.

## 💬 Questions?

Open an issue or start a discussion!
