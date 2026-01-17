# Form Handling Implementation

## Tổng quan
Đã implement form handling để nhập và quản lý metadata của đề thi ở bước 1 (MixStartPage).

## Chi tiết triển khai

### 1. Store (mixStore.ts)
**Interface mới:**
```typescript
interface ExamMetadata {
  examName: string;         // Tên kỳ thi
  subject: string;          // Môn thi
  durationMinutes: number;  // Số phút
  numVariants: number;      // Số lượng đề cần trộn
  customExamCodes: string[]; // Mã đề tùy chỉnh
}
```

**State management:**
- `examMetadata: ExamMetadata | null` - Lưu thông tin metadata
- `setExamMetadata(metadata: ExamMetadata)` - Cập nhật metadata
- Tự động persist vào localStorage

### 2. Constants (constants/exam.ts)
```typescript
export const DEFAULT_EXAM_CODES = ["132", "209", "357", "485"];
export const DEFAULT_NUM_VARIANTS = 4;
export const DEFAULT_DURATION = 90;
```

### 3. MixStartPage - Form State
**State variables:**
- `examName` - Tên kỳ thi (khởi tạo từ cache hoặc "")
- `subject` - Môn thi
- `duration` - Số phút (string để handle input)
- `numVariants` - Số lượng đề (string)
- `examCodes` - Array mã đề tùy chỉnh

**Dynamic behavior:**
- useEffect tự động điều chỉnh số lượng input mã đề khi `numVariants` thay đổi
- Nếu tăng số lượng → thêm mã đề mới từ DEFAULT_EXAM_CODES
- Nếu giảm → cắt bớt array

### 4. Form Validation (handleSubmit)
**Kiểm tra các điều kiện:**
1. ✅ Tên kỳ thi không được để trống
2. ✅ Môn thi không được để trống
3. ✅ Số phút phải là số hợp lệ
4. ✅ Số phút phải > 0
5. ✅ Số lượng đề phải là số hợp lệ
6. ✅ Số lượng đề phải > 0
7. ✅ Tất cả mã đề phải được nhập (không để trống)
8. ✅ Không được có mã đề trùng lặp (sử dụng Set)
9. ✅ File .docx phải được chọn

**Thông báo lỗi:**
- Sử dụng `window.alert()` để hiển thị lỗi validation
- Mỗi điều kiện có message cụ thể

**Sau khi validate:**
```typescript
const metadata: ExamMetadata = {
  examName: examName.trim(),
  subject: subject.trim(),
  durationMinutes: parsedDuration,
  numVariants: parsedNumVariants,
  customExamCodes: trimmedCodes
};
setExamMetadata(metadata);
// Tiếp tục xử lý upload file...
```

### 5. Form UI Components

**Row 1: Tên kì thi + Môn thi**
- 2 columns responsive (grid-cols-2 trên md)
- Icons: DocumentTextIcon, AcademicCapIcon
- Two-way binding: `value={examName}` + `onChange={(e) => setExamName(e.target.value)}`

**Row 2: Số phút + Số lượng đề**
- Input type="number" với min={1}
- Icons: ClockIcon, DocumentDuplicateIcon
- Two-way binding tương tự

**Row 3: Mã đề tùy chỉnh (Dynamic)**
```tsx
<div className="mt-3 grid grid-cols-2 gap-4 md:grid-cols-4">
  {examCodes.map((code, idx) => (
    <div key={idx} className="relative">
      <input
        type="text"
        value={code}
        onChange={(e) => {
          const newCodes = [...examCodes];
          newCodes[idx] = e.target.value;
          setExamCodes(newCodes);
        }}
        placeholder={`Mã đề ${idx + 1}`}
        className="..."
      />
    </div>
  ))}
</div>
```
- Grid responsive: 2 columns mobile, 4 columns desktop
- Số lượng inputs = `numVariants`
- Center-aligned text cho mã đề

### 6. PreviewPage - Confirmation Modal
**Hiển thị real data thay vì hardcoded:**
```tsx
const { examMetadata, numVariants } = useMixStore();

<div className="flex justify-between text-sm">
  <span className="font-medium text-slate-700">Tên kì thi:</span>
  <span className="font-semibold text-slate-900">{examMetadata?.examName || "—"}</span>
</div>
// Tương tự cho subject, durationMinutes, numVariants
<div className="flex justify-between text-sm">
  <span className="font-medium text-slate-700">Mã đề:</span>
  <span className="font-semibold text-slate-900">
    {examMetadata?.customExamCodes?.join(", ") || "—"}
  </span>
</div>
```

## Luồng dữ liệu
```
1. User nhập form ở MixStartPage
2. useState local tracking: examName, subject, duration, numVariants, examCodes
3. User submit → handleSubmit validate
4. Nếu pass validation → save to Zustand store: setExamMetadata()
5. Store persist to localStorage
6. User navigate to PreviewPage
7. PreviewPage đọc examMetadata từ store
8. Modal hiển thị real data
9. User confirm → mixExams được gọi với customExamCodes
```

## Testing Checklist
- [ ] Nhập đầy đủ 4 fields mandatory → Submit thành công
- [ ] Để trống bất kỳ field nào → Hiện error alert
- [ ] Nhập số phút/số lượng đề = 0 hoặc âm → Error
- [ ] Nhập mã đề trùng lặp (ví dụ: 132, 132, 357, 485) → Error
- [ ] Thay đổi số lượng đề từ 4 → 6 → Hiện thêm 2 input mã đề
- [ ] Thay đổi số lượng đề từ 4 → 2 → Chỉ còn 2 input mã đề
- [ ] Submit và kiểm tra modal ở PreviewPage hiển thị đúng data
- [ ] Refresh page → Data vẫn còn (localStorage persistence)

## Next Steps
- [ ] Pass customExamCodes to Rust mixer instead of generating random codes
- [ ] Update MixedResultPage to display examMetadata
- [ ] Add export filename based on examName + subject
- [ ] Consider adding form reset button
