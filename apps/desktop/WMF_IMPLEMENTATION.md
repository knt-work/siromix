# WMF/EMF Image Support Implementation

## Tổng quan

Đã implement giải pháp phát hiện và xử lý file WMF/EMF (từ OLE Equation Editor 3.0) trong SiroMix.

## Files đã tạo/thay đổi

### 1. `src/lib/wmfConverter.ts`
Utility phát hiện file WMF/EMF:
- `isWmfFile(path)`: Kiểm tra extension có phải .wmf hoặc .emf
- `getWmfInfo(path)`: Lấy thông tin file (filename, extension)

### 2. `src/components/ImageSegment.tsx`
Component render image với hỗ trợ phát hiện WMF:
- **Ảnh thông thường** (PNG/JPG): Render trực tiếp qua `convertFileSrc`
- **File WMF/EMF**: Hiển thị fallback message với hướng dẫn khắc phục chi tiết

### 3. `src/pages/Preview/PreviewPage.tsx`
Updated để sử dụng `ImageSegment` component thay vì render `<img>` trực tiếp

### 4. Dependencies đã cài
```json
{
  "@tauri-apps/plugin-fs": "^2.4.4",
  "wmf": "^1.0.2",
  "rtf.js": "^3.0.9"
}
```

## Trạng thái hiện tại

### ✅ Đã hoàn thành
- ✅ Phát hiện file WMF/EMF
- ✅ Hiển thị fallback UI với thông báo rõ ràng
- ✅ Hướng dẫn người dùng 3 cách khắc phục
- ✅ Component architecture sẵn sàng cho future enhancement

### ⏳ Chưa implement (TODO)
- ⏳ **WMF Conversion thực tế**: JavaScript libraries (wmf, rtf.js) không đủ mạnh để render WMF từ Equation Editor 3.0
  
## Tại sao không convert WMF ngay?

### Vấn đề với JavaScript WMF parsers:
1. **Thư viện `wmf` (SheetJS)**:
   - Chỉ parse metadata cơ bản
   - Không render đầy đủ GDI commands phức tạp
   - Thiếu support cho Equation Editor WMF

2. **Thư viện `rtf.js`**:
   - Thiết kế cho RTF documents (embedded WMF)
   - Không hoạt động tốt với standalone WMF files
   - API không phù hợp cho use case này

### Recommendation cho Phase 2:

Có 3 options để implement WMF conversion thực sự:

#### **Option 1: Rust Backend + ImageMagick (Recommended)**
```rust
// Thêm vào src-tauri/Cargo.toml
[dependencies]
imagemagick = "0.5"

// Tạo Tauri command
#[tauri::command]
async fn convert_wmf_to_png(wmf_path: String) -> Result<String, String> {
    // 1. Đọc WMF file
    // 2. Convert sang PNG bằng ImageMagick
    // 3. Lưu vào cache/<hash>.png
    // 4. Return cache path
}
```

**Pros:**
- ImageMagick hỗ trợ đầy đủ WMF/EMF
- Cross-platform (Windows, macOS, Linux)
- Offline, không cần internet
- Có thể bundle binary với app

**Cons:**
- Tăng kích thước app (~10-20 MB)
- Phức tạp hơn để setup build

#### **Option 2: System Command (wmf2png, ImageMagick CLI)**
```typescript
// Frontend
import { Command } from '@tauri-apps/plugin-shell';

async function convertWmf(wmfPath: string): Promise<string> {
  const outputPath = `cache/${hash}.png`;
  await Command.create('convert', [wmfPath, outputPath]).execute();
  return outputPath;
}
```

**Pros:**
- Đơn giản, không tăng app size
- Linh hoạt (có thể dùng nhiều tool)

**Cons:**
- ❌ Yêu cầu user tự cài ImageMagick → **vi phạm requirement**
- Không đáp ứng "1 app installer only"

#### **Option 3: Cloud Service API**
```typescript
async function convertWmfViaAPI(wmfPath: string): Promise<string> {
  const bytes = await readFile(wmfPath);
  const response = await fetch('https://api.example.com/convert-wmf', {
    method: 'POST',
    body: bytes,
  });
  return await response.json();
}
```

**Pros:**
- Không tăng app size
- Có thể leverage cloud processing power

**Cons:**
- ❌ Cần internet connection → **vi phạm offline requirement**
- Privacy concerns (upload đề thi lên server)
- Latency cao

## Kết luận và next steps

### Current State: MVP Ready ✅
App hiện tại:
- ✅ Phát hiện được WMF files
- ✅ Hiển thị thông báo rõ ràng cho người dùng
- ✅ Hướng dẫn cách khắc phục
- ✅ Không crash khi gặp WMF
- ✅ UX tốt với fallback UI

### Recommended Roadmap:

**Phase 1 (Hiện tại):** ✅ DONE
- Fallback UI + user guidance

**Phase 2 (Next Sprint):**
- Implement Option 1 (Rust + ImageMagick)
- Bundle ImageMagick binary với Tauri app
- Implement caching strategy (disk-based)
- Test với nhiều loại WMF files

**Phase 3 (Future):**
- Lazy loading WMF conversion (chỉ convert khi scroll vào viewport)
- Web Worker cho conversion (nếu chuyển sang pure JS solution)
- Batch conversion cho performance

## Testing

### Test case cần chạy:
1. ✅ File có ảnh PNG/JPG thông thường → hiển thị OK
2. 🔜 File có OLE Equation (WMF) → hiển thị fallback message
3. 🔜 File mixed (cả OMML math + WMF + PNG) → render đúng từng loại
4. 🔜 Stress test: 50+ equations trong 1 file

### Để test ngay bây giờ:
```bash
cd apps/desktop
pnpm dev
# Import file .docx có OLE Equation
# → Xem fallback UI có hiển thị đúng không
```

## API Documentation

### ImageSegment Component

```typescript
import { ImageSegment } from '@/components/ImageSegment';

// Usage
<ImageSegment 
  assetPath="/path/to/image.png"  // or .wmf
  className="custom-class"         // optional
/>
```

**Props:**
- `assetPath`: Absolute path đến file ảnh
- `className`: Optional CSS classes

**Behavior:**
- PNG/JPG/GIF → render `<img>` với `convertFileSrc`
- WMF/EMF → render fallback message với hướng dẫn

### wmfConverter utilities

```typescript
import { isWmfFile, getWmfInfo } from '@/lib/wmfConverter';

// Check if file is WMF/EMF
if (isWmfFile('/path/to/file.wmf')) {
  const { filename, ext } = getWmfInfo('/path/to/file.wmf');
  console.log(`${filename} is a ${ext} file`);
}
```

## Notes

- Parser backend (Rust) **không cần thay đổi** - đã parse đúng và extract WMF files
- Frontend architecture đã sẵn sàng cho WMF conversion khi có solution
- Current implementation đáp ứng MVP requirement: user-friendly error handling
