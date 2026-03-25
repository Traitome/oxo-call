---
name: grep
category: text-processing
description: Print lines matching a pattern — search plain-text data using regular expressions
tags: [search, pattern, regex, text, filter, lines, files]
author: oxo-call built-in
source_url: "https://www.gnu.org/software/grep/manual/grep.html"
---

## Concepts

- grep searches each input file for lines containing a match to the PATTERN and prints matching lines. Syntax: 'grep [options] PATTERN [file...]'. Use '-r' to search directories recursively.
- Pattern types: default BRE (basic regex), -E for ERE/extended regex (enables +, ?, |, () without escaping), -F for fixed string (no regex, fastest), -P for Perl-compatible regex (PCRE).
- Key options: -i (ignore case), -n (show line numbers), -c (count matches per file), -l (list matching filenames only), -v (invert: show non-matching lines), -o (print only the matching part).
- Context options: -A N (N lines after match), -B N (N lines before match), -C N (N lines before and after). Useful for log analysis and code search.
- Use -r / -R for recursive directory search. Combine with --include='*.py' to restrict to specific file types. Use --exclude-dir='.git' to skip directories.
- grep returns exit code 0 if matches found, 1 if no matches, 2 on error. This makes it useful in shell conditionals: 'if grep -q pattern file; then ...; fi'.
- Always use the exact pattern and filename values from the task description — never substitute generic placeholder names like 'keyword', 'file', or 'pattern'.

## Pitfalls

- Patterns with special regex characters (., *, [, ], ^, $, \, (, ), {, }) must be escaped or use -F for literal matching. E.g., to search for '1.2.3', use 'grep -F "1.2.3"' or 'grep "1\.2\.3"'.
- grep without -r searches only listed files, not subdirectories. For recursive search use 'grep -r PATTERN dir/' or 'grep -rn PATTERN .'.
- Without quoting, shell metacharacters in the pattern may be expanded before grep sees them. Always quote the PATTERN: grep 'my pattern' file, not grep my pattern file.
- -i (case-insensitive) can significantly slow searches on large files with many matches; use with --max-count if only checking existence.
- grep -v prints non-matching lines — it does NOT delete lines. To remove lines from a file, use sed: 'sed -i '/pattern/d' file'.
- For counting total matches (not matching lines), use 'grep -o pattern file | wc -l' since -c counts matching LINES, not occurrences.
- Never use placeholder values like 'keyword', 'pattern', or 'file.txt' in the output — always use the specific strings and filenames from the task description.

## Examples

### search for a keyword in a file, ignoring case, with line numbers
**Args:** `-in "error" application.log`
**Explanation:** -i case-insensitive; -n prints line numbers; searches for the exact string "error" in application.log

### recursively search all Python files for a function definition
**Args:** `-rn "def connect" --include='*.py' src/`
**Explanation:** -r recursive; -n line numbers; --include restricts to .py files; searches src/ directory

### show context lines around each match
**Args:** `-C 3 "NullPointerException" error.log`
**Explanation:** -C 3 shows 3 lines before and after each match; useful for reading log context

### count the number of matching lines in a file
**Args:** `-c "^ERROR" server.log`
**Explanation:** -c prints only the count of matching lines; ^ anchors to start of line

### search for multiple patterns using extended regex
**Args:** `-E "(error|warning|fatal)" app.log`
**Explanation:** -E enables extended regex; | is the alternation operator; matches lines with any of the three words

### find files containing a pattern (list filenames only)
**Args:** `-rl "TODO" src/`
**Explanation:** -r recursive; -l lists only filenames with matches, not the matching lines themselves

### invert match: show lines that do NOT contain a pattern
**Args:** `-v "^#" config.ini`
**Explanation:** -v inverts the match; ^# matches comment lines so -v shows all non-comment lines

### extract only the matching part of each line
**Args:** `-oE "[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+" access.log`
**Explanation:** -o prints only the matched text; -E enables extended regex for the IP address pattern

### search in multiple files and show filename with each match
**Args:** `-Hn "import" *.py`
**Explanation:** -H always shows the filename (default when multiple files); -n shows line numbers

### search for a fixed string (no regex interpretation)
**Args:** `-F "error[0]" debug.log`
**Explanation:** -F treats the pattern as a literal string; brackets are NOT interpreted as regex character classes

### search for a pattern and show only the first 5 matches
**Args:** `-m 5 "WARN" application.log`
**Explanation:** -m 5 stops after finding 5 matching lines; useful for large log files

### search recursively excluding a directory
**Args:** `-rn "TODO" --exclude-dir='.git' --exclude-dir='node_modules' .`
**Explanation:** --exclude-dir skips the named directories during recursive search

### find lines where a specific column matches a value
**Args:** `-P "^\S+\s+200\s" access.log`
**Explanation:** -P enables Perl regex; matches lines where the second whitespace-separated field is exactly 200

### search for word boundary match (whole word only)
**Args:** `-w "main" *.c`
**Explanation:** -w matches only complete words; avoids matching 'main' inside 'maintain' or 'domain'

### binary search: check if a pattern exists (no output, use exit code)
**Args:** `-q "SUCCESS" results.log`
**Explanation:** -q suppresses all output; exit code 0 if found, 1 if not; use in shell scripts: 'if grep -q ...'
