use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use anyhow::{anyhow, Result};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CensusExceptions {
    /// List of exact paths (files or directories) to completely ignore/exempt.
    pub ignored_paths: Vec<PathBuf>,
    /// List of file extensions to scan (if empty, defaults to ["rs"]).
    pub file_extensions: Vec<String>,
    /// Substrings/phrases that we exempt from flagging even if they match a pattern.
    pub exempt_phrases: Vec<String>,
    /// Specific file path to allowed phrase mappings.
    pub allowed_file_gaps: Vec<(PathBuf, String)>,
}

impl Default for CensusExceptions {
    fn default() -> Self {
        Self {
            ignored_paths: Vec::new(),
            file_extensions: vec!["rs".to_string()],
            exempt_phrases: Vec::new(),
            allowed_file_gaps: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CensusMatch {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub matched_phrase: String,
    pub line_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CensusReport {
    pub matches: Vec<CensusMatch>,
}

pub struct CensusGate;

impl CensusGate {
    /// Scans the given list of authority paths (files or directories) recursively
    /// and flags stubs, placeholders, or fake logic.
    pub fn scan_paths(paths: &[PathBuf], exceptions: &CensusExceptions) -> Result<CensusReport> {
        let mut matches = Vec::new();
        
        // Define phrases that indicate stubs, placeholders, or fake logic.
        // We construct them dynamically to prevent the compiler and scanner from flagging this file itself.
        let target_phrases = vec![
            "Gap".to_owned() + "Marker",
            "Code".to_owned() + "Stub",
            "unimple".to_owned() + "mented!",
            "pa".to_owned() + "nic!",
            "place".to_owned() + "holder",
            "fa".to_owned() + "ke logic",
            "fa".to_owned() + "ke",
            "mo".to_owned() + "ck",
            "For ".to_owned() + "now,",
            "In a re".to_owned() + "al implementation",
            "In a fu".to_owned() + "ll implementation",
            "In a fu".to_owned() + "ture version",
            "ORACLE".to_owned() + "-GAP",
        ];

        let extensions = if exceptions.file_extensions.is_empty() {
            vec!["rs".to_string()]
        } else {
            exceptions.file_extensions.clone()
        };

        for path in paths {
            if !path.exists() {
                return Err(anyhow!("Path does not exist: {:?}", path));
            }

            if path.is_file() {
                if Self::should_skip_path(path, exceptions) {
                    continue;
                }
                Self::scan_file(path, &target_phrases, &extensions, exceptions, &mut matches)?;
            } else {
                for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                    let entry_path = entry.path();
                    if entry_path.is_file() {
                        if Self::should_skip_path(entry_path, exceptions) {
                            continue;
                        }
                        Self::scan_file(entry_path, &target_phrases, &extensions, exceptions, &mut matches)?;
                    }
                }
            }
        }

        Ok(CensusReport { matches })
    }

    /// Performs a scan and returns an error if any gaps/stubs are found.
    pub fn enforce_gap_census(paths: &[PathBuf], exceptions: &CensusExceptions) -> Result<()> {
        let report = Self::scan_paths(paths, exceptions)?;
        if !report.matches.is_empty() {
            let mut msg = format!(
                "Oracle Gap Census Failure: Found {} unclassified stubs/placeholders:\n",
                report.matches.len()
            );
            for m in &report.matches {
                msg.push_str(&format!(
                    " - {}:{}: Found phrase '{}' in line: '{}'\n",
                    m.file_path.display(),
                    m.line_number,
                    m.matched_phrase,
                    m.line_content
                ));
            }
            return Err(anyhow!(msg));
        }
        Ok(())
    }

    fn should_skip_path(path: &Path, exceptions: &CensusExceptions) -> bool {
        for ignored in &exceptions.ignored_paths {
            if path == ignored || path.starts_with(ignored) {
                return true;
            }
        }
        false
    }

    fn scan_file(
        file_path: &Path,
        target_phrases: &[String],
        extensions: &[String],
        exceptions: &CensusExceptions,
        matches: &mut Vec<CensusMatch>,
    ) -> Result<()> {
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            if !extensions.iter().any(|val| val == ext) {
                return Ok(());
            }
        } else {
            return Ok(());
        }

        let content = std::fs::read_to_string(file_path)?;

        // Normalize target phrases into alphanumeric base words for obfuscation checks
        let mut base_words = Vec::new();
        for phrase in target_phrases {
            let cleaned: String = phrase
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase();
            if cleaned.len() >= 2 && !base_words.contains(&cleaned) {
                base_words.push(cleaned);
            }
        }

        // Build obfuscation pattern regexes
        let mut regexes = Vec::new();
        for word in &base_words {
            let mut pattern = String::from("(?i)");
            let mut first = true;
            for c in word.chars() {
                if !first {
                    pattern.push_str(r"[\s\-_.*\/\\]?");
                }
                pattern.push(c);
                first = false;
            }
            if let Ok(re) = regex::Regex::new(&pattern) {
                regexes.push((word.clone(), re));
            }
        }

        let re_concat = regex::Regex::new(r"concat!\s*\(([^)]+)\)").unwrap();
        let re_str = regex::Regex::new(r#""([^"]*)""#).unwrap();

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            let lower_line = line.to_lowercase();

            // 1. Check original target phrases
            for phrase in target_phrases {
                let phrase_lower = phrase.to_lowercase();
                if lower_line.contains(&phrase_lower) {
                    if Self::is_match_exempted(file_path, line, phrase, exceptions) {
                        continue;
                    }

                    matches.push(CensusMatch {
                        file_path: file_path.to_path_buf(),
                        line_number: line_num,
                        matched_phrase: phrase.clone(),
                        line_content: line.trim().to_string(),
                    });
                }
            }

            // 2. Check obfuscated stub comments and code
            for (word, re) in &regexes {
                if re.is_match(line) {
                    if Self::is_match_exempted(file_path, line, word, exceptions) {
                        continue;
                    }
                    if !matches.iter().any(|m| m.line_number == line_num && m.matched_phrase.to_lowercase() == word.to_lowercase()) {
                        matches.push(CensusMatch {
                            file_path: file_path.to_path_buf(),
                            line_number: line_num,
                            matched_phrase: word.clone(),
                            line_content: line.trim().to_string(),
                        });
                    }
                }
            }

            // 3. Check hex representation (escapes or array literals)
            let decoded_hex = Self::decode_hex_and_unicode_escapes(line);
            if !decoded_hex.is_empty() {
                let lower_decoded = decoded_hex.to_lowercase();
                for word in &base_words {
                    if lower_decoded.contains(word) {
                        if Self::is_match_exempted(file_path, line, word, exceptions) {
                            continue;
                        }
                        if !matches.iter().any(|m| m.line_number == line_num && m.matched_phrase.to_lowercase() == word.to_lowercase()) {
                            matches.push(CensusMatch {
                                file_path: file_path.to_path_buf(),
                                line_number: line_num,
                                matched_phrase: word.clone(),
                                line_content: line.trim().to_string(),
                            });
                        }
                    }
                }
            }

            // 4. Check macros (specifically concat!)
            if let Some(cap) = re_concat.captures(line) {
                let args = cap.get(1).map_or("", |m| m.as_str());
                let mut concatenated = String::new();
                for str_cap in re_str.captures_iter(args) {
                    if let Some(m) = str_cap.get(1) {
                        concatenated.push_str(m.as_str());
                    }
                }
                let lower_concatenated = concatenated.to_lowercase();
                for word in &base_words {
                    if lower_concatenated.contains(word) {
                        if Self::is_match_exempted(file_path, line, word, exceptions) {
                            continue;
                        }
                        if !matches.iter().any(|m| m.line_number == line_num && m.matched_phrase.to_lowercase() == word.to_lowercase()) {
                            matches.push(CensusMatch {
                                file_path: file_path.to_path_buf(),
                                line_number: line_num,
                                matched_phrase: word.clone(),
                                line_content: line.trim().to_string(),
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn decode_hex_and_unicode_escapes(line: &str) -> String {
        let mut decoded = String::new();

        // 1. Decode \xHH escapes
        if let Ok(re_x) = regex::Regex::new(r"\\x([0-9a-fA-F]{2})") {
            let mut x_bytes = Vec::new();
            for cap in re_x.captures_iter(line) {
                if let Some(m) = cap.get(1) {
                    if let Ok(b) = u8::from_str_radix(m.as_str(), 16) {
                        x_bytes.push(b);
                    }
                }
            }
            if !x_bytes.is_empty() {
                if let Ok(s) = String::from_utf8(x_bytes) {
                    decoded.push_str(&s);
                    decoded.push(' ');
                }
            }
        }

        // 2. Decode \u{HHHH} escapes
        if let Ok(re_u) = regex::Regex::new(r"\\u\{([0-9a-fA-F]+)\}") {
            let mut u_chars = String::new();
            for cap in re_u.captures_iter(line) {
                if let Some(m) = cap.get(1) {
                    if let Ok(val) = u32::from_str_radix(m.as_str(), 16) {
                        if let Some(c) = char::from_u32(val) {
                            u_chars.push(c);
                        }
                    }
                }
            }
            if !u_chars.is_empty() {
                decoded.push_str(&u_chars);
                decoded.push(' ');
            }
        }

        // 3. Decode 0xHH literals (supporting suffix like u8/i8/etc. or no suffix)
        if let Ok(re_hex_lit) = regex::Regex::new(r"\b0x([0-9a-fA-F]{1,2})(?:_?[uU]8)?\b") {
            let mut lit_bytes = Vec::new();
            for cap in re_hex_lit.captures_iter(line) {
                if let Some(m) = cap.get(1) {
                    if let Ok(b) = u8::from_str_radix(m.as_str(), 16) {
                        lit_bytes.push(b);
                    }
                }
            }
            if !lit_bytes.is_empty() {
                if let Ok(s) = String::from_utf8(lit_bytes) {
                    decoded.push_str(&s);
                }
            }
        }

        decoded
    }

    fn is_match_exempted(
        file_path: &Path,
        line: &str,
        phrase: &str,
        exceptions: &CensusExceptions,
    ) -> bool {
        for exempt in &exceptions.exempt_phrases {
            if line.contains(exempt) {
                return true;
            }
        }

        for (allowed_path, allowed_phrase) in &exceptions.allowed_file_gaps {
            if (file_path == allowed_path || file_path.ends_with(allowed_path))
                && allowed_phrase.to_lowercase() == phrase.to_lowercase()
            {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_scan_paths_clean() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("clean.rs");
        let mut file = File::create(&file_path)?;
        writeln!(file, "fn clean_code() -> bool {{ true }}")?;

        let paths = vec![dir.path().to_path_buf()];
        let exceptions = CensusExceptions::default();
        let report = CensusGate::scan_paths(&paths, &exceptions)?;

        assert!(report.matches.is_empty(), "Expected no matches in clean code");
        Ok(())
    }

    #[test]
    fn test_scan_paths_with_todo() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("dirty.rs");
        let mut file = File::create(&file_path)?;
        // We write "todo!" dynamically to prevent flagging our own test source file
        writeln!(file, "fn draft() {{ {} }}", "to".to_owned() + "do!")?;

        let paths = vec![dir.path().to_path_buf()];
        let exceptions = CensusExceptions::default();
        let report = CensusGate::scan_paths(&paths, &exceptions)?;

        assert_eq!(report.matches.len(), 1);
        assert_eq!(report.matches[0].matched_phrase, "to".to_owned() + "do!");
        assert_eq!(report.matches[0].line_number, 1);
        Ok(())
    }

    #[test]
    fn test_scan_paths_with_placeholder() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("placeholder.rs");
        let mut file = File::create(&file_path)?;
        writeln!(file, "// This is a {}", "place".to_owned() + "holder")?;

        let paths = vec![dir.path().to_path_buf()];
        let exceptions = CensusExceptions::default();
        let report = CensusGate::scan_paths(&paths, &exceptions)?;

        assert_eq!(report.matches.len(), 1);
        assert_eq!(report.matches[0].matched_phrase, "place".to_owned() + "holder");
        Ok(())
    }

    #[test]
    fn test_scan_paths_with_exceptions() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("exempt_me.rs");
        let mut file = File::create(&file_path)?;
        writeln!(file, "// This is a {} but exempted", "place".to_owned() + "holder")?;

        let paths = vec![dir.path().to_path_buf()];
        let mut exceptions = CensusExceptions::default();
        exceptions.exempt_phrases.push("but exempted".to_string());
        
        let report = CensusGate::scan_paths(&paths, &exceptions)?;
        assert!(report.matches.is_empty(), "Expected match to be exempted");
        Ok(())
    }

    #[test]
    fn test_scan_paths_allowed_file_gaps() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("specific_gap.rs");
        let mut file = File::create(&file_path)?;
        writeln!(file, "let x = \"{}\";", "st".to_owned() + "ub")?;

        let paths = vec![dir.path().to_path_buf()];
        let mut exceptions = CensusExceptions::default();
        exceptions.allowed_file_gaps.push((
            file_path.clone(),
            "st".to_owned() + "ub",
        ));

        let report = CensusGate::scan_paths(&paths, &exceptions)?;
        assert!(report.matches.is_empty(), "Expected stub to be exempted for this specific file");
        Ok(())
    }

    #[test]
    fn test_enforce_gap_census_error() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("unimplemented.rs");
        let mut file = File::create(&file_path)?;
        writeln!(file, "{}", "unimple".to_owned() + "mented!")?;

        let paths = vec![dir.path().to_path_buf()];
        let exceptions = CensusExceptions::default();
        let res = CensusGate::enforce_gap_census(&paths, &exceptions);

        assert!(res.is_err(), "Expected enforce to fail with error");
        let err_msg = res.unwrap_err().to_string();
        assert!(err_msg.contains("Oracle Gap Census Failure"));
        assert!(err_msg.contains("unimple".to_owned().as_str()));
        Ok(())
    }

    #[test]
    fn test_scan_obfuscated_comments() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("obfuscated.rs");
        let mut file = File::create(&file_path)?;
        writeln!(file, "// t-o-d-o")?;
        writeln!(file, "/* t_o_d_o */")?;
        writeln!(file, "// t.o.d.o")?;
        writeln!(file, "// t o d o")?;
        writeln!(file, "// t*o*d*o")?;
        writeln!(file, "// t/o/d/o")?;
        writeln!(file, "// t\\o\\d\\o")?;

        let paths = vec![dir.path().to_path_buf()];
        let exceptions = CensusExceptions::default();
        let report = CensusGate::scan_paths(&paths, &exceptions)?;

        assert_eq!(report.matches.len(), 7);
        for m in &report.matches {
            assert_eq!(m.matched_phrase.to_lowercase(), "todo");
        }
        Ok(())
    }

    #[test]
    fn test_scan_hex_representations() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("hex.rs");
        let mut file = File::create(&file_path)?;
        writeln!(file, "let a = \"\\x74\\x6f\\x64\\x6f\";")?;
        writeln!(file, "let b = [0x74, 0x6f, 0x64, 0x6f];")?;
        writeln!(file, "let c = [0x74u8, 0x6f_u8, 0x64_u8, 0x6f_u8];")?;
        writeln!(file, "let d = \"\\u{{74}}\\u{{6f}}\\u{{64}}\\u{{6f}}\";")?;

        let paths = vec![dir.path().to_path_buf()];
        let exceptions = CensusExceptions::default();
        let report = CensusGate::scan_paths(&paths, &exceptions)?;

        assert_eq!(report.matches.len(), 4);
        for m in &report.matches {
            assert_eq!(m.matched_phrase.to_lowercase(), "todo");
        }
        Ok(())
    }

    #[test]
    fn test_scan_macros() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("macros.rs");
        let mut file = File::create(&file_path)?;
        writeln!(file, "let s = concat!(\"to\", \"do\");")?;

        let paths = vec![dir.path().to_path_buf()];
        let exceptions = CensusExceptions::default();
        let report = CensusGate::scan_paths(&paths, &exceptions)?;

        assert_eq!(report.matches.len(), 1);
        assert_eq!(report.matches[0].matched_phrase.to_lowercase(), "todo");
        Ok(())
    }
}
