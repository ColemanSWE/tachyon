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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_request() {
        let request_data = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let request = Request::parse(request_data).unwrap();

        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/");
        assert_eq!(request.version, "HTTP/1.1");
        assert_eq!(request.headers.get("host"), Some(&"example.com".to_string()));
    }

    #[test]
    fn test_parse_request_with_multiple_headers() {
        let request_data = b"POST /api/data HTTP/1.1\r\nHost: api.example.com\r\nContent-Type: application/json\r\n\r\n";
        let request = Request::parse(request_data).unwrap();

        assert_eq!(request.method, "POST");
        assert_eq!(request.path, "/api/data");
        assert_eq!(request.version, "HTTP/1.1");
        assert_eq!(request.headers.get("host"), Some(&"api.example.com".to_string()));
        assert_eq!(request.headers.get("content-type"), Some(&"application/json".to_string()));
    }

    #[test]
    fn test_parse_invalid_request() {
        let request_data = b"INVALID REQUEST\r\n\r\n";
        let result = Request::parse(request_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_request() {
        let request_data = b"";
        let result = Request::parse(request_data);
        assert!(result.is_err());
    }
} 