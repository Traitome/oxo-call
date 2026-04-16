---
name: sed
category: text-processing
description: Stream editor for filtering and transforming text — substitution, deletion, insertion
tags: [sed, text, replace, edit, stream, regex, transform]
author: oxo-call built-in
source_url: "https://www.gnu.org/software/sed/manual/sed.html"
---

## Concepts
- sed reads input line by line, applies the script, and writes to stdout. Use -i to edit files in-place. Basic syntax: 'sed [options] 'script' [file...]'.
- Substitution syntax: 's/PATTERN/REPLACEMENT/FLAGS'. Flags: g (replace all occurrences per line, not just first), i (case-insensitive), p (print if substitution made). Delimiter can be any char: 's|/path/|/newpath|g'.
- Address syntax: N (line number), $ (last line), /pattern/ (matching lines), N,M (range), /pat1/,/pat2/ (pattern range). No address = apply to all lines.
- Common commands: s (substitute), d (delete line), p (print line), a (append after), i (insert before), q (quit), y (transliterate). Chain with -e or separate by newline/semicolon.
- sed uses BRE by default; use -E (or -r on some systems) for extended regex (ERE) which allows +, ?, |, () without escaping.
- Capture groups: in BRE use \( \) and \1 \2 backreferences; in ERE (-E) use ( ) unescaped. E.g., sed -E 's/(foo)(bar)/\2\1/' swaps foo and bar.
- Multiple expressions: use -e for each expression, e.g., sed -e 's/a/b/' -e 's/c/d/' file. Or separate with semicolons: sed 's/a/b/;s/c/d/' file.
- The -n flag suppresses default output; only lines explicitly printed with p will appear. Combine -n with /pattern/p to mimic grep behavior.
- c command replaces entire lines; N command reads next line into pattern space.
- w command writes matching lines to a file.

## Pitfalls
- 'sed -i' modifies the file in-place without creating a backup by default. Use 'sed -i.bak' to save the original as a backup file before modifying.
- sed -i syntax differs between GNU sed and BSD sed (macOS): GNU accepts 'sed -i "s/x/y/"'; BSD requires 'sed -i "" "s/x/y/"'. For portability, use 'sed -i.bak'.
- The substitution pattern is a regex, not a fixed string: special chars (., *, [, ^, $, \) must be escaped. Use 'sed 's/1\.0/2.0/g'' to replace literal '1.0'.
- Without the 'g' flag, only the FIRST occurrence on each line is replaced. Add 'g' after the closing delimiter to replace all occurrences.
- Newlines in the replacement: sed cannot insert newlines with \n in the replacement on all platforms. Use $'\n' or printf for portability.
- sed 'd' deletes the entire matching line, not just the matching part. To delete only matched text, use 's/pattern//'.
- When using alternate delimiters (s|pat|repl|), ensure the delimiter does not appear in the pattern or replacement unescaped.
- c command replaces the entire line, not just the matched portion; use with caution.

## Examples

### replace all occurrences of a word in a file in-place
**Args:** `-i 's/foo/bar/g' file.txt`
**Explanation:** -i edits in-place; s/foo/bar/g replaces every occurrence of 'foo' with 'bar' on each line

### replace text in-place and keep a backup
**Args:** `-i.bak 's/old_host/new_host/g' config.conf`
**Explanation:** -i.bak modifies the file and saves original as config.conf.bak before changes

### delete all blank lines from a file
**Args:** `'/^$/d' file.txt`
**Explanation:** /^$/ matches empty lines; d deletes them; output goes to stdout (pipe or use -i)

### delete lines containing a pattern
**Args:** `-i '/^#/d' script.sh`
**Explanation:** deletes all lines starting with # (comment lines); -i modifies in-place

### print only lines matching a pattern (like grep)
**Args:** `-n '/error/p' app.log`
**Explanation:** -n suppresses default output; /error/p prints only lines that match 'error'

### extract and reformat date using capture groups
**Args:** `-E 's/([0-9]{4})-([0-9]{2})-([0-9]{2})/\3\/\2\/\1/' dates.txt`
**Explanation:** -E enables extended regex; capture groups rearrange YYYY-MM-DD to DD/MM/YYYY

### add a prefix to every line in a file
**Args:** `'s/^/PREFIX: /' input.txt`
**Explanation:** ^ matches start of line; the replacement inserts 'PREFIX: ' before every line

### remove trailing whitespace from all lines
**Args:** `-E 's/[[:space:]]+$//' file.txt`
**Explanation:** [[:space:]]+ matches one or more trailing whitespace characters; $ anchors to end of line

### replace only on a specific line number
**Args:** `'10s/old/new/' file.txt`
**Explanation:** address '10' restricts substitution to line 10 only

### insert a line after a matching pattern
**Args:** `'/^\[section\]/a new_key=value' config.ini`
**Explanation:** 'a' appends the text after every line matching the address pattern

### replace old string with new string across multiple files in-place
**Args:** `-i 's/old/new/g' file1.txt file2.txt file3.txt`
**Explanation:** -i edits all listed files in-place; g replaces every occurrence per line across all files

### substitute a path separator in file paths
**Args:** `'s|/old/path/|/new/path/|g' paths.txt`
**Explanation:** using | as delimiter avoids escaping forward slashes; replaces all occurrences of the path prefix

### delete a range of lines between two patterns
**Args:** `'/^START/,/^END/d' file.txt`
**Explanation:** deletes all lines from the first line matching START through the first line matching END (inclusive)

### replace the second occurrence of a pattern on each line
**Args:** `'s/pattern/replacement/2' file.txt`
**Explanation:** the number 2 after the closing delimiter replaces only the second occurrence per line

### strip HTML tags from a file
**Args:** `'s/<[^>]*>//g' page.html`
**Explanation:** [^>]* matches any tag content; g removes all tags on each line; output is tag-free text

### apply multiple substitutions in one invocation
**Args:** `-e 's/red/blue/g' -e 's/old/new/g' file.txt`
**Explanation:** -e applies each expression in order; both replacements are applied to each line

### number all non-empty lines in a file
**Args:** `'/./=' file.txt`
**Explanation:** /./= prints the line number for every non-empty line; combine with -n for selective output

### replace entire line matching pattern
**Args:** `'/pattern/c NEW_LINE_CONTENT' file.txt`
**Explanation:** c command replaces entire line containing pattern with NEW_LINE_CONTENT

### read next line and join with current line
**Args:** `'N;s/\n/ /' file.txt`
**Explanation:** N reads next line into pattern space; s/\n/ / replaces newline with space; joins consecutive lines

### write matching lines to separate file
**Args:** `'/pattern/w output.txt' file.txt`
**Explanation:** w command writes lines matching pattern to output.txt; useful for extracting specific records

### perform substitution only on last line
**Args:** `'$s/old/new/' file.txt`
**Explanation:** $ address targets only the last line; substitution only applied to final line of file

### insert text at beginning of file
**Args:** `'1i # Header comment' file.txt`
**Explanation:** 1i inserts text before line 1; useful for adding headers to files

### extract lines between two line numbers
**Args:** `-n '10,20p' file.txt`
**Explanation:** -n suppresses default output; 10,20p prints only lines 10 through 20 (inclusive)
