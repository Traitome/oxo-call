---
name: ac
category: System Administration / Accounting
description: A Unix/Linux utility that displays connect time statistics for users by reading system login accounting files (such as wtmp or pacct). Generates reports showing total connection time, daily breakdowns, and per-user usage summaries.
tags: [linux, accounting, connect-time, login-history, system-administration, stats, admin]
author: AI-generated
source_url: https://www.gnu.org/software/acct/manual/html_node/ac.html
---

## Concepts

- `ac` reads login records from the system accounting file (typically `/var/log/wtmp`) to calculate cumulative connection time for all users or individual users
- The default output shows total connect time in hours for all users combined; using `-p` breaks down the total by each unique username found in the accounting file
- A `-d` flag produces daily totals, showing connect time per calendar day, useful for analyzing usage patterns over time
- The `-a` flag outputs additive daily totals, which adds each day's total to the cumulative running total for each subsequent day
- Timestamps in the source file are interpreted as UTC by default; the `TZ` environment variable can influence timezone-aware output
- Output is written to standard output in a simple text format, with each line typically containing a username (for `-p`) or date (for `-d`) and the hours:minutes

## Pitfalls

- **Empty or missing accounting file**: If the accounting file (like `/var/log/wtmp`) is corrupted, empty, or missing, `ac` produces no output or zero totals, leading to incorrect conclusions about system usage. Always verify the file exists and is readable with `ls -la /var/log/wtmp`.
- **Running without root privileges**: `ac` may fail to read system accounting files if executed by a non-privileged user, resulting in permission denied errors or zero totals. Use `sudo ac` or run as root to access all accounting data.
- **Confusing `-d` with `-p`**: Using `-d` shows daily totals (useful for reporting), while `-p` shows per-user totals; mixing them up produces confusing or useless reports. `-d` and `-p` can be combined with `-a` but cannot be used alone.
- **Assuming timezone accuracy**: `ac` interprets timestamps in the source file as UTC; if the system clock or accounting file was edited or corrupted across timezone changes, output may show unexpected values, especially with `-d`.
- **Empty or stale accounting file**: On systems with no recent logins or where the accounting subsystem is disabled, the output will show zero or near-zero totals, which may be misinterpreted as a bug.

## Examples

### Display total connect time for all users combined
**Args:** `-p`
**Explanation:** The `-p` flag prints individual totals per user, but when combined correctly it shows the breakdown needed to see each user's contribution to the total connect time from the accounting file.

### Print daily connect time totals
**Args:** `-d`
**Explanation:** Using `-d` displays connect time broken down by calendar day, allowing system administrators to see usage patterns on specific dates rather than cumulative totals.

### Show both daily and per-user totals
**Args:** `-pd`
**Explanation:** Combining `-p` and `-d` produces a report showing per-user totals for each day, which is thorough but generates larger output suitable for detailed auditing.

### Print totals with year displayed
**Args:** `-dy`
**Explanation:** The `-y` flag adds the year to daily output, making historical records clearer; combining with `-d` gives year-aware daily connect time summaries.

### Print in reverse chronological order
**Args:** `-dz`
**Explanation:** The `-z` flag reverses the order of output so that the most recent day (or user) appears first, useful when quickly checking recent activity.

### Show additive daily totals with year
**Args:** `-ady`
**Explanation:** With `-a` each day's total is added to a running cumulative total shown on subsequent lines, giving a progressive view of connect time; adding `-y` displays the year for context.

### Write output to a specific file instead of stdout
**Args:** `-d > daily_usage.txt`
**Explanation:** Redirecting output to a file allows storing the results for later analysis, reporting, or archiving without displaying them on the terminal.

### Run with a custom accounting file path
**Args:** `/var/log/wtmp`
**Explanation:** Passing an explicit accounting file path overrides the default, useful when reading archived logs, backups, or non-standard file locations.

### Combine individual per-user totals with reverse order
**Args:** `-pz`
**Explanation:** Using `-p` together with `-z` shows each user's total but in reverse alphabetical order by username, which can be helpful when looking for outliers at the end of the list.

### Print total hours only without labels
**Args:** `-p`
**Explanation:** For scripting purposes, `-p` provides the simple format that can be parsed; the output is plain text suitable for piping into other analysis tools or scripts.