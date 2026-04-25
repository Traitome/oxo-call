---
name: awk
category: text-processing
description: Pattern scanning and text processing language for structured text and data extraction
tags: [awk, gawk, text, columns, csv, tsv, pattern, processing, data, one-liner]
author: oxo-call built-in
source_url: "https://www.gnu.org/software/gawk/manual/gawk.html"
---

## Concepts

- awk processes input line by line, splitting each line into fields. Default field separator is whitespace; $1 is the first field, $2 the second, $NF is the last field, $0 is the entire line.
- Program structure: `awk 'BEGIN{setup} /pattern/{action} END{teardown}' file`. BEGIN runs once before input; END runs once after all input; pattern blocks run per matching line.
- Field separator: -F ',' sets comma as separator (for CSV); -F '\t' for tab; -F ':' for colon. Inside the script, set FS='|' in BEGIN for multi-char separators.
- Built-in variables: NR (current line/record number), NF (number of fields on current line), FS (field separator), OFS (output field separator), RS (record separator), FILENAME (current input file name).
- Pass shell variables with -v: `awk -v var="$SHELL_VAR" '$1>var'`. Never embed shell variables directly in the awk program — single quotes prevent shell expansion.
- Arithmetic and string operations are built-in: NR>10 to skip first 10 lines, $3+0 to convert string to number, length($0) for string length, split(), sub(), gsub() for regex operations.
- awk can accumulate values: `awk '{sum+=$1} END{print sum}'` computes the sum of column 1. Use arrays for counting/grouping: `awk '{count[$1]++} END{for(k in count) print k, count[k]}'`.
- Field numbering starts at 1. To print columns 1 and 3, use `{print $1,$3}` or `{print $1","$3}` — the exact field indices must match the task.
- Read awk program from a file with -f: `awk -f program.awk data.txt`. Useful for complex multi-line programs.
- getline reads the next input line or from a pipe/file: `getline line < "file"` reads from a file; `"cmd" | getline` reads from a command.
- Two-file processing: NR counts all lines across all files; FNR resets per file. Use `FNR==NR{...;next}` idiom to process two files in one pass.
- printf provides C-style formatted output: `printf "%-20s %10d\n", $1, $2`. Unlike print, printf does NOT add a newline automatically.

## Pitfalls

- The awk program must be enclosed in single quotes (`'...'`), not double quotes. Double quotes cause shell expansion of $1, $2 etc. as shell variables.
- awk field numbering starts at 1, not 0 (unlike most programming languages). $0 is the entire record.
- Shell variable expansion inside single-quoted awk programs does not work. Pass shell variables with -v: `awk -v threshold="$THRESH" '$1>threshold'` instead of using $THRESH inside the single-quoted script.
- When using -F with tab, quote it properly: -F '\t' (in single quotes) on the command line. Using -F "\t" in double quotes may not work correctly on all shells.
- printf in awk does not add a newline by default — use \n explicitly: `printf "%s\n", $1`. The print statement adds a newline automatically.
- awk split() returns the number of fields, not the array. Indices of the resulting array start at 1.
- Modifying $1 or other fields causes awk to reconstruct $0 using OFS (default space). If OFS is not set, columns may be joined with spaces even if input was comma-separated.
- Always use the exact field indices from the task description — never substitute $1 for $3 or other field numbers.
- NR counts lines across all input files; FNR resets to 1 for each file. For two-file joins, use `FNR==NR` to distinguish which file is being processed.
- Floating point comparison: `$1 == 0.1` may fail due to precision. Use `$1 > 0.099 && $1 < 0.101` or `int($1*10) == 1` for robust comparisons.

## Examples

### print specific columns from a CSV file
**Args:** `-F ',' '{print $1","$3}' file.csv`
**Explanation:** -F ',' sets comma as delimiter; prints columns 1 and 3 with a comma separator between them; file.csv input

### sum values in a column and print the total
**Args:** `'{sum+=$2} END{print "Total:", sum}' data.txt`
**Explanation:** sum+=$2 accumulates column 2; END block prints the total after all lines are processed; data.txt input

### filter and print lines where a column exceeds a threshold
**Args:** `'$3 > 100 {print $0}' data.tsv`
**Explanation:** $3 > 100 is the pattern; {print $0} prints the entire matching line; data.tsv input

