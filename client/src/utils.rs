use arrayvec::ArrayVec;

pub fn color_to_string(color: &ArrayVec<u8, 3>) -> String {
    format!("rgb({}, {}, {})", color[0], color[1], color[2])
}

pub fn delimit_string<'a>(lines: &[String], delimiter: &str) -> String {
    let mut result = String::new();
    for (i, line) in lines.iter().enumerate() {
        result += line;
        if i != lines.len() - 1 {
            result += delimiter;
        }
    }
    result
}
