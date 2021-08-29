use arrayvec::ArrayVec;

pub fn color_to_string(color: &ArrayVec<u8, 3>) -> String {
    format!("rgb({}, {}, {})", color[0], color[1], color[2])
}
