# 🎬 Caption Cleaner

A small desktop tool that **extracts captions from YouTube videos**, **cleans formatting**, and **corrects spelling and grammar** — without rewriting or adding information.

Built as a **personal developer utility** using **Rust + Tauri** for the backend and **React + shadcn/ui** for the frontend.

---

## ✨ Features

* Extract captions from YouTube videos using `yt-dlp`
* Convert WebVTT (`.vtt`) captions into clean, readable text
* Fix:

  * Spelling mistakes
  * Grammar errors
  * Punctuation and capitalization
  * Broken line formatting
* **Strict non-rewriting guarantee** (no summarization, no paraphrasing)
* Optional AI-powered cleanup using **Groq (LLaMA 3.1)**
* Runs entirely on your machine (no server)

---

## 🧠 How It Works

1. The app runs `yt-dlp` to download English captions (`.vtt`)
2. Rust parses the VTT file and removes:

   * Timestamps
   * Metadata
   * HTML tags
3. Lines belonging to the same sentence are merged
4. (Optional) The cleaned text is sent to Groq for **mechanical correction only**
5. The final transcript is displayed in the UI and can be copied or saved

---

## 🛠 Tech Stack

### Backend

* **Rust**
* **Tauri**
* `yt-dlp`
* `reqwest`, `serde`, `regex`
* `thiserror` for typed error handling

### Frontend

* **React**
* **TypeScript**
* **shadcn/ui**
* **Tailwind CSS**

### AI (Optional)

* **Groq API**
* LLaMA 3.1 (8B Instant)
* `temperature = 0` for deterministic edits

---

## 📦 Project Structure

```
caption-cleaner/
├── src-tauri/
│   ├── src/
│   │   ├── errors.rs
│   │   ├── vtt.rs
│   │   ├── yt_dlp.rs
│   │   ├── groq.rs
│   │   ├── pipeline.rs
│   │   └── main.rs
│   └── Cargo.toml
├── src/
│   └── App.tsx
└── README.md
```

---

## 🚀 Getting Started

### Prerequisites

* Rust (stable)
* Node.js
* `yt-dlp` available in `PATH`
* (Optional) Groq API key

```bash
brew install yt-dlp
```

---

### Environment Variables (AI cleanup)

```bash
export GROQ_API_KEY="your_api_key_here"
```

---

### Run in development

```bash
bun install
bun run tauri dev
```

---

## 🧪 Design Philosophy

* **Small scope, clean implementation**
* Deterministic behavior
* No hidden rewriting or summarization
* Clear separation between:

  * Caption extraction
  * Formatting
  * AI-assisted correction
* Built as a **personal developer tool**, not a SaaS product

---
