---
name: cat
category: utilities
description: Concatenate files and print on the standard output. The cat command reads files sequentially and writes their contents to stdout in the order given. It is commonly used to display file contents, combine multiple files, and redirect output to new files.
tags: [text-processing, file-manipulation, concatenation, stdout, io-redirection]
author: AI-generated
source_url: https://www.gnu.org/software/coreutils/manual/html_node/cat-invocation.html
---

## Concepts

- `cat` reads each file in sequence from left to right and writes the complete contents to standard output. When no files are specified, it reads from standard input, allowing it to be used in pipelines.
- The tool processes data byte-by-byte without modification, preserving all characters including newlines, tabs, and whitespace. It does not perform any formatting, sorting, or filtering.
- Multiple files can be concatenated in a single invocation by specifying them as arguments; the output will be the logical combination of all input files in the order provided.

## Pitfalls

- Using `cat` on large files when you only need to view the beginning causes unnecessary I/O overhead. The entire file is read and written even if only the first few lines are needed, wasting memory and time.
- If `cat` is used without redirection on a very large file that exceeds terminal buffer capacity, important output at the beginning may scroll out of view before you can read it.
- When globbing patterns (e.g., `cat *.txt`) are used and no files match the pattern, `cat` will read from stdin interactively, causing the terminal to appear frozen while waiting for input.

## Examples

### Display the contents of a single file
**Args:** `example.txt`
**Explanation:** Reads and outputs the entire contents of example.txt to the terminal, allowing quick inspection of the file's complete contents.

### Combine three files into one output
**Args:** `header.txt data.txt footer.txt`
**Explanation:** Concatenates the three files in order and writes the combined output to stdout, useful for assembling document sections.

### Create a new file from multiple inputs
**Args:** `part1.txt part2.txt part3.txt > merged.txt`
**Explanation:** Redirects the concatenated output to a new file named merged.txt, effectively combining multiple files into one.

### Number all output lines
**Args:** `-n input.txt`
**Explanation:** Precedes each line with its line number, making it easier to reference specific lines when analyzing files.

### Show non-printing characters visibly
**Args:** `-A file.txt`
**Explanation:** Displays non-printing characters such as tabs as `^I` and end-of-line as `$`, helpful for debugging whitespace and formatting issues.

### Display multiple files with separators
**Args:** `-s file1.txt file2.txt`
**Explanation:** Squeezes multiple consecutive blank lines into a single blank line, reducing visual clutter in files with excessive spacing.