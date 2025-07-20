use std::collections::HashMap;

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
}

impl Request {
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        let request_str = String::from_utf8_lossy(data);
        let lines: Vec<&str> = request_str.lines().collect();

        if lines.is_empty() {
            return Err("Empty request".to_string());
        }

        let request_line: Vec<&str> = lines[0].split_whitespace().collect();
        if request_line.len() != 3 {
            return Err("Invalid request line".to_string());
        }

        let method = request_line[0].to_string();
        let path = request_line[1].to_string();
        let version = request_line[2].to_string();

        let mut headers = HashMap::new();

        for line in lines.iter().skip(1) {
            if line.is_empty() {
                break;
            }

            if let Some(colon_pos) = line.find(':') {
                let key = line[..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 1..].trim().to_string();
                headers.insert(key, value);
            }
        }

        Ok(Request {
            method,
            path,
            version,
            headers,
        })
    }
} 