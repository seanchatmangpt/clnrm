//! Phase 3: Token Extraction and Indexing
//!
//! Builds TokenIndex for files: extracts identifiers, comments, headings,
//! config keys and their frequencies/positions.

use crate::schemas::TokenIndex;
use regex::Regex;

/// Token extractor for different file types
pub struct TokenExtractor;

impl TokenExtractor {
    /// Extract tokens from a Rust source file
    pub fn extract_rust_tokens(content: &str) -> TokenIndex {
        let mut index = TokenIndex::default();
        let lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = line_idx + 1;

            // Extract tokens using regex patterns
            index.extract_rust_identifiers(line, line_num);
            index.extract_comments(line, line_num);
        }

        index
    }

    /// Extract tokens from Markdown documentation
    pub fn extract_markdown_tokens(content: &str) -> TokenIndex {
        let mut index = TokenIndex::default();
        let lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = line_idx + 1;

            // Extract headings
            if line.starts_with('#') {
                let heading = line.trim_start_matches('#').trim();
                index.headings.push(heading.to_string());
                index.add_token(heading, line_num, 0);
            }

            // Extract code blocks
            if line.starts_with("```") {
                continue;
            }

            // Extract all words as potential keywords
            for (col, word) in line.split_whitespace().enumerate() {
                let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                if !cleaned.is_empty() {
                    index.add_token(cleaned, line_num, col);
                }
            }
        }

        index
    }

    /// Extract tokens from TOML configuration
    pub fn extract_toml_tokens(content: &str) -> TokenIndex {
        let mut index = TokenIndex::default();
        let lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = line_idx + 1;

            // Extract keys
            if let Some(eq_idx) = line.find('=') {
                let key_part = line[..eq_idx].trim();
                if !key_part.is_empty() {
                    index.config_keys.push(key_part.to_string());
                    index.add_token(key_part, line_num, 0);
                }
            }

            // Extract all tokens
            for (col, word) in line.split_whitespace().enumerate() {
                let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                if !cleaned.is_empty() && !cleaned.starts_with('"') {
                    index.add_token(cleaned, line_num, col);
                }
            }
        }

        index
    }

    /// Extract tokens from YAML configuration
    pub fn extract_yaml_tokens(content: &str) -> TokenIndex {
        let mut index = TokenIndex::default();
        let lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = line_idx + 1;

            // Extract keys (before colon)
            if let Some(colon_idx) = line.find(':') {
                let key_part = line[..colon_idx].trim();
                if !key_part.is_empty() && !key_part.starts_with('#') {
                    index.config_keys.push(key_part.to_string());
                    index.add_token(key_part, line_num, 0);
                }
            }

            // Extract all tokens
            for (col, word) in line.split_whitespace().enumerate() {
                let cleaned =
                    word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-');
                if !cleaned.is_empty() && !cleaned.starts_with('#') {
                    index.add_token(cleaned, line_num, col);
                }
            }
        }

        index
    }

    /// Generic token extraction for unknown file types
    pub fn extract_generic_tokens(content: &str) -> TokenIndex {
        let mut index = TokenIndex::default();
        let lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = line_idx + 1;

            // Skip comments
            if line.trim_start().starts_with('#') || line.trim_start().starts_with("//") {
                continue;
            }

            // Extract all tokens
            for (col, word) in line.split_whitespace().enumerate() {
                let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                if !cleaned.is_empty() {
                    index.add_token(cleaned, line_num, col);
                }
            }
        }

        index
    }
}

impl TokenIndex {
    /// Add a token to the index
    pub fn add_token(&mut self, token: &str, line: usize, col: usize) {
        let lower = token.to_lowercase();
        let entry = self.tokens.entry(lower).or_insert((0, Vec::new()));
        entry.0 += 1;
        entry.1.push((line, col));
    }

