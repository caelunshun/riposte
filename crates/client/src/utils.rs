use arrayvec::ArrayVec;
use dume::{Srgba, Text};

pub fn merge_text_lines(lines: impl IntoIterator<Item = Text>) -> Text {
    delimit_text(lines, text!("\n"))
}

pub fn delimit_text(text: impl IntoIterator<Item = Text>, delimiter: Text) -> Text {
    let mut res = Text::from_sections(None);

    for (i, line) in text.into_iter().enumerate() {
        if i != 0 {
            res.extend(delimiter.clone());
        }
        res.extend(line);
    }

    res
}

pub fn convert_color(color: &ArrayVec<u8, 3>) -> Srgba<u8> {
    Srgba::new(color[0], color[1], color[2], u8::MAX)
}
