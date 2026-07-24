use std::io::{self, Read};

pub fn parse_http_request<R: Read>(reader: &mut R) -> io::Result<(String, String)> {
    let mut buffer = Vec::new();
    let mut read_buf = [0_u8; 4096];
    loop {
        let bytes_read = reader.read(&mut read_buf)?;
        if bytes_read == 0 {
            break;
        }
        buffer.extend_from_slice(&read_buf[..bytes_read]);
        if let Some(header_end) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
            let header_bytes = buffer[..header_end].to_vec();
            let body_start = header_end + 4;
            let mut body = buffer[body_start..].to_vec();
            let header_text = String::from_utf8_lossy(&header_bytes).into_owned();
            if let Some(content_length) = parse_content_length(&header_text) {
                while body.len() < content_length {
                    let bytes_read = reader.read(&mut read_buf)?;
                    if bytes_read == 0 {
                        return Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "incomplete HTTP body for declared Content-Length",
                        ));
                    }
                    body.extend_from_slice(&read_buf[..bytes_read]);
                }
                body.truncate(content_length);
            }
            return Ok((header_text, String::from_utf8_lossy(&body).into_owned()));
        }
    }
    let header = String::from_utf8_lossy(&buffer).into_owned();
    Ok((header, String::new()))
}

fn parse_content_length(header_text: &str) -> Option<usize> {
    header_text.lines().find_map(|line| {
        let trimmed = line.trim();
        let (name, value) = trimmed.split_once(':')?;
        if name.trim().eq_ignore_ascii_case("content-length") {
            value.trim().parse::<usize>().ok()
        } else {
            None
        }
    })
}
