---
name: byobu
category: terminal-multiplexer
description: A text-based window manager and terminal multiplexer that provides enhanced session management and status notifications for tmux or screen backends.
tags:
  - terminal
  - multiplexing
  - screen
  - tmux
  - session-management
  - window-manager
  - status-bar
author: AI-generated
source_url: https://byobu.org
---

## Concepts

- Byobu is a lightweight wrapper around tmux or GNU Screen that adds convenient keybindings, status notifications, and session persistence—allowing you to manage multiple terminal windows within a single terminal interface.
- The backend preference is determined at compile time: if tmux is installed, byobu uses it; otherwise, byobu falls back to screen, and this choice affects which configuration files are used (~/.byobu/tmux.conf or ~/.byobu/screenrc).
- Status notifications display real-time system information (CPU load, memory usage, network throughput, battery level, date/time) via hardstatus lines, which can be customized in ~/.byobu/status.
- Keybindings follow a consistent prefix pattern: F3 splits panes, F4 switches panes, and Alt+Up/Down navigates between windows—these are designed to work without conflicting with common terminal emulators.

## Pitfalls

- Starting byobu without the -r flag when reattaching to an existing session causes a new session to be created instead, leaving the old session orphaned and potentially consuming extra resources.
- Using byobu with an outdated or misconfigured tmux/screen backend leads to missing status notifications and unresponsive keybindings, especially on systems where tmux was installed after byobu.
- Pressing Ctrl+C to exit a session inside byobu detaches rather than terminates the session, which can be confusing if you expect the terminal to close completely.
- Running byobu inside another terminal multiplexer (like nested screen within tmux) causes keybinding conflicts and unpredictable pane behavior, so avoid nesting multiplexers.
- Installing byobu via package managers that defer tmux/screen as optional dependencies may result in a non-functional setup if the backend is not explicitly installed alongside it.

## Examples

### Start a new byobu session from scratch
**Args:** 
**Explanation:** Launching byobu without arguments creates a fresh session with status notifications and default keybindings.

### Reattach to an existing detached byobu session
**Args:** -r
**Explanation:** The -r flag tells byobu to attach to a previously detached session, preserving running processes and history.

### Start byobu using the screen backend explicitly
**Args:** -S
**Explanation:** The -S flag forces byobu to use GNU Screen as its backend instead of tmux, useful when tmux is unavailable.

### List all active byobu/screen sessions
**Args:** screen -ls
**Explanation:** This command lists running screen sessions, helping identify which session to attach to when reconnecting.

### Send a byobu session to the background (detach)
**Args:** Ctrl+A, D
**Explanation:** Pressing the prefix key (Ctrl+A) followed by D detaches the current session, leaving all windows and processes intact for later reconnection.

### Kill a specific orphaned screen session by ID
**Args:** screen -X -S 12345 kill
**Explanation:** This forcefully terminates a screen session with PID 12345, cleaning up resources when a session becomes unresponsive.

### Launch byobu with tmux backend in detaching mode
**Args:** byobu-tmux -d
**Explanation:** Starting the tmux backend directly with -d daemonizes the session, keeping it running in the background without attaching to the terminal.

### Display byobu help and keybinding reference
**Args:** help
**Explanation:** Running byobu help outputs a quick reference of all available keybindings and configuration options for customizing the session.

### Disable automatic status notifications for a session
**Args:** BYOBU_STATUS=off byobu
**Explanation:** Setting the environment variable BYOBU_STATUS to "off" suppresses status bar updates, useful for scripts or reduced visual clutter.

### Transfer control of a session to another user (multiplayer mode)
**Args:** detach -HO
**Explanation:** The detach command with -H creates a hologram mode session that another user can attach to, enabling collaborative terminal sharing.

---

## Data Model and I/O Format

Byobu manages sessions through a hierarchical model: each session contains one or more windows, and each window can be split into multiple panes using tmux or screen layout commands. The default configuration is stored in ~/.byobu/profile, which defines environment variables and backend selection rules. Status data is sourced from ~/.byobu/bin/\* scripts and rendered into the hardstatus string at configurable intervals. Session metadata (socket paths, PID files) is stored in /tmp/byobu-username to enable multi-user isolation on shared systems.