    /// Extract Rust identifiers and types
    fn extract_rust_identifiers(&mut self, line: &str, line_num: usize) {
        // Skip comments
        let code_part = if let Some(comment_idx) = line.find("//") {
            &line[..comment_idx]
        } else {
            line
        };

        // Pattern for identifiers: underscore, letters, numbers
        let ident_re = Regex::new(r"\b([a-zA-Z_][a-zA-Z0-9_]*)\b").unwrap();

        for (col, cap) in ident_re.captures_iter(code_part).enumerate() {
            if let Some(m) = cap.get(1) {
                self.add_token(m.as_str(), line_num, col);
            }
        }

        // Extract keywords
        if code_part.contains("fn ") {
            self.add_token("fn", line_num, 0);
        }
        if code_part.contains("struct ") {
            self.add_token("struct", line_num, 0);
        }
        if code_part.contains("impl ") {
            self.add_token("impl", line_num, 0);
        }
        if code_part.contains("trait ") {
            self.add_token("trait", line_num, 0);
        }
        if code_part.contains("async ") {
            self.add_token("async", line_num, 0);
        }
    }

    /// Extract comments
    fn extract_comments(&mut self, line: &str, line_num: usize) {
        if let Some(idx) = line.find("//") {
            let comment = line[idx + 2..].trim();
            if !comment.is_empty() {
                self.comments.push(comment.to_string());
                for (col, word) in comment.split_whitespace().enumerate() {
                    let cleaned = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
                    if !cleaned.is_empty() {
                        self.add_token(cleaned, line_num, col);
                    }
                }
            }
        }
    }

    /// Get all unique tokens
    pub fn unique_tokens(&self) -> Vec<String> {
        self.tokens.keys().cloned().collect()
    }

    /// Get frequency of a token
    pub fn frequency(&self, token: &str) -> usize {
        self.tokens
            .get(&token.to_lowercase())
            .map(|(freq, _)| *freq)
            .unwrap_or(0)
    }

    /// Get positions of a token
    pub fn positions(&self, token: &str) -> Vec<(usize, usize)> {
        self.tokens
            .get(&token.to_lowercase())
            .map(|(_, positions)| positions.clone())
            .unwrap_or_default()
    }
}

/// Tokenize a file based on its language
pub fn tokenize_file(path: &str, content: &str) -> TokenIndex {
    let lower = path.to_lowercase();

    let mut index = if lower.ends_with(".rs") {
        TokenExtractor::extract_rust_tokens(content)
    } else if lower.ends_with(".md") {
        TokenExtractor::extract_markdown_tokens(content)
    } else if lower.ends_with(".toml") {
        TokenExtractor::extract_toml_tokens(content)
    } else if lower.ends_with(".yaml") || lower.ends_with(".yml") {
        TokenExtractor::extract_yaml_tokens(content)
    } else {
        TokenExtractor::extract_generic_tokens(content)
    };

    index.file_path = path.to_string();
    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rust_tokens() {
        let code = r#"
        fn main() {
            // This is a comment
            let x = 42;
        }
        "#;

        let index = TokenExtractor::extract_rust_tokens(code);
        assert!(index.frequency("fn") > 0);
        assert!(index.frequency("main") > 0);
    }

    #[test]
    fn test_extract_markdown_tokens() {
        let markdown = "# Hello\nThis is a test document\n## Subsection";
        let index = TokenExtractor::extract_markdown_tokens(markdown);
        assert!(!index.headings.is_empty());
        assert!(index.headings.iter().any(|h| h.contains("Hello")));
    }

    #[test]
    fn test_extract_toml_tokens() {
        let toml = "[section]\nkey = \"value\"\n";
        let index = TokenExtractor::extract_toml_tokens(toml);
        assert!(!index.config_keys.is_empty());
    }

    #[test]
    fn test_token_index_frequency() {
        let mut index = TokenIndex::default();
        index.add_token("test", 1, 0);
        index.add_token("test", 2, 5);
        assert_eq!(index.frequency("test"), 2);
    }
}
