---
name: openclaw
category: ai-assistant
description: OpenClaw personal AI assistant CLI; manages the Gateway daemon, channels, skills, and workspace configuration
tags: [ai, assistant, gateway, clawd, skills, channels, telegram, discord, slack, workspace]
author: oxo-call built-in
source_url: "https://docs.openclaw.ai/"
---

## Concepts
- OpenClaw is a personal AI assistant platform installed globally via npm: `npm install -g openclaw@latest`.
- The default **state directory** is `~/.openclaw/`; all config, credentials, sessions, and workspace data live here.
- **Workspace directory** (skills, prompts, memories): `~/.openclaw/workspace` — keep this as a private git repo for easy backup.
- **Main config file**: `~/.openclaw/openclaw.json` (JSON5 format; override path with `OPENCLAW_CONFIG_PATH`).
- **Environment file**: `~/.openclaw/.env` — loaded last, does not override existing env vars set in the shell.
- **Credentials directory**: `~/.openclaw/credentials/` — stores channel auth tokens (WhatsApp, Telegram, Discord, Slack, etc.).
- **Sessions directory**: `~/.openclaw/agents/<agentId>/sessions/` — conversation history per agent.
- **Gateway logs**: `/tmp/openclaw/` — temporary log files written by the Gateway process.
- **OPENCLAW_STATE_DIR** env var overrides the state directory (default `~/.openclaw/`).
- **OPENCLAW_HOME** env var replaces the home directory used for all internal paths; useful for service accounts.
- The **Gateway** is the control-plane process; start with `openclaw gateway` or as a daemon via `openclaw onboard --install-daemon`.
- Gateway listens on WebSocket port **18789** by default (HTTP REST API also on the same port).
- Requires **Node.js ≥ 22** (Node 24 recommended); runs on macOS, Linux, and Windows (WSL2).
- `gateway probe` is the debug command that checks connectivity to configured and local gateways.
- `message` command provides unified outbound messaging across all connected channels.
- `agent` command runs a single agent turn via CLI for testing and automation.

## Pitfalls
- deleting `~/.openclaw/credentials/` removes all stored API keys and channel auth tokens; re-authentication is required.
- Running `openclaw gateway` without `--install-daemon` starts a foreground process; use `--install-daemon` for persistent background operation (via systemd on Linux, launchd on macOS).
- Config changes in `openclaw.json` require a gateway restart to take effect; use `openclaw gateway restart`.
- `OPENCLAW_STATE_DIR` must be set before starting the gateway; changing it after creation means the gateway won't find existing sessions or skills.
- Mixing `OPENCLAW_HOME` and `OPENCLAW_STATE_DIR` can cause path resolution conflicts; prefer setting only `OPENCLAW_STATE_DIR` unless full isolation is needed.
- On Linux servers without a display, the TUI requires a proper terminal; use `openclaw gateway` headless mode or SSH with a real terminal.
- The `~/.openclaw/.env` file is automatically loaded but does NOT override env vars already set in the shell session.
- `gateway probe` always checks both configured remote gateway and localhost; multiple reachable gateways may cause confusion.
- `--dev` flag creates development config with reduced security; do not use in production environments.

## Examples

### run the interactive onboarding wizard and install the gateway daemon
**Args:** `onboard --install-daemon`
**Explanation:** openclaw onboard subcommand; --install-daemon installs systemd (Linux) or launchd (macOS) user service; guides through gateway setup

### start the gateway in the foreground with verbose output
**Args:** `gateway --port 18789 --verbose`
**Explanation:** openclaw gateway subcommand; --port 18789 specifies port; --verbose detailed logs; starts Gateway control-plane process

### check gateway and overall service health
**Args:** `health`
**Explanation:** openclaw health subcommand; runs connectivity and auth checks across gateway, channels, and AI providers

### show gateway status (running, port, version)
**Args:** `gateway status`
**Explanation:** openclaw gateway status subcommand; prints Gateway daemon status, port, and version

### restart the gateway daemon
**Args:** `gateway restart`
**Explanation:** openclaw gateway restart subcommand; restarts the background daemon (systemd/launchd)

### stop the gateway daemon
**Args:** `gateway stop`
**Explanation:** openclaw gateway stop subcommand; gracefully stops the background daemon

### login and connect a messaging channel
**Args:** `channels login`
**Explanation:** openclaw channels login subcommand; interactive wizard to authenticate and connect a messaging channel

### list all installed skills
**Args:** `skills list`
**Explanation:** openclaw skills list subcommand; shows all available skills from workspace, registry, and MCP sources

### run the setup wizard (config + workspace bootstrap)
**Args:** `setup`
**Explanation:** openclaw setup subcommand; initializes ~/.openclaw/openclaw.json and ~/.openclaw/workspace

### send a message to a contact through the assistant
**Args:** `message send --to +1234567890 --message "Hello from openclaw"`
**Explanation:** openclaw message send subcommand; --to +1234567890 recipient number; --message "Hello from openclaw" message content

### probe gateway connectivity
**Args:** `gateway probe`
**Explanation:** openclaw gateway probe subcommand; checks connectivity to configured remote gateway and localhost

### run agent turn via CLI
**Args:** `agent turn --agent-id my_agent --input "Analyze this data"`
**Explanation:** openclaw agent turn subcommand; --agent-id my_agent agent identifier; --input "Analyze this data" input prompt

### start gateway in development mode
**Args:** `gateway --dev --port 18789`
**Explanation:** openclaw gateway subcommand; --dev development mode; --port 18789 gateway port

### list available agents
**Args:** `agent list`
**Explanation:** openclaw agent list subcommand; lists all configured agents with IDs and status
