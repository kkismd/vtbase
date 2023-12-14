use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};

pub struct IncludeReader<R> {
    reader: R,
    include_stack: Vec<Box<dyn Iterator<Item = io::Result<String>>>>,
    current_path: PathBuf,
}

impl<R: BufRead> IncludeReader<R> {
    pub fn new(reader: R, current_path: PathBuf) -> Self {
        IncludeReader {
            reader,
            include_stack: Vec::new(),
            current_path: current_path,
        }
    }

    pub fn lines(self) -> impl Iterator<Item = io::Result<String>> {
        self
    }
}

impl<R: BufRead> Iterator for IncludeReader<R> {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(include_reader) = self.include_stack.last_mut() {
                if let Some(line) = include_reader.next() {
                    return Some(line);
                } else {
                    self.include_stack.pop();
                }
            } else {
                let mut line = String::new();
                match self.reader.read_line(&mut line) {
                    Ok(0) => return None,
                    Ok(_) => {
                        if let Some(filename) = detect_include_directive(&line) {
                            let path = self.current_path.join(&filename);
                            if let Ok(file) = File::open(&Path::new(&path)) {
                                let reader = BufReader::new(file);
                                self.include_stack.push(Box::new(reader.lines()));
                            } else {
                                return Some(Err(io::Error::new(
                                    io::ErrorKind::NotFound,
                                    "File not found",
                                )));
                            }
                        } else {
                            return Some(Ok(line));
                        }
                    }
                    Err(e) => return Some(Err(e)),
                }
            }
        }
    }
}

fn detect_include_directive(s: &str) -> Option<String> {
    let re = regex::Regex::new(r#"^\s+\+=\"([^\"]+)\"\s*$"#).unwrap();
    re.captures(s)
        .and_then(|cap| cap.get(1).map(|m| m.as_str().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_detect_include_directive() {
        let s = "    +=\"foo.inc\"";
        assert_eq!(detect_include_directive(s), Some("foo.inc".to_string()));
    }

    #[test]
    fn test_detect_include_directive_empty_name() {
        let s = "    +=\"\"";
        assert_eq!(detect_include_directive(s), None);
    }
}
