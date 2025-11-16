use crate::utilities::Conversion;

pub(crate) struct Page {
    width: f32,
    height: f32,
    left_margin: f32,
    right_margin: f32,
    usable_width: f32,
}

impl Page {
    pub fn new(width_mm: f32, height_mm: f32, left_margin_mm: f32, right_margin_mm: f32) -> Self {
        let usable_width = width_mm - left_margin_mm - right_margin_mm;

        Self {
            width: width_mm,
            height: height_mm,
            left_margin: left_margin_mm,
            right_margin: right_margin_mm,
            usable_width,
        }
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn left_margin(&self) -> f32 {
        self.left_margin
    }

    pub fn right_margin(&self) -> f32 {
        self.right_margin
    }

    pub fn usable_width(&self) -> f32 {
        self.usable_width
    }

    pub fn compute_wrapping(&self, font_size_pt: f32, indent_mm: f32) -> usize {
        let usable_mm_with_indent = (self.usable_width - indent_mm).max(0.0);
        let usable_pt = self.mm_to_pt(usable_mm_with_indent) as f64;
        let font_size = font_size_pt as f64;
        let avg_char_factor = 0.50_f64;
        let avg_char_width_pt = font_size * avg_char_factor;
        let computed = (usable_pt / avg_char_width_pt).floor() as isize;

        computed.max(1) as usize
    }
}

impl Default for Page {
    fn default() -> Self {
        Self::new(210.0, 297.0, 20.0, 20.0)
    }
}

impl Conversion for Page {}
