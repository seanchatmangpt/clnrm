//! Formatter Trait
//!
//! Core trait defining the contract for all test output formatters.
//! Follows Chicago School TDD principles with clear collaboration contracts.

use crate::error::Result;
use crate::formatting::test_result::TestSuite;

/// Type of formatter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatterType {
    /// Human-readable terminal output (default)
    Human,
    /// Structured JSON output
    Json,
    /// JUnit XML format for CI integration
    Junit,
    /// Test Anything Protocol (TAP) format
    Tap,
}

impl FormatterType {
    /// Parse formatter type from string
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "human" | "h" => Some(Self::Human),
            "json" | "j" => Some(Self::Json),
            "junit" | "xml" => Some(Self::Junit),
            "tap" | "t" => Some(Self::Tap),
            _ => None,
        }
    }

    /// Get the default file extension for this formatter
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Human => "txt",
            Self::Json => "json",
            Self::Junit => "xml",
            Self::Tap => "tap",
        }
    }

    /// Get formatter type name
    pub fn name(&self) -> &'static str {
        match self {
            Self::Human => "human",
            Self::Json => "json",
            Self::Junit => "junit",
            Self::Tap => "tap",
        }
    }
}

// ---------------------------------------------------------------------------
// TableFormatter – ASCII table with padded cells
// ---------------------------------------------------------------------------

/// Builds a fixed-width ASCII table from headers + rows
#[derive(Debug, Clone)]
pub struct TableFormatter {
    /// Column headers
    pub headers: Vec<String>,
    /// Data rows
    pub rows: Vec<Vec<String>>,
    /// Per-column minimum widths (updated as rows are added)
    pub col_widths: Vec<usize>,
}

impl TableFormatter {
    /// Create a new table with the given column headers.
    ///
    /// Column widths are initialized to the header lengths.
    pub fn new(headers: Vec<String>) -> Self {
        let col_widths = headers.iter().map(|h| h.len()).collect();
        Self {
            headers,
            rows: Vec::new(),
            col_widths,
        }
    }

    /// Append a row, expanding column widths if any cell is wider than the
    /// current column width.  Returns `&mut Self` for chaining.
    pub fn add_row(&mut self, row: Vec<String>) -> &mut Self {
        for (i, cell) in row.iter().enumerate() {
            if i < self.col_widths.len() && cell.len() > self.col_widths[i] {
                self.col_widths[i] = cell.len();
            }
        }
        self.rows.push(row);
        self
    }

    /// Render the table to an ASCII string.
    ///
    /// Example output:
    /// ```text
    /// +-------+------+
    /// | Name  | Age  |
    /// +-------+------+
    /// | Alice | 30   |
    /// +-------+------+
    /// ```
    pub fn render(&self) -> String {
        let separator = self.build_separator();
        let mut out = String::new();

        // Top border
        out.push_str(&separator);
        out.push('\n');

        // Header row
        out.push_str(&self.format_row(&self.headers));
        out.push('\n');

        // Header separator
        out.push_str(&separator);
        out.push('\n');

        // Data rows
        for row in &self.rows {
            out.push_str(&self.format_row(row));
            out.push('\n');

            out.push_str(&separator);
            out.push('\n');
        }

        out
    }

    fn build_separator(&self) -> String {
        let mut sep = String::from("+");
        for &w in &self.col_widths {
            sep.push_str(&"-".repeat(w + 2));
            sep.push('+');
        }
        sep
    }

    fn format_row(&self, row: &[String]) -> String {
        let mut line = String::from("|");
        for (i, &w) in self.col_widths.iter().enumerate() {
            let cell = row.get(i).map(|s| s.as_str()).unwrap_or("");
            line.push_str(&format!(" {:<width$} |", cell, width = w));
        }
        line
    }
}

// ---------------------------------------------------------------------------
// Standalone formatting helpers
// ---------------------------------------------------------------------------

/// Format a duration given in milliseconds into a human-readable string.
///
/// - ≥ 1 000 ms → `"1.2s"`
/// - ≥ 1 ms     → `"234ms"`
/// - < 1 ms     → `"45μs"`
pub fn format_duration_ms(ms: u64) -> String {
    if ms >= 1_000 {
        format!("{:.1}s", ms as f64 / 1_000.0)
    } else {
        format!("{}ms", ms)
    }
}

/// Format a duration given in **microseconds** into a human-readable string.
///
/// Exposed so callers can obtain `"45μs"` style output without floating-point
/// millisecond fractions.
pub fn format_duration_us(us: u64) -> String {
    if us >= 1_000_000 {
        format!("{:.1}s", us as f64 / 1_000_000.0)
    } else if us >= 1_000 {
        format!("{}ms", us / 1_000)
    } else {
        format!("{}μs", us)
    }
}

/// Format a byte count into a human-readable string.
///
/// - ≥ 1 GB → `"1.2 GB"`
/// - ≥ 1 MB → `"234 MB"`
/// - ≥ 1 KB → `"45 KB"`
/// - otherwise → `"12 B"`
pub fn format_bytes(bytes: u64) -> String {
    const GB: u64 = 1_073_741_824;
    const MB: u64 = 1_048_576;
    const KB: u64 = 1_024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.0} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.0} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format a percentage as `"73.4%"`.
///
/// If `total` is zero, returns `"0.0%"`.
pub fn format_percentage(value: f64, total: f64) -> String {
    if total == 0.0 {
        return "0.0%".to_string();
    }
    format!("{:.1}%", value / total * 100.0)
}

/// Truncate a string to `max_len` characters, appending `"..."` if truncated.
///
/// If `s.len() <= max_len` the original string is returned unchanged.
pub fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        "...".chars().take(max_len).collect()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Apply terminal color to a string using the `colored` crate.
///
/// Supported `color` values: `"green"`, `"red"`, `"yellow"`, `"blue"`.
/// Any unknown color returns the string unmodified.
pub fn colorize(s: &str, color: &str) -> String {
    use colored::Colorize;
    match color {
        "green" => s.green().to_string(),
        "red" => s.red().to_string(),
        "yellow" => s.yellow().to_string(),
        "blue" => s.blue().to_string(),
        _ => s.to_string(),
    }
}

// ---------------------------------------------------------------------------

/// Formatter trait for test output
///
/// All formatters must implement this trait to provide consistent output generation.
/// This trait defines the collaboration contract between the test runner and formatters.
///
/// # TDD Note
/// This trait is designed to support testability. Implementations are
/// independently testable.
pub trait Formatter: Send + Sync {
    /// Format a test suite into a string
    ///
    /// # Arguments
    /// * `suite` - Test suite containing test results
    ///
    /// # Returns
    /// * `Result<String>` - Formatted output string
    ///
    /// # Errors
    /// Returns error if formatting fails (e.g., serialization errors)
    fn format(&self, suite: &TestSuite) -> Result<String>;

    /// Get the formatter name
    fn name(&self) -> &'static str;

    /// Get the formatter type
    fn formatter_type(&self) -> FormatterType;
}
