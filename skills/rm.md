---
name: rm
category: filesystem
description: Remove files and directories from the filesystem (irreversible without backup)
tags: [delete, remove, filesystem, cleanup, files, directories]
author: oxo-call built-in
source_url: "https://www.gnu.org/software/coreutils/manual/html_node/rm-invocation.html"
---

## Concepts

- rm removes files immediately and permanently — there is NO undo and NO trash bin by default on most Linux systems. Always double-check the path before running rm.
- Key flags: -r / -R for recursive directory removal; -f to force without prompting even if files are write-protected; -i for interactive confirmation before each deletion; -v for verbose output showing what was deleted.
- Combining -r and -f (-rf or -fr) removes directories and all their contents non-interactively. This is one of the most dangerous command combinations on Linux.
- Use 'ls <path>' or 'find <path>' to verify exactly what will be deleted before running rm. For large deletions, prefer -i (interactive) or test with 'find' first.
- The --dry-run equivalent for rm is to use 'echo rm <args>' or 'ls <path>' to preview. Alternatively, use 'trash-put' (trash-cli) for a reversible deletion.
- Wildcards are expanded by the shell before rm sees them: 'rm *.log' deletes all .log files in the current directory. An errant space like 'rm * .log' deletes ALL files in the directory.

## Pitfalls

- Never run 'rm -rf /' or 'rm -rf /*' — this destroys the entire filesystem and renders the system unbootable. Most modern systems have --no-preserve-root as a safety guard, but do not rely on it.
- 'rm -rf .' or 'rm -rf ./' deletes the current directory and everything inside it. Confirm your working directory with 'pwd' before running recursive rm.
- A space between a path and a wildcard can cause catastrophic deletion: 'rm -rf /data/ *.bak' deletes /data/ AND all *.bak files in the current directory. Quote or brace expansions carefully.
- 'rm -rf <variable>' where the variable is empty or unset becomes 'rm -rf ' or 'rm -rf /' in some shells. Always check that path variables are non-empty before using them in rm.
- rm -f suppresses 'no such file' errors silently — combine with -v to see what was actually deleted, or omit -f when debugging.
- Prefer 'rm -i' or 'rm --interactive' for deletions in unfamiliar directories. For bulk cleanup scripts, log deleted paths with -v and redirect to a log file.

## Examples

### remove a single file
**Args:** `file.txt`
**Explanation:** removes file.txt from the current directory; fails if the file does not exist

### remove multiple files matching a pattern
**Args:** `-v *.tmp`
**Explanation:** -v prints each deleted filename; glob *.tmp matches all .tmp files in the current directory

### remove a directory and all its contents
**Args:** `-r old_results/`
**Explanation:** -r recursively removes the directory; prompts on write-protected files unless combined with -f

### interactively remove files, asking for confirmation before each deletion
**Args:** `-i *.log`
**Explanation:** -i prompts before each deletion; safer than -f when cleaning up logs

### force-remove a directory and its contents without prompts
**Args:** `-rf temp_build/`
**Explanation:** -rf is irreversible — verify the path with 'ls temp_build/' before running; no confirmation is given

### force-remove a stale build directory
**Args:** `-rf /tmp/stale_dir/`
**Explanation:** -rf removes the directory and its contents non-interactively; always verify the path with 'ls /tmp/stale_dir/' before running

### remove a file with a name starting with a dash
**Args:** `-- -weirdfile.txt`
**Explanation:** -- signals end of options so -weirdfile.txt is treated as a filename, not a flag

### remove an empty directory
**Args:** `-d emptydir/`
**Explanation:** -d removes empty directories like rmdir; fails if the directory is not empty

### verbosely remove all files of a specific type in the current directory
**Args:** `-v *.bak`
**Explanation:** -v prints each filename as it is deleted; the shell expands *.bak before rm sees it; confirm matches first with: ls *.bak

### remove a symbolic link without following it to the target
**Args:** `symlink_name`
**Explanation:** rm removes the symlink itself, not the file it points to; do NOT use -r on a symlink to a directory, as -r may follow it
