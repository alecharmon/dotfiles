#!/usr/bin/env python3
"""Print one line per provider:
'<name> <session_left> <session_reset_in> <weekly_left> <weekly_reset_in>' ('?' when unknown).

Claude: live from the OAuth usage endpoint, token from Claude Code's keychain entry.
Codex: last rate_limits snapshot in ~/.codex/sessions (no network).
"""
import glob
import json
import os
import subprocess
import time
import urllib.request
from datetime import datetime

CACHE = os.path.expanduser("~/.cache/sketchybar-ai-usage.json")
UA = {"User-Agent": "sketchybar-ai-usage"}  # opencode.ai 403s the default Python-urllib UA


def fmt_in(epoch):
    s = max(0, int(epoch - time.time()))
    d, h, m = s // 86400, s % 86400 // 3600, s % 3600 // 60
    return f"{d}d{h}h" if d else f"{h}h{m:02d}m" if h else f"{m}m"


def ts(iso):
    return datetime.fromisoformat(iso.replace("Z", "+00:00")).timestamp()


def claude():
    raw = subprocess.check_output(
        ["security", "find-generic-password", "-s", "Claude Code-credentials", "-w"], text=True)
    token = json.loads(raw)["claudeAiOauth"]["accessToken"]
    req = urllib.request.Request("https://api.anthropic.com/api/oauth/usage", headers={
        **UA, "Authorization": f"Bearer {token}", "anthropic-beta": "oauth-2025-04-20"})
    data = json.load(urllib.request.urlopen(req, timeout=5))
    return {k: (w["utilization"], ts(w["resets_at"]))
            for k, src in (("session", "five_hour"), ("weekly", "seven_day"))
            if (w := data.get(src)) and w.get("resets_at")}


def codex():
    files = sorted(glob.glob(os.path.expanduser("~/.codex/sessions/*/*/*/*.jsonl")),
                   key=os.path.getmtime, reverse=True)
    for f in files[:5]:  # ponytail: latest few sessions; older snapshots are stale anyway
        with open(f) as fh:
            lines = [l for l in fh if '"rate_limits"' in l]
        for line in reversed(lines):
            rl = json.loads(line)["payload"].get("rate_limits") or {}
            wins = {}
            for w in (rl.get("primary"), rl.get("secondary")):
                if w and w.get("resets_at"):
                    # 10080 min = weekly; anything shorter is the 5h session window
                    kind = "weekly" if w.get("window_minutes", 0) >= 10080 else "session"
                    wins[kind] = (w["used_percent"], w["resets_at"])
            if wins:
                return wins
    return None


def opencode():
    with open(os.path.expanduser("~/.local/share/opencode/auth.json")) as fh:
        key = json.load(fh)["opencode-go"]["key"]
    req = urllib.request.Request("https://opencode.ai/zen/go/v1/usage",
                                 headers={**UA, "Authorization": f"Bearer {key}"})
    usage = json.load(urllib.request.urlopen(req, timeout=5))["usage"]
    # ponytail: monthly window ignored; add it here if it ever becomes the binding one
    return {k: (100 if w["status"] == "rate-limited" else w["percent"], ts(w["resetsAt"]))
            for k, src in (("session", "rolling"), ("weekly", "weekly")) if (w := usage.get(src))}


try:
    with open(CACHE) as fh:
        cache = json.load(fh)
except Exception:
    cache = {}

for name, fn in (("claude", claude), ("codex", codex), ("opencode", opencode)):
    try:
        wins = fn()
    except Exception:
        wins = None
    if wins:
        cache[name] = wins
    else:  # fetch failed (429 etc.): fall back to last good reading
        wins = cache.get(name, {})
    out = [name]
    for kind in ("session", "weekly"):
        if kind in wins:
            used, reset = wins[kind]
            # a window whose reset already passed is back to 100%
            out += ["100" if reset <= time.time() else str(round(100 - used)), fmt_in(reset)]
        else:
            out += ["?", "?"]
    print(" ".join(out))

os.makedirs(os.path.dirname(CACHE), exist_ok=True)
with open(CACHE, "w") as fh:
    json.dump(cache, fh)
