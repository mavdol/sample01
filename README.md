# Sample01

> An open-source desktop app for generating diverse and realistic synthetic tabular data using local LLMs

## ✨ Features

### Intelligent Data Generation

- **DAG-based Pipeline**: Row generation follows a strict, deterministic pipeline where each column has its own rules
- **Column Dependencies**: Reference other columns in your rules (e.g., `@Column1 + @Column2`)
- **Strict Typing**: Each column enforces a specific type (text, int, float, JSON, etc.) for consistency and quality

### Quality & Diversity

- **Smart Diversification**: Identifies and avoids overused words and patterns
- **Persona Rotation**: Cycles through 5 different writing styles to vary content
- **Local Processing**: All generation happens on your machine using llama.cpp

## 🚀 Getting Started

### Prerequisites

- Node.js
- pnpm
- Rust

### Installation

```bash
pnpm install
pnpm run tauri dev
```

## 📖 How to Use

1. **Download a model** - Choose from compatible LLM models
2. **Create your dataset** - Define columns with their types and rules
3. **Select model & generate** - Pick your model and specify row count
4. **Export** - Download as CSV

## 🛠️ Technical Details

Built with:

- **Inference Engine**: llama.cpp (Rust bindings)
- **Architecture**: Desktop app using Tauri
- **Export Formats**: CSV (more coming soon)

## 🤝 Contributing

Contributions are welcome! This project is perfect for learning and experimentation.

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

- Add export formats (Parquet, JSON, Excel, etc.)
- Optimize inference performance
- Improve diversification algorithms

**🔴 Hard**

- Add Ollama integration
- Support online LLM providers (OpenAI, Anthropic, etc.)
- Implement distributed generation for massive datasets

## 📝 License

[Your license here]

## 💬 Questions?

Open an issue or start a discussion!
