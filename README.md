# <img src="public/favicon.svg" width="38" height="38" align="center" style="margin-right: 8px;" /> VillFlow 2.0

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Tauri](https://img.shields.io/badge/Tauri-v2-FFC131?logo=tauri&logoColor=white)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-v19-61DAFB?logo=react&logoColor=black)](https://react.dev/)
[![TypeScript](https://img.shields.io/badge/TypeScript-v5-3178C6?logo=typescript&logoColor=white)](https://www.typescriptlang.org/)
[![Tailwind CSS](https://img.shields.io/badge/Tailwind_CSS-v4-06B6D4?logo=tailwindcss&logoColor=white)](https://tailwindcss.com/)

**VillFlow 2.0** is a premium, lightweight, global push-to-talk voice assistant designed for Windows. It empowers you to stream your voice, transcribe it instantly via **Speechmatics**, clean it up or process it intelligently using **Groq LLMs**, and automatically insert the result or execute actions—all with a single hotkey press from anywhere in the operating system.

---

### 🌐 Live Download Page

Visit our live hosted download page to get the latest pre-compiled installers for Windows:
👉 **[VillFlow Download Center](https://sreekargpalli.github.io/VillFlow2.0/)**

---

## ✨ Key Features

- **🎙️ Global Push-to-Talk**: Press a custom hotkey anywhere in Windows to record, transcribe, and automatically type out your spoken thoughts.
- **⚡ Real-Time STT (Speechmatics)**: Leverages Speechmatics' low-latency API for ultra-accurate speech recognition.
- **🧠 Smart Processing (Groq)**: Automatically formats, cleans up grammar, or executes instructions on your transcription using Groq's high-speed LLMs (e.g. LLaMA 3.3).
- **💬 Elegant Status Overlay**: A non-intrusive, hardware-accelerated, transparent overlay pill that floats on screen to give real-time feedback (`Recording...`, `Processing...`, `Done!`, `Error`).
- **🎛️ Settings Dashboard**: A sleek, dark-mode dashboard to control:
  - **General Settings**: Auto-start, notifications, active modes.
  - **Global Hotkeys**: Custom keyboard shortcut mappings.
  - **Audio Input**: Select input devices with real-time level monitoring.
  - **API Keys**: Vault-encrypted key storage.
  - **Prompts**: Customizable LLM prompt presets.
- **🔒 Encrypted & Private**: Sensitive API credentials are securely stored using Windows Credential Manager.

---

## 🛠️ Tech Stack

- **Frontend**: [React 19](https://react.dev/), [TypeScript](https://www.typescriptlang.org/), and [Tailwind CSS v4](https://tailwindcss.com/)
- **Desktop Runtime**: [Tauri v2](https://tauri.app/) (Rust-based security and performance)
- **APIs & Backend Services**: [Speechmatics WebSocket API](https://www.speechmatics.com/) and [Groq Cloud SDK](https://groq.com/)
- **Credentials Manager**: Windows Credential Manager wrapper in Rust

---

## 🚀 Getting Started

### Prerequisites

To run or build the application from source, you will need:

1. **Rust & Cargo**: Version 1.75+ (install via [rustup](https://rustup.rs/))
2. **Node.js**: Version 18+ and `npm`
3. **Microsoft C++ Build Tools**: Required for compiling Tauri/Rust on Windows.

### Installation & Local Development

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/SreekarGpalli/VillFlow2.0.git
   cd VillFlow2.0
   ```

2. **Install Dependencies**:
   ```bash
   npm install
   ```

3. **Run the App in Dev Mode**:
   ```bash
   npm run tauri dev
   ```

---

## 📦 Building Installers

To compile and package the application into standalone installers (`.msi` or `.exe`), execute:

```bash
npm run tauri build
```

The compiled binaries will be outputted to:
- **MSI Installer**: `src-tauri/target/release/bundle/msi/`
- **NSIS EXE Installer**: `src-tauri/target/release/bundle/nsis/`

---

## 🔧 Configuration & Customization

## 🔧 Configuration & Customization

### 🔑 Getting Your API Keys (100% Free)

VillFlow 2.0 connects directly from your machine to Speechmatics and Groq. Both providers offer **excellent free tiers** that allow you to use VillFlow completely free of charge:

#### 1. Speechmatics (Real-Time Speech-to-Text)
* **What you get**: **4 Hours of Free Transcription** every single month.
* **How to get it for free**:
  1. Go to the [Speechmatics Portal](https://portal.speechmatics.com/) and register for a free account.
  2. In the dashboard sidebar, click on **API Keys**.
  3. Click **Generate New API Key**.
  4. Copy the generated key and paste it into the **Services & APIs** tab in the VillFlow settings panel.

#### 2. Groq Cloud (Ultra-Fast Text Cleanup & Commands)
* **What you get**: **Free API access** to state-of-the-art open models (like LLaMA 3.3 and Gemma 2) with high rate limits, perfect for personal assistants.
* **How to get it for free**:
  1. Go to the [Groq Console](https://console.groq.com/) and sign up.
  2. In the sidebar, click on **API Keys**.
  3. Click **Create API Key**, enter a name (e.g., `VillFlow`), and copy it.
  4. Paste the key into the **Services & APIs** tab in the VillFlow settings panel.
  5. Select your preferred model (e.g., `llama-3.3-70b-versatile`) from the models dropdown in VillFlow.

### Prompt Templates
Custom instructions can be defined in the **Prompts** tab to dictate how the assistant formats your transcript. For example:
- *Standard STT*: Cleans up verbal stumbles and formats transcription into readable paragraphs.
- *Developer Mode*: Automatically formats transcriptions into commented code snippets or documentation.

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🤝 Contributing

Contributions are welcome! Please feel free to open a Pull Request or create an Issue to report bugs or request new features.