### count occurrences of each unique value in a column
**Args:** `'{count[$1]++} END{for(k in count) print k, count[k]}' data.txt`
**Explanation:** count[$1]++ builds a frequency map; END block iterates the array and prints key-value pairs; data.txt input

### print lines between two patterns (inclusive)
**Args:** `'/START/,/END/{print}' file.txt`
**Explanation:** pattern range /START/,/END/ matches from the first line matching START to the first line matching END; file.txt input

### remove duplicate consecutive lines
**Args:** `'prev!=$0{print; prev=$0}' file.txt`
**Explanation:** prints a line only if it differs from the previous one; equivalent to uniq but works on any file; file.txt input

### add line numbers to output
**Args:** `'{print NR, $0}' file.txt`
**Explanation:** NR is the current record number; prepends line number to each line; file.txt input

### convert tab-separated to comma-separated
**Args:** `-F '\t' 'BEGIN{OFS=","} {$1=$1; print}' input.tsv`
**Explanation:** -F '\t' sets tab input separator; OFS="," sets output separator; $1=$1 forces field rebuild; input.tsv input

### calculate average of a column
**Args:** `'{sum+=$1; n++} END{if(n>0) print "Average:", sum/n}' values.txt`
**Explanation:** accumulates sum and count; END computes and prints the average; guard against division by zero; values.txt input

### print the last field of each line regardless of column count
**Args:** `'{print $NF}' file.txt`
**Explanation:** NF is the number of fields, so $NF always refers to the last field; file.txt input

### print columns 2 and 4 from a tab-separated file
**Args:** `-F '\t' '{print $2"\t"$4}' data.tsv`
**Explanation:** -F '\t' tab separator; prints exactly the second and fourth fields with a tab between them; data.tsv input

### skip the header line and process data
**Args:** `'NR>1 {print $1, $2, $3}' data.csv`
**Explanation:** NR>1 skips line 1 (header); prints first three fields from all subsequent lines; data.csv input

### extract fields 1 through 3 from a colon-separated file
**Args:** `-F ':' '{print $1":"$2":"$3}' /etc/passwd`
**Explanation:** -F ':' sets colon separator; prints the username, password field, and UID joined by colons; /etc/passwd input

### replace a value in a specific column
**Args:** `-F '\t' 'BEGIN{OFS="\t"} $2=="old_value"{$2="new_value"} {print}' data.tsv`
**Explanation:** -F '\t' sets tab input separator; OFS="\t" preserves tab output; conditionally replaces column 2 when it equals 'old_value'; prints all lines; data.tsv input

### compute frequency and percentage of values in column 3
**Args:** `'{count[$3]++; total++} END{for(k in count) printf "%s\t%d\t%.2f%%\n", k, count[k], count[k]/total*100}' data.txt`
**Explanation:** counts column 3 values; END iterates with k as the key and prints each value with its count and percentage; total tracks total row count; data.txt input

### join two files on a common key column
**Args:** `-F '\t' 'FNR==NR{a[$1]=$2; next} $1 in a{print $0, a[$1]}' file2.tsv file1.tsv`
**Explanation:** -F '\t' sets tab separator; FNR==NR processes file2.tsv first, storing key-value pairs in array a; next skips to next record; for file1.tsv input, looks up key in array and appends matching value

### pass a shell variable into an awk program
**Args:** `-v min_qual="$MIN_QUAL" '$5 >= min_qual' variants.vcf`
**Explanation:** -v passes the shell variable $MIN_QUAL as awk variable min_qual; used in the pattern to filter lines where field 5 meets the threshold; variants.vcf input

### read an awk program from a file
**Args:** `-f process.awk input.txt`
**Explanation:** -f reads the awk program from process.awk instead of the command line; input.txt input; useful for complex multi-line programs that are hard to quote correctly

### pretty-print tabular output with printf
**Args:** `'{printf "%-30s %10d %8.2f\n", $1, $2, $3}' data.txt`
**Explanation:** printf formats output: %-30s left-justified 30-char string; %10d right-justified 10-digit integer; %8.2f 8-char float with 2 decimals; \n adds newline; data.txt input
