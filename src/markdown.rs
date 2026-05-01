//! Terminal markdown rendering with professional styling.
//!
//! This module provides a configured `MadSkin` for rendering markdown
//! content in the terminal with proper styling for:
//! - Headers (colored, bold)
//! - Code blocks (indented with left margin, styled)
//! - Inline code (highlighted)
//! - Lists (proper indentation and bullets)
//! - Tables (styled)
//! - Blockquotes (styled markers)

use termimad::{
    CompoundStyle, LineStyle, MadSkin, StyledChar,
    crossterm::style::{Attribute, Color},
};

/// Create a professionally configured MadSkin for terminal markdown rendering.
fn create_skin() -> MadSkin {
    let mut skin = MadSkin::default();

    // Headers: cyan color, bold, for all 8 levels
    for header in skin.headers.iter_mut() {
        header.set_fg(Color::Cyan);
        header.add_attr(Attribute::Bold);
    }

    // Code blocks: gray background, left margin for visual indentation
    skin.code_block = LineStyle {
        compound_style: CompoundStyle::new(
            Some(Color::White),
            Some(Color::DarkGrey),
            Attribute::Bold.into(),
        ),
        align: termimad::Alignment::Left,
        left_margin: 2,
        right_margin: 2,
    };

    // Inline code: yellow foreground, dark background
    skin.inline_code = CompoundStyle::new(
        Some(Color::Yellow),
        Some(Color::DarkGrey),
        Attribute::Bold.into(),
    );

    // Bold: white, bold
    skin.bold = CompoundStyle::new(Some(Color::White), None, Attribute::Bold.into());

    // Italic: grey
    skin.italic = CompoundStyle::new(Some(Color::Grey), None, Attribute::Italic.into());

    // Bullet: green arrow for better visibility
    skin.bullet = StyledChar::from_fg_char(Color::Green, '▸');

    // Quote mark: dimmed vertical bar
    skin.quote_mark = StyledChar::from_fg_char(Color::Grey, '▌');

    // Horizontal rule: dimmed line
    skin.horizontal_rule = StyledChar::from_fg_char(Color::DarkGrey, '─');

    // Table: grey styled
    skin.table = LineStyle {
        compound_style: CompoundStyle::new(Some(Color::Grey), None, Default::default()),
        align: termimad::Alignment::Left,
        left_margin: 0,
        right_margin: 0,
    };

    // List indentation: indent entire block, not just first line
    skin.list_items_indentation_mode = termimad::ListItemsIndentationMode::Block;

    skin
}

/// Global skin instance (lazy initialization)
static SKIN: std::sync::OnceLock<MadSkin> = std::sync::OnceLock::new();

/// Get the global configured MadSkin instance.
fn get_skin() -> &'static MadSkin {
    SKIN.get_or_init(create_skin)
}

/// Pre-process text to escape asterisks that are likely glob patterns
/// rather than markdown emphasis markers.
///
/// Handles: `*.log`, `*.bam`, `*.txt`, etc.
/// Preserves: code blocks, inline code, bold (`**`), list items (`* `).
fn escape_glob_asterisks(text: &str) -> String {
    let mut result = String::with_capacity(text.len() + 64);
    let mut in_code_block = false;
    let mut chars = text.char_indices().peekable();
    let bytes = text.as_bytes();

    while let Some((i, ch)) = chars.next() {
        if in_code_block {
            result.push(ch);
            // Check for closing code fence
            if ch == '`' && i + 2 < text.len() && &bytes[i..i + 3] == b"```" {
                in_code_block = false;
                // Skip the remaining backticks
                chars.next(); // second `
                chars.next(); // third `
                result.push_str("``");
            }
            continue;
        }

        // Check for opening code fence
        if ch == '`' && i + 2 < text.len() && &bytes[i..i + 3] == b"```" {
            in_code_block = true;
            result.push_str("```");
            chars.next(); // second `
            chars.next(); // third `
            continue;
        }

        // Skip inline code spans
        if ch == '`' {
            result.push(ch);
            // Copy everything until closing backtick
            for (_, inner_ch) in chars.by_ref() {
                result.push(inner_ch);
                if inner_ch == '`' {
                    break;
                }
            }
            continue;
        }

        if ch == '*' {
            // Check for bold marker `**`
            let next_is_star = chars.peek().map(|(_, c)| *c == '*').unwrap_or(false);

            if next_is_star {
                // Bold marker — keep as-is
                result.push_str("**");
                chars.next(); // consume the second *
                continue;
            }

            // Check for list item: `* ` at start of line
            let at_line_start = i == 0 || text[..i].ends_with('\n');
            if at_line_start && chars.peek().map(|(_, c)| *c == ' ').unwrap_or(false) {
                result.push('*');
                continue;
            }

            // Check for glob pattern: `*.` (e.g., `*.log`, `*.bam`)
            let next_is_dot = chars.peek().map(|(_, c)| *c == '.').unwrap_or(false);
            if next_is_dot {
                result.push_str("\\*");
                continue;
            }

            // Check for glob pattern: `*/` (e.g., `*/path`)
            let next_is_slash = chars.peek().map(|(_, c)| *c == '/').unwrap_or(false);
            if next_is_slash {
                result.push_str("\\*");
                continue;
            }

            // Default: keep as-is (valid emphasis or other markdown)
            result.push('*');
        } else {
            result.push(ch);
        }
    }

    result
}

