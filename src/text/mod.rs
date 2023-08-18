// renderer needs text drawing capability
// keep it simple:
// - we will use a library to convert font glyphs (bezier) into bitmaps
// - stitch those bitmaps into an atlas
// - draw textured rects for each letter

pub mod font_bitmap_manager;
pub mod pipeline;
