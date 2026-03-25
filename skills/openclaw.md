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

## Pitfalls
- deleting `~/.openclaw/credentials/` removes all stored API keys and channel auth tokens; re-authentication is required.
- Running `openclaw gateway` without `--install-daemon` starts a foreground process; use `--install-daemon` for persistent background operation (via systemd on Linux, launchd on macOS).
- Config changes in `openclaw.json` require a gateway restart to take effect; use `openclaw gateway restart`.
- `OPENCLAW_STATE_DIR` must be set before starting the gateway; changing it after creation means the gateway won't find existing sessions or skills.
- Mixing `OPENCLAW_HOME` and `OPENCLAW_STATE_DIR` can cause path resolution conflicts; prefer setting only `OPENCLAW_STATE_DIR` unless full isolation is needed.
- On Linux servers without a display, the TUI requires a proper terminal; use `openclaw gateway` headless mode or SSH with a real terminal.
- The `~/.openclaw/.env` file is automatically loaded but does NOT override env vars already set in the shell session.

## Examples

### run the interactive onboarding wizard and install the gateway daemon
**Args:** `onboard --install-daemon`
**Explanation:** guides through gateway setup, auth, channels, and skills; --install-daemon installs a systemd (Linux) or launchd (macOS) user service so the gateway stays running

### start the gateway in the foreground with verbose output
**Args:** `gateway --port 18789 --verbose`
**Explanation:** starts the Gateway control-plane process on port 18789; --verbose shows detailed logs; use for debugging

### check gateway and overall service health
**Args:** `health`
**Explanation:** runs connectivity and auth checks across the gateway, channels, and AI providers; quick sanity check after setup or update

### show gateway status (running, port, version)
**Args:** `gateway status`
**Explanation:** prints whether the Gateway daemon is running, which port it is listening on, and the current version

### restart the gateway daemon
**Args:** `gateway restart`
**Explanation:** restarts the background daemon (systemd/launchd); required after editing ~/.openclaw/openclaw.json for config changes to take effect

### stop the gateway daemon
**Args:** `gateway stop`
**Explanation:** gracefully stops the background daemon; use before manual editing of credentials or running an upgrade

### login and connect a messaging channel
**Args:** `channels login`
**Explanation:** interactive wizard to authenticate and connect a channel (WhatsApp, Telegram, Discord, Slack, etc.); tokens are stored in ~/.openclaw/credentials/

### list all installed skills
**Args:** `skills list`
**Explanation:** shows all available skills from the user workspace (~/.openclaw/workspace/), the skill registry, and MCP sources

### run the setup wizard (config + workspace bootstrap)
**Args:** `setup`
**Explanation:** initialises ~/.openclaw/openclaw.json and ~/.openclaw/workspace if they do not exist; safe to re-run

### send a message to a contact through the assistant
**Args:** `message send --to +1234567890 --message "Hello from openclaw"`
**Explanation:** delivers a message via the connected channel (WhatsApp/Telegram/etc.) to the specified number; requires a paired channel