/// Render markdown text to the terminal with professional styling.
///
/// This function prints the markdown content directly to stdout
/// with proper styling for headers, code blocks, lists, tables, etc.
/// Asterisks in glob patterns (e.g., `*.log`) are automatically escaped
/// to prevent misinterpretation as italic markers.
pub fn render_markdown(text: &str) {
    let skin = get_skin();
    let processed = escape_glob_asterisks(text);
    skin.print_text(&processed);
}

/// Render markdown text to a string with ANSI escape codes.
///
/// Use this when you need to capture the rendered output instead of
/// printing directly to stdout.
#[allow(dead_code)]
pub fn render_markdown_to_string(text: &str) -> String {
    let skin = get_skin();
    let processed = escape_glob_asterisks(text);
    skin.term_text(&processed).to_string()
}

/// Render only the first few lines of markdown (for previews).
///
/// Truncates at line boundaries to avoid cutting in the middle of
/// a markdown element.
#[allow(dead_code)]
pub fn render_markdown_preview(text: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = text.lines().take(max_lines).collect();
    let preview = lines.join("\n");
    if text.lines().count() > max_lines {
        render_markdown_to_string(&format!("{}\n...", preview))
    } else {
        render_markdown_to_string(&preview)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_markdown_headers() {
        render_markdown("# Heading 1\n## Heading 2\n### Heading 3");
    }

    #[test]
    fn test_render_markdown_code_blocks() {
        render_markdown("```bash\nsamtools sort input.bam -o output.bam\n```");
        render_markdown("```rust\nfn main() {\n    println!(\"Hello\");\n}\n```");
    }

    #[test]
    fn test_render_markdown_inline_code() {
        render_markdown("Use `samtools sort` to sort BAM files.");
    }

    #[test]
    fn test_render_markdown_lists() {
        render_markdown("- Item 1\n- Item 2\n- Item 3");
        render_markdown("1. First\n2. Second\n3. Third");
        render_markdown("- Parent\n  - Child 1\n  - Child 2");
    }

    #[test]
    fn test_render_markdown_tables() {
        render_markdown(
            "| Tool | Purpose |\n\
             |------|--------|\n\
             | samtools | BAM manipulation |\n\
             | bwa | Read alignment |",
        );
    }

    #[test]
    fn test_render_markdown_blockquotes() {
        render_markdown("> This is a quote.\n> Multiple lines.");
    }

    #[test]
    fn test_render_markdown_mixed() {
        render_markdown(
            "# Bioinformatics Pipeline\n\n\
             This pipeline uses **samtools** for BAM processing.\n\n\
             ## Steps\n\n\
             1. Sort BAM: `samtools sort input.bam`\n\
             2. Index BAM: `samtools index sorted.bam`\n\n\
             ```bash\n\
             # Example command\n\
             samtools view -b input.bam > output.bam\n\
             ```\n\n\
             > Note: Always check file integrity.",
        );
    }

    #[test]
    fn test_render_markdown_empty() {
        render_markdown("");
        render_markdown("\n\n\n");
    }

    #[test]
    fn test_render_markdown_to_string() {
        let output = render_markdown_to_string("# Test\n\n**Bold** text.");
        assert!(output.contains("Test"));
        assert!(output.contains("Bold"));
    }

    #[test]
    fn test_render_markdown_preview() {
        let text = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5";
        let preview = render_markdown_preview(text, 3);
        assert!(preview.contains("Line 1"));
        assert!(preview.contains("Line 2"));
        assert!(preview.contains("Line 3"));
        assert!(preview.contains("..."));
    }

    #[test]
    fn test_render_markdown_preview_short() {
        let text = "Line 1\nLine 2";
        let preview = render_markdown_preview(text, 5);
        assert!(preview.contains("Line 1"));
        assert!(preview.contains("Line 2"));
        assert!(!preview.contains("..."));
    }

    #[test]
    fn test_skin_is_cached() {
        let skin1 = get_skin();
        let skin2 = get_skin();
        assert!(std::ptr::eq(skin1, skin2));
    }

    #[test]
    fn test_escape_glob_asterisks() {
        // Glob patterns should be escaped
        assert_eq!(escape_glob_asterisks("*.log"), "\\*.log");
        assert_eq!(escape_glob_asterisks("*.bam"), "\\*.bam");
        assert_eq!(escape_glob_asterisks("*.txt"), "\\*.txt");
        assert_eq!(escape_glob_asterisks("*/path"), "\\*/path");

        // Bold markers should NOT be escaped
        assert_eq!(escape_glob_asterisks("**bold**"), "**bold**");

        // List items should NOT be escaped
        assert_eq!(escape_glob_asterisks("* item"), "* item");
        assert_eq!(escape_glob_asterisks("\n* item"), "\n* item");

        // Code blocks should NOT be modified
        assert_eq!(
            escape_glob_asterisks("```bash\n*.log\n```"),
            "```bash\n*.log\n```"
        );

        // Inline code should NOT be modified
        assert_eq!(escape_glob_asterisks("`*.log`"), "`*.log`");

        // Mixed content
        assert_eq!(
            escape_glob_asterisks("Use *.bam files with **samtools**"),
            "Use \\*.bam files with **samtools**"
        );

        // User's example: find command with glob pattern
        assert_eq!(
            escape_glob_asterisks("find /var/log/ -name \"*.log\" -exec mv {} /backup \\;"),
            "find /var/log/ -name \"\\*.log\" -exec mv {} /backup \\;"
        );
    }

    #[test]
    fn test_render_markdown_to_string_various_elements() {
        let cases: Vec<(&str, &str)> = vec![
            ("# Header 1", "Header 1"),
            ("**bold text**", "bold text"),
            ("- list item", "list item"),
            ("`inline code`", "inline code"),
            ("> blockquote", "blockquote"),
        ];
        for (input, expected_fragment) in cases {
            let output = render_markdown_to_string(input);
            assert!(
                output.contains(expected_fragment),
                "render_markdown_to_string({input:?}) should contain {expected_fragment:?}"
            );
        }
    }

    #[test]
    fn test_escape_glob_table() {
        let cases: Vec<(&str, &str)> = vec![
            ("no wildcards", "no wildcards"),
            ("*.bam", "\\*.bam"),
            ("*.fastq.gz", "\\*.fastq.gz"),
            ("**bold**", "**bold**"),       // bold marker - not escaped
            ("* item", "* item"),            // list item - not escaped
            ("`*.log`", "`*.log`"),          // inline code - not escaped
        ];
        for (input, expected) in cases {
            assert_eq!(
                escape_glob_asterisks(input),
                expected,
                "escape_glob_asterisks({input:?}) should be {expected:?}"
            );
        }
    }

    #[test]
    fn test_render_markdown_preview_zero_max_lines() {
        let text = "Line 1\nLine 2\nLine 3";
        // Edge case: max_lines=0 → should not panic
        let _preview = render_markdown_preview(text, 0);
    }

    #[test]
    fn test_render_markdown_preview_exact_count() {
        let lines: Vec<String> = (1..=5).map(|i| format!("Line {i}")).collect();
        let text = lines.join("\n");
        // Request exactly the number of lines → no ellipsis
        let preview = render_markdown_preview(&text, 5);
        assert!(!preview.contains("..."), "should not have ellipsis when not truncated");
    }
}
