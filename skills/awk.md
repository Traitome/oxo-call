---
name: awk
category: text-processing
description: Pattern scanning and text processing language for structured text and data extraction
tags: [awk, text, columns, csv, tsv, pattern, processing, data]
author: oxo-call built-in
source_url: "https://www.gnu.org/software/gawk/manual/gawk.html"
---

## Concepts

- awk processes input line by line, splitting each line into fields. Default field separator is whitespace; $1 is the first field, $2 the second, $NF is the last field, $0 is the entire line.
- Program structure: 'awk 'BEGIN{setup} /pattern/{action} END{teardown}' file'. BEGIN runs once before input; END runs once after all input; pattern blocks run per matching line.
- Field separator: -F ',' sets comma as separator (for CSV); -F '\t' for tab; -F ':' for colon. Inside the script, set FS='|' in BEGIN for multi-char separators.
- Built-in variables: NR (current line/record number), NF (number of fields on current line), FS (field separator), OFS (output field separator), RS (record separator).
- Arithmetic and string operations are built-in: NR>10 to skip first 10 lines, $3+0 to convert string to number, length($0) for string length, split(), sub(), gsub() for regex operations.
- awk can accumulate values: 'awk '{sum+=$1} END{print sum}'' computes the sum of column 1. Use arrays for counting/grouping: 'awk '{count[$1]++} END{for(k in count) print k, count[k]}'.

## Pitfalls

- awk field numbering starts at 1, not 0 (unlike most programming languages). $0 is the entire record.
- Shell variable expansion inside single-quoted awk programs does not work. Pass shell variables with -v: 'awk -v threshold="$THRESH" '$1>threshold'' instead of using $THRESH inside the single-quoted script.
- When using -F with tab, quote it properly: -F '\t' (in single quotes) on the command line. Using -F "\t" in double quotes may not work correctly on all shells.
- printf in awk does not add a newline by default — use \n explicitly: printf "%s\n", $1. The print statement adds a newline automatically.
- awk split() returns the number of fields, not the array. Indices of the resulting array start at 1.
- Modifying $1 or other fields causes awk to reconstruct $0 using OFS (default space). If OFS is not set, columns may be joined with spaces even if input was comma-separated.

## Examples

### print specific columns from a CSV file
**Args:** `-F ',' '{print $1","$3}' file.csv`
**Explanation:** -F ',' sets comma as delimiter; prints columns 1 and 3 with a comma separator

### sum values in a column and print the total
**Args:** `'{sum+=$2} END{print "Total:", sum}' data.txt`
**Explanation:** sum+=$2 accumulates column 2; END block prints the total after all lines are processed

### filter and print lines where a column exceeds a threshold
**Args:** `'$3 > 100 {print $0}' data.tsv`
**Explanation:** $3 > 100 is the pattern; {print $0} prints the entire matching line

### count occurrences of each unique value in a column
**Args:** `'{count[$1]++} END{for(k in count) print k, count[k]}' data.txt`
**Explanation:** count[$1]++ builds a frequency map; END block iterates the array and prints key-value pairs

### print lines between two patterns (inclusive)
**Args:** `'/START/,/END/{print}' file.txt`
**Explanation:** pattern range /START/,/END/ matches from the first line matching START to the first line matching END

### remove duplicate consecutive lines
**Args:** `'prev!=$0{print; prev=$0}' file.txt`
**Explanation:** prints a line only if it differs from the previous one; equivalent to uniq but works on any file

### add line numbers to output
**Args:** `'{print NR, $0}' file.txt`
**Explanation:** NR is the current record number; prepends line number to each line

### convert tab-separated to comma-separated
**Args:** `-F '\t' 'BEGIN{OFS=","} {$1=$1; print}' input.tsv`
**Explanation:** -F '\t' sets tab input separator; OFS="," sets output separator; $1=$1 forces field rebuild

### calculate average of a column
**Args:** `'{sum+=$1; n++} END{if(n>0) print "Average:", sum/n}' values.txt`
**Explanation:** accumulates sum and count; END computes and prints the average; guard against division by zero

### print the last field of each line regardless of column count
**Args:** `'{print $NF}' file.txt`
**Explanation:** NF is the number of fields, so $NF always refers to the last field
