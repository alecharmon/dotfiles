---
name: agent-ready-install
description: Use when installing or configuring agent-ready tmux sidebar markers for Pi, Claude Code, or another terminal agent
---

# Agent Ready Install

## Overview

Install agent-ready by wiring terminal agents to the generic dotfiles marker CLI. Keep agent-specific config thin; all agents should call `~/dev/dotfiles/scripts/agent-ready set|clear`.

## Quick Reference

| Target | Add |
|---|---|
| Pi | `~/.config/agent-ready/pi-extension.ts` in `~/.pi/agent/settings.json` `extensions` |
| Claude Code | Hook entries from `~/.config/agent-ready/claude-hooks.example.json` |
| Other agent | Call `agent-ready set` when idle, `agent-ready clear` when user submits input |

## Steps

1. Ensure dotfiles are stowed so `agent-ready/.config/agent-ready` exists under `~/.config/agent-ready`.
2. Test the generic marker inside tmux:

```bash
~/dev/dotfiles/scripts/agent-ready set --source manual
~/dev/dotfiles/scripts/agent-ready list
~/dev/dotfiles/scripts/agent-ready clear
```

3. Configure Pi by appending the extension path:

```json
{
  "extensions": ["~/.config/agent-ready/pi-extension.ts"]
}
```

Preserve existing settings and append if `extensions` already exists.

4. Configure Claude Code by merging hook entries from:

```text
~/.config/agent-ready/claude-hooks.example.json
```

into `~/.claude/settings.json` or `~/.claude/settings.local.json`.

5. Verify with the tmux sidebar open. A completed agent turn should show `🔔`; submitting another prompt should clear it.

## Common Mistakes

- Do not hardcode Pi or Claude logic into `scripts/tmux-tabs`; it only reads marker files.
- Do not overwrite existing Pi or Claude settings; merge arrays/objects.
- Do not require tmux outside checks; `agent-ready` no-ops when `$TMUX_PANE` is missing.
