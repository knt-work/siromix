// src-tauri/src/docx/config.rs
//! Configuration constants for document formatting
//! Compliant with Nghị định 30/2020/NĐ-CP - Vietnamese government decree on administrative documents

/// Nghị định 30/2020/NĐ-CP - Quy định về công tác văn thư
/// https://thuvienphapluat.vn/van-ban/Bo-may-hanh-chinh/Nghi-dinh-30-2020-ND-CP-cong-tac-van-thu-434678.aspx
///
/// This struct contains all formatting standards mandated by the Vietnamese government
/// for official documents. DO NOT modify these values unless the decree is amended.
pub struct NghiDinh30;

impl NghiDinh30 {
    // Unit conversion: 1 mm = 56.7 twips (twentieth of a point)
    // 1 point = 20 twips
    
    // ==================== PAGE SIZE ====================
    /// A4 paper width: 210mm
    pub const PAGE_WIDTH_TWIPS: i32 = 11906;
    
    /// A4 paper height: 297mm
    pub const PAGE_HEIGHT_TWIPS: i32 = 16838;
    
    // ==================== MARGINS ====================
    /// Top margin: 20mm (20-25mm range)
    pub const MARGIN_TOP_TWIPS: i32 = 1134;
    
    /// Bottom margin: 20mm (20-25mm range)
    pub const MARGIN_BOTTOM_TWIPS: i32 = 1134;
    
    /// Left margin: 30mm (30-35mm range, for binding)
    pub const MARGIN_LEFT_TWIPS: i32 = 1701;
    
    /// Right margin: 15mm (15-20mm range)
    pub const MARGIN_RIGHT_TWIPS: i32 = 851;
    
    /// Header margin: 12.5mm
    pub const MARGIN_HEADER_TWIPS: i32 = 708;
    
    /// Footer margin: 12.5mm
    pub const MARGIN_FOOTER_TWIPS: i32 = 708;
    
    // ==================== FONT ====================
    /// Font family: Times New Roman (mandatory)
    pub const FONT_NAME: &'static str = "Times New Roman";
    
    /// Font encoding: Unicode per TCVN 6909:2001
    pub const FONT_ENCODING: &'static str = "Unicode (TCVN 6909:2001)";
    
    // ==================== FONT SIZES ====================
    // Note: OpenXML uses half-points (1pt = 2 half-points)
    
    /// Title font size: 14pt
    pub const FONT_SIZE_TITLE: i32 = 28;
    
    /// Exam name font size: 16pt
    pub const FONT_SIZE_EXAM_NAME: i32 = 32;
    
    /// Subtitle font size: 12pt
    pub const FONT_SIZE_SUBTITLE: i32 = 24;
    
    /// Body text font size: 13pt (main content)
    pub const FONT_SIZE_BODY: i32 = 26;
    
    /// Page number font size: 13pt (as per decree requirement)
    pub const FONT_SIZE_PAGE_NUMBER: i32 = 26;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test suite to ensure Nghị định 30 standards are NOT changed
    /// If these tests fail, someone is trying to change government standards!
    
    #[test]
    fn test_nghi_dinh_30_page_size_immutable() {
        // A4 paper size MUST be 210mm x 297mm
        // 210mm = 11906 twips, 297mm = 16838 twips
        assert_eq!(
            NghiDinh30::PAGE_WIDTH_TWIPS,
            11906,
            "Page width MUST be 11906 twips (210mm) per Nghị định 30"
        );
        assert_eq!(
            NghiDinh30::PAGE_HEIGHT_TWIPS,
            16838,
            "Page height MUST be 16838 twips (297mm) per Nghị định 30"
        );
    }

    #[test]
    fn test_nghi_dinh_30_margins_immutable() {
        // Margins per Nghị định 30: top/bottom 20mm, left 30mm, right 15mm
        // Conversion: 1mm = 56.7 twips
        assert_eq!(
            NghiDinh30::MARGIN_TOP_TWIPS,
            1134,
            "Top margin MUST be 1134 twips (20mm) per Nghị định 30"
        );
        assert_eq!(
            NghiDinh30::MARGIN_BOTTOM_TWIPS,
            1134,
            "Bottom margin MUST be 1134 twips (20mm) per Nghị định 30"
        );
        assert_eq!(
            NghiDinh30::MARGIN_LEFT_TWIPS,
            1701,
            "Left margin MUST be 1701 twips (30mm) per Nghị định 30"
        );
        assert_eq!(
            NghiDinh30::MARGIN_RIGHT_TWIPS,
            851,
            "Right margin MUST be 851 twips (15mm) per Nghị định 30"
        );
    }

    #[test]
    fn test_nghi_dinh_30_font_name_immutable() {
        // Font MUST be Times New Roman
        assert_eq!(
            NghiDinh30::FONT_NAME,
            "Times New Roman",
            "Font name MUST be 'Times New Roman' per Nghị định 30"
        );
    }

    #[test]
    fn test_nghi_dinh_30_font_encoding_immutable() {
        // Encoding MUST be Unicode TCVN 6909:2001
        assert_eq!(
            NghiDinh30::FONT_ENCODING,
            "Unicode (TCVN 6909:2001)",
            "Font encoding MUST be 'Unicode (TCVN 6909:2001)' per Nghị định 30"
        );
    }

    #[test]
    fn test_nghi_dinh_30_font_sizes_immutable() {
        // Font sizes per Nghị định 30 (in half-points)
        assert_eq!(
            NghiDinh30::FONT_SIZE_TITLE,
            28,
            "Title font size MUST be 28 (14pt) per Nghị định 30"
        );
        assert_eq!(
            NghiDinh30::FONT_SIZE_EXAM_NAME,
            32,
            "Exam name font size MUST be 32 (16pt) per Nghị định 30"
        );
        assert_eq!(
            NghiDinh30::FONT_SIZE_SUBTITLE,
            24,
            "Subtitle font size MUST be 24 (12pt) per Nghị định 30"
        );
        assert_eq!(
            NghiDinh30::FONT_SIZE_BODY,
            26,
            "Body font size MUST be 26 (13pt) per Nghị định 30"
        );
        assert_eq!(
            NghiDinh30::FONT_SIZE_PAGE_NUMBER,
            26,
            "Page number font size MUST be 26 (13pt) per Nghị định 30"
        );
    }

    #[test]
    fn test_nghi_dinh_30_complete_compliance() {
        // Comprehensive test to ensure ALL standards are met
        // Page size A4
        assert_eq!(NghiDinh30::PAGE_WIDTH_TWIPS, 11906);
        assert_eq!(NghiDinh30::PAGE_HEIGHT_TWIPS, 16838);
        
        // Margins
        assert_eq!(NghiDinh30::MARGIN_TOP_TWIPS, 1134);
        assert_eq!(NghiDinh30::MARGIN_BOTTOM_TWIPS, 1134);
        assert_eq!(NghiDinh30::MARGIN_LEFT_TWIPS, 1701);
        assert_eq!(NghiDinh30::MARGIN_RIGHT_TWIPS, 851);
        
        // Font
        assert_eq!(NghiDinh30::FONT_NAME, "Times New Roman");
        
        // Font sizes
        assert_eq!(NghiDinh30::FONT_SIZE_BODY, 26); // 13pt
        assert_eq!(NghiDinh30::FONT_SIZE_PAGE_NUMBER, 26); // 13pt
    }
}
