---
name: perl
category: scripting
description: Perl interpreter; scripting language widely used in legacy bioinformatics pipelines, text processing, BioPerl, and system automation
tags: [perl, bioperl, cpan, scripting, text-processing, one-liner, regex, bioinformatics]
author: oxo-call built-in
source_url: "https://perldoc.perl.org/"
---

## Concepts
- Perl is invoked with `perl`; scripts have a `.pl` extension and typically start with `#!/usr/bin/env perl` or `#!/usr/bin/perl`.
- Module search path `@INC` lists directories where Perl looks for modules; inspect with `perl -V` or `perl -e 'print join "\n", @INC'`.
- CPAN (Comprehensive Perl Archive Network) provides thousands of modules; install with `cpanm` (cpanminus) or `cpan`.
- User-local module installation (no root needed) via `local::lib`: modules go to `~/perl5/`; add `eval $(perl -I ~/perl5/lib/perl5 -Mlocal::lib)` to shell profile.
- `~/.cpan/` — default CPAN build and source directory; configure with `~/.cpan/CPAN/MyConfig.pm`.
- BioPerl lives in the `Bio::` namespace; install with `cpanm Bio::Perl` or via conda (`conda install -c bioconda perl-bioperl`).
- One-liners: `perl -e 'code'` executes an expression; `-n` wraps code in a `while (<>)` loop; `-p` is like `-n` but prints `$_` each iteration.
- `-i[extension]` edits files in-place (like `sed -i`); `-i.bak` makes backup copies before editing.
- `-w` enables warnings; `use strict; use warnings;` is the best-practice preamble for scripts.
- Regular expressions: Perl's regex engine is highly expressive; use `=~` for matching, `s///` for substitution, `split /pattern/, $str` for splitting.
- `$ENV{VAR}` accesses environment variables inside Perl scripts.
- Perl is included in most Unix/Linux distributions at `/usr/bin/perl`; check with `perl --version`.

## Pitfalls
- Not using `use strict; use warnings;` leads to hard-to-debug variable typos and undefined-value warnings.
- On macOS, the system perl at `/usr/bin/perl` is very old (5.18); use a Homebrew or perlbrew-managed version for modern bioinformatics.
- DANGER: `perl -i` (without an extension) edits files in-place with no backup; always test with `-n` first.
- Module paths: if a bioinformatics script fails with "Can't locate Module.pm", add its parent dir to PERL5LIB: `export PERL5LIB=/path/to/lib:$PERL5LIB`.
- `cpan` versus `cpanm`: prefer `cpanm` (App::cpanminus) for easier, faster installs; `cpan` asks many interactive questions during first run.
- Encoding issues: legacy bioinformatics Perl scripts may break with wide characters; pass `use open ':std', ':utf8';` or `binmode STDOUT, ':utf8';`.
- `while (<STDIN>)` holds the entire line including `\n`; always `chomp` before string comparisons.

## Examples

### run a Perl script
**Args:** `script.pl input.txt`
**Explanation:** executes script.pl with input.txt as argument; script must be readable; use `perl -w script.pl` for warnings

### print the Perl version and module search paths
**Args:** `-V`
**Explanation:** prints compile-time configuration, @INC paths, and Perl version; useful for debugging module-not-found errors

### one-liner: print lines matching a pattern
**Args:** `-ne 'print if /^>/' sequences.fasta`
**Explanation:** -n loops over each line; prints lines starting with '>' (FASTA headers); equivalent to grep but with full Perl regex

### one-liner: extract specific columns from a TSV
**Args:** `-lane 'print join("\t", @F[0,2,4])' data.tsv`
**Explanation:** -a splits on whitespace into @F; -l adds newline handling; prints columns 1, 3, and 5 (0-indexed)

### in-place substitution (edit file directly)
**Args:** `-i.bak -pe 's/chr/Chr/g' genome.fa`
**Explanation:** -i.bak saves backup as genome.fa.bak; -p prints each modified line; s/// replaces all occurrences

### count FASTA sequences in a file
**Args:** `-ne '$c++ if /^>/; END { print "$c sequences\n" }' input.fa`
**Explanation:** increments counter for each header line; END block executes after all input is processed

### install a module via CPAN one-liner
**Args:** `-MCPAN -e 'CPAN::Shell->install("Bio::SeqIO")'`
**Explanation:** installs Bio::SeqIO via the CPAN shell; prefer `cpanm Bio::SeqIO` (cpanminus) for a simpler, non-interactive install

### set up a local user-space Perl module directory
**Args:** `-Mlocal::lib`
**Explanation:** prints shell commands to configure ~/perl5/ as a local lib dir; eval the output: eval $(perl -Mlocal::lib)

### check if a required module is installed
**Args:** `-MBio::SeqIO -e 1`
**Explanation:** exits 0 if Bio::SeqIO loads successfully, non-zero otherwise; useful in CI/pipeline pre-checks

### run a bioinformatics script with a custom library path
**Args:** `-I /path/to/bioperl-lib script.pl input.fasta`
**Explanation:** -I adds the path to @INC; useful when BioPerl is not system-installed but located in a local directory
