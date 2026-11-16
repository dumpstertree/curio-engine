use crate::UiFontDesc;

/// the atlas grid: where each glyph's cell lives in the texture.
/// v grows downward in texture space, the same way y grows downward in UI
/// pixels — so a cell's top edge is its v_min
pub struct UiFontGrid {
    pub columns: i32,
    pub rows: i32,
    pub u_left: f32,
    pub v_top: f32,
    pub cell_width: f32,
    pub cell_height: f32,
    pub char_order: String,
}
impl UiFontGrid {
    pub fn new(desc: UiFontDesc, width: u32, height: u32) -> UiFontGrid {
        let w = width as f32;
        let h = height as f32;
        UiFontGrid {
            columns: desc.columns,
            rows: desc.rows,
            u_left: desc.padding_left as f32 / w,
            v_top: desc.padding_top as f32 / h,
            cell_width:
                (1.0 - desc.padding_left as f32 / w - desc.padding_right as f32 / w) / desc.columns as f32,
            cell_height:
                (1.0 - desc.padding_top as f32 / h - desc.padding_bottom as f32 / h) / desc.rows as f32,
            char_order: desc.char_order,
        }
    }
    // cell rect [u_min, v_top, u_max, v_bottom] — None for a char the
    // atlas has no cell for
    pub fn glyph_uv(&self, ch: char) -> Option<[f32; 4]> {
        let mut index: i32 = -1;
        for (position, found) in self.char_order.chars().enumerate() {
            if found == ch {
                index = position as i32;
                break;
            }
        }
        if index < 0 {
            return None;
        }
        let col = (index % self.columns) as f32;
        let row = (index / self.columns) as f32;
        Some([
            self.u_left + col * self.cell_width,
            self.v_top + row * self.cell_height,
            self.u_left + (col + 1.0) * self.cell_width,
            self.v_top + (row + 1.0) * self.cell_height,
        ])
    }
}
