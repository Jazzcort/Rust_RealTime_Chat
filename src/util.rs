pub(crate) fn read_buf(buf: &[u8]) -> String {
    match buf.iter().enumerate().find(|(_, x)| **x == 0) {
        Some((ind, _)) => {
            String::from_utf8_lossy(&buf[..ind]).to_string()
        },
        None => String::from_utf8_lossy(&buf).to_string(),
    }
}

pub(crate) fn clear_buf(buf: &mut [u8]) {
    let size = buf.len();
    for i in 0..size {
        buf[i] = 0
    }
}