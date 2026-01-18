# SiroMix - Ứng dụng Trộn Đề Thi Thông Minh
# SiroMix - Smart Exam Shuffling Application

![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)
![Tauri](https://img.shields.io/badge/Tauri-2.0-brightgreen.svg)
![React](https://img.shields.io/badge/React-19.1-61dafb.svg)
![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)

## 📝 Giới thiệu | Introduction

**Tiếng Việt:**

SiroMix là ứng dụng desktop offline-first giúp giáo viên tạo nhiều đề thi khác nhau từ một đề gốc bằng cách tự động trộn thứ tự câu hỏi và đáp án. Ứng dụng được xây dựng bằng Tauri (Rust + React), đảm bảo hiệu suất cao và khả năng làm việc offline hoàn toàn.

**English:**

SiroMix is an offline-first desktop application that helps teachers create multiple exam variants from a single source exam by automatically shuffling question and answer orders. Built with Tauri (Rust + React), it ensures high performance and complete offline functionality.

## ✨ Tính năng chính | Key Features

### Tiếng Việt:
- ✅ **Phân tích đề thi DOCX**: Tự động phân tích cấu trúc câu hỏi, đáp án từ file Word
- ✅ **Hỗ trợ đa định dạng nội dung**: Text, công thức toán học (OMML), hình ảnh (PNG, JPEG, WMF)
- ✅ **Xác thực đáp án tự động**: Phát hiện đáp án đúng dựa trên gạch chân hoặc màu đỏ
- ✅ **Trộn thông minh**: Thuật toán Fisher-Yates với seed để trộn câu hỏi và đáp án
- ✅ **Xuất đề thi DOCX**: Tạo file Word cho từng mã đề với định dạng chuẩn
- ✅ **Xuất đáp án Excel**: Bảng đáp án chéo với công thức kiểm tra tự động
- ✅ **Preview trước khi trộn**: Xem trước nội dung đề gốc với công thức toán MathML
- ✅ **Offline hoàn toàn**: Không cần kết nối internet, dữ liệu được lưu cục bộ

### English:
- ✅ **DOCX Exam Analysis**: Automatically parse question and answer structure from Word files
- ✅ **Multi-format Content Support**: Text, mathematical formulas (OMML), images (PNG, JPEG, WMF)
- ✅ **Automatic Answer Validation**: Detect correct answers based on underline or red color
- ✅ **Smart Shuffling**: Fisher-Yates algorithm with seed for reproducible shuffling
- ✅ **DOCX Export**: Generate Word files for each exam code with standard formatting
- ✅ **Excel Answer Key**: Cross-reference answer table with automatic validation formulas
- ✅ **Preview Before Mixing**: Preview source exam content with MathML formulas
- ✅ **Fully Offline**: No internet required, all data stored locally

## 🏗️ Kiến trúc hệ thống | System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   SiroMix Architecture                       │
└─────────────────────────────────────────────────────────────┘

┌──────────────────────┐
│   Frontend (React)   │
│  - UI/UX Interface   │
│  - State Management  │
│  - MathML Rendering  │
└──────────┬───────────┘
           │ IPC (Tauri Commands)
           ▼
┌──────────────────────────────────────────────────────────┐
│              Backend (Rust - Tauri)                      │
├──────────────────────────────────────────────────────────┤
│  ┌────────────┐  ┌────────────┐  ┌──────────────┐       │
│  │   Parser   │  │  Validator │  │    Mixer     │       │
│  │ - DOCX XML │  │ - Answers  │  │ - Shuffle    │       │
│  │ - Content  │  │ - Rules    │  │ - Algorithm  │       │
│  └────────────┘  └────────────┘  └──────────────┘       │
│                                                           │
│  ┌────────────┐  ┌────────────┐  ┌──────────────┐       │
│  │   Assets   │  │   Writer   │  │    Excel     │       │
│  │ - Images   │  │ - DOCX Gen │  │ - Answer Key │       │
│  │ - WMF→PNG  │  │ - Formatting│  │ - Formulas   │       │
│  └────────────┘  └────────────┘  └──────────────┘       │
└──────────────────────┬───────────────────────────────────┘
                       │
                       ▼
           ┌───────────────────────┐
           │   Local File System   │
           │  - Workspace Storage  │
           │  - Asset Management   │
           └───────────────────────┘
```

## 🔄 Luồng dữ liệu | Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    Exam Processing Flow                      │
└─────────────────────────────────────────────────────────────┘

1. UPLOAD & ANALYZE
   ┌──────────┐
   │ User     │
   │ Selects  │
   │ DOCX     │
   └────┬─────┘
        │
        ▼
   ┌─────────────────┐
   │ analyze_docx    │
   │ - Extract XML   │
   │ - Parse Q&A     │
   │ - Extract Media │
   │ - Validate      │
   └────┬────────────┘
        │
        ▼
   ┌─────────────────┐
   │ parsed.json     │
   │ + assets/       │
   └────┬────────────┘
        │
        │
2. PREVIEW & CONFIGURE
        │
        ▼
   ┌─────────────────┐
   │ PreviewPage     │
   │ - Show Questions│
   │ - Render Math   │
   │ - Configure     │
   └────┬────────────┘
        │
        │
3. MIX EXAMS
        │
        ▼
   ┌─────────────────┐
   │ mix_exams       │
   │ - Shuffle Q     │
   │ - Shuffle A     │
   │ - Map Answers   │
   └────┬────────────┘
        │
        ▼
   ┌─────────────────┐
   │ MixedExam[]     │
   │ (in memory)     │
   └────┬────────────┘
        │
        │
4. EXPORT
        │
        ▼
   ┌──────────────────────────┐
   │ export_mixed_exams       │
   │ ┌──────────────────────┐ │
   │ │ For each variant:    │ │
   │ │ - Generate DOCX XML  │ │
   │ │ - Embed images       │ │
   │ │ - Format document    │ │
   │ └──────────────────────┘ │
   │ ┌──────────────────────┐ │
   │ │ Generate Excel:      │ │
   │ │ - Cross-ref table    │ │
   │ │ - Formulas           │ │
   │ └──────────────────────┘ │
   └────┬─────────────────────┘
        │
        ▼
   ┌─────────────────┐
   │ Output Files    │
   │ - De_101.docx   │
   │ - De_102.docx   │
   │ - ...           │
   │ - Dap_An.xlsx   │
   └─────────────────┘
```

## 📊 Sơ đồ chức năng | Feature Map

```
SiroMix
│
├── 📄 DOCX Processing
│   ├── Parse document.xml
│   ├── Extract paragraphs (w:p)
│   ├── Detect question patterns (Câu X.)
│   ├── Detect option patterns (A., B., C., D.)
│   ├── Extract segments
│   │   ├── Text (w:t)
│   │   ├── Math (m:oMath → OMML)
│   │   └── Images (w:drawing, w:object)
│   └── Media extraction
│       ├── Copy images from word/media/
│       ├── WMF → PNG conversion (async)
│       └── Asset path mapping
│
├── ✅ Validation
│   ├── Answer marking detection
│   │   ├── Underline (w:u)
│   │   └── Red color (w:color="FF0000")
│   ├── Single correct answer per question
│   └── Error codes
│       ├── E020: Missing correct mark
│       └── E021: Multiple correct marks
│
├── 🔀 Mixing Algorithm
│   ├── Fisher-Yates shuffle
│   ├── Seeded random (reproducible)
│   ├── Question shuffling
│   ├── Option shuffling
│   └── Answer mapping
│       ├── Track original → new position
│       └── Update correct answer labels
│
├── 📤 Export System
│   ├── DOCX Generation
│   │   ├── OpenXML structure
│   │   ├── Header with exam info
│   │   ├── Question formatting
│   │   ├── OMML math injection
│   │   ├── Image embedding
│   │   └── Footer with page numbers
│   └── Excel Answer Key
│       ├── Header row (Câu, Đề gốc, Đề 101, ...)
│       ├── Data rows (1-50)
│       ├── Conditional formatting
│       └── Validation formulas
│
└── 💾 Storage
    ├── Job workspace isolation
    ├── Asset management
    └── Parsed data caching
```

## 🛠️ Tech Stack

### Frontend
- **React 19** - UI framework
- **TypeScript** - Type safety
- **Tailwind CSS 4** - Styling
- **Zustand** - State management
- **React Router** - Navigation
- **MathJax 3** - Math formula rendering
- **Heroicons** - Icon library

### Backend
- **Rust** - Core processing
- **Tauri 2** - Desktop framework
- **zip** - DOCX file handling
- **regex** - Pattern matching
- **rust_xlsxwriter** - Excel generation
- **rand** - Random number generation
- **image** - Image processing

## 📦 Cài đặt | Installation

### Yêu cầu hệ thống | System Requirements

**Tiếng Việt:**
- Windows 10/11, macOS 10.15+, hoặc Linux
- Node.js 18+ và pnpm 10+
- Rust 1.70+ (cho development)

**English:**
- Windows 10/11, macOS 10.15+, or Linux
- Node.js 18+ and pnpm 10+
- Rust 1.70+ (for development)

### Development Setup

```bash
# Clone repository
git clone <repository-url>
cd siromix

# Install dependencies
cd apps/desktop
pnpm install

# Run development server
pnpm tauri dev
```

### Build Production

```bash
# Build for production
pnpm tauri build

# Output: src-tauri/target/release/bundle/
```

## 📖 Cách sử dụng | Usage Guide

### Tiếng Việt:

1. **Tải đề gốc DOCX**
   - Click "Chọn file DOCX" và chọn file đề thi gốc
   - Nhập thông tin: Tên kì thi, Môn thi, Thời gian, Trường, v.v.
   - Click "Phân tích đề thi"

2. **Xem trước và kiểm tra**
   - Xem danh sách câu hỏi đã phân tích
   - Kiểm tra công thức toán, hình ảnh
   - Cấu hình số lượng đề và mã đề

3. **Trộn đề thi**
   - Click "Bắt đầu trộn đề"
   - Đợi quá trình xử lý (shuffle câu hỏi và đáp án)

4. **Xuất file**
   - Click "Tải về kết quả"
   - Chọn thư mục lưu
   - Nhận file: De_101.docx, De_102.docx, ..., Dap_An.xlsx

### English:

1. **Upload Source DOCX**
   - Click "Choose DOCX file" and select source exam
   - Enter metadata: Exam name, Subject, Duration, School, etc.
   - Click "Analyze Exam"

2. **Preview and Verify**
   - View parsed question list
   - Check math formulas and images
   - Configure number of variants and exam codes

3. **Mix Exams**
   - Click "Start Mixing"
   - Wait for processing (shuffling questions and answers)

4. **Export Files**
   - Click "Download Results"
   - Choose output folder
   - Receive files: De_101.docx, De_102.docx, ..., Dap_An.xlsx

## 📋 Định dạng đề gốc | Source Exam Format

### Tiếng Việt:

**Cấu trúc câu hỏi:**
```
Câu 1. <Nội dung câu hỏi>
A. <Đáp án A>
B. <Đáp án B>
C. <Đáp án C>
D. <Đáp án D>

Câu 2. <Nội dung câu hỏi>
...
```

**Đánh dấu đáp án đúng:**
- Gạch chân label (A., B., C., D.)
- Hoặc tô màu đỏ (#FF0000) cho label

**Hỗ trợ nội dung:**
- Text thường
- Công thức toán (sử dụng Equation Editor trong Word)
- Hình ảnh (PNG, JPEG, WMF)

### English:

**Question Structure:**
```
Question 1. <Question content>
A. <Answer A>
B. <Answer B>
C. <Answer C>
D. <Answer D>

Question 2. <Question content>
...
```

**Marking Correct Answer:**
- Underline the label (A., B., C., D.)
- Or apply red color (#FF0000) to the label

**Supported Content:**
- Plain text
- Math formulas (using Equation Editor in Word)
- Images (PNG, JPEG, WMF)

## 🔧 API Commands

### Tauri Commands

```rust
// Analyze DOCX exam file
analyze_docx(jobId: string, sourcePath: string) 
  → { ok: boolean, jobId: string, errors?: Error[] }

// Get parsed exam data
get_parsed(jobId: string) 
  → ParsedDoc

// Mix exams (create variants)
mix_exams(parsedDoc: ParsedDoc, numVariants: number, customCodes?: string[]) 
  → MixedExam[]

// Export to DOCX and XLSX
export_mixed_exams(jobId: string, exams: MixedExam[], originalAnswers: string[], outputDir: string) 
  → { success: boolean, docxFiles: string[], xlsxFile: string }
```

## 📂 Cấu trúc thư mục | Directory Structure

```
siromix/
├── apps/desktop/                    # Desktop application
│   ├── src/                         # React frontend
│   │   ├── app/                     # App root
│   │   ├── pages/                   # Page components
│   │   │   ├── MixStart/           # Upload & configure
│   │   │   ├── Preview/            # Preview & validation
│   │   │   └── MixedResult/        # Results & export
│   │   ├── components/              # Shared components
│   │   ├── services/tauri/          # Tauri API wrappers
│   │   ├── store/                   # Zustand state management
│   │   ├── lib/                     # Utilities
│   │   └── constants/               # Constants
│   └── src-tauri/                   # Rust backend
│       └── src/
│           ├── docx/                # DOCX processing modules
│           │   ├── parser.rs       # XML parsing
│           │   ├── validator.rs    # Answer validation
│           │   ├── mixer.rs        # Shuffling algorithm
│           │   ├── writer.rs       # DOCX generation
│           │   ├── excel.rs        # Excel export
│           │   └── assets.rs       # Media extraction
│           └── storage/             # File system operations
├── crates/                          # Shared Rust crates
└── packages/                        # Shared packages
```

## 🧪 Testing

### Validation Error Codes

| Code | Tiếng Việt | English |
|------|-----------|---------|
| E020 | Thiếu đánh dấu đáp án đúng | Missing correct answer mark |
| E021 | Nhiều đáp án được đánh dấu | Multiple correct answers marked |

## 🎯 Roadmap

- [ ] Hỗ trợ câu hỏi nhiều đáp án đúng
- [ ] Khóa câu hỏi/đáp án không bị trộn
- [ ] Template đề thi tùy chỉnh
- [ ] In trực tiếp từ ứng dụng
- [ ] Lưu lịch sử các đề đã trộn
- [ ] Hỗ trợ nhiều ngôn ngữ UI

## 📄 License

MIT License - Copyright (c) 2025

## 👥 Contributors

Made with ❤️ for educators

---

## 📸 Screenshots

```
┌─────────────────────────────────────────┐
│  🏠 Mix Start Page                       │
│  ┌─────────────────────────────────┐    │
│  │ 📁 Choose DOCX file             │    │
│  ├─────────────────────────────────┤    │
│  │ Exam Name: ___________________  │    │
│  │ Subject:   ___________________  │    │
│  │ Duration:  ___________________  │    │
│  │ School:    ___________________  │    │
│  │ Variants:  ___________________  │    │
│  └─────────────────────────────────┘    │
│           [Analyze Exam]                 │
└─────────────────────────────────────────┘

┌─────────────────────────────────────────┐
│  👁️ Preview Page                         │
│  ┌─────────────────────────────────┐    │
│  │ Question 1: ..................  │    │
│  │   A. ........................  │    │
│  │   B. ........................  │    │
│  │   C. ........................  │    │
│  │   D. ........................  │    │
│  ├─────────────────────────────────┤    │
│  │ Question 2: ..................  │    │
│  │   ...                            │    │
│  └─────────────────────────────────┘    │
│           [Start Mixing]                 │
└─────────────────────────────────────────┘

┌─────────────────────────────────────────┐
│  ✅ Result Page                          │
│  ┌─────────────────────────────────┐    │
│  │ ✓ 4 exam variants created       │    │
│  │ ✓ 50 questions per exam         │    │
│  │                                  │    │
│  │ Exam Codes: 101, 102, 103, 104  │    │
│  │                                  │    │
│  │ [Answer Key Table Preview]      │    │
│  └─────────────────────────────────┘    │
│         [Download Results]               │
└─────────────────────────────────────────┘
```

---

**🎓 SiroMix - Làm việc offline, hiệu suất cao, dễ sử dụng**

**🎓 SiroMix - Work offline, High performance, Easy to use**
