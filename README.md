# CATUL

Simple offline CAT tool (Computer-Assisted Translation) built with Tauri + React + TypeScript + Rust + SQLite.

Right now it's a basic proof-of-concept: just a dark-mode editor placeholder using Monaco.

## Current status (March 2026)

- Side-by-side layout placeholder
- Fake segment list
- Monaco Editor for target text editing
- Basic segment selection & navigation
- No real Translation Memory yet
- No file import/export
- No fuzzy matching
- No glossary
- No QA checks

Everything runs locally via Tauri — fully offline.

## Tech stack

- Frontend: React + TypeScript + Tailwind CSS + Monaco Editor
- Desktop / Backend: Tauri v2 (Rust)
- Database: planning SQLite
- License: Apache 2.0

## Next realistic steps (in order)

1. Set up SQLite DB + basic schema for segments & TM
2. Implement exact match lookup from Rust
3. Add fuzzy matching (levenshtein or strsim)
4. Load/save segments from/to DB using Tauri commands
5. Basic .txt file import/export
6. Glossary table + simple term highlighting
7. Basic QA checks (tags, numbers, punctuation)

Later (maybe): .docx, .srt, .pdf support, optional MT suggestions, etc.

## How to run locally

```bash
# Install frontend dependencies
npm install

# Start frontend dev server
npm run dev

# In a separate terminal → start Tauri app (backend + window)
cd src-tauri
cargo tauri dev
```

##### Currently developing with all my heart to create a more friendly and opensource enviroment for translators