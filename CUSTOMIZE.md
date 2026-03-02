# hookplayer — Customization Guide

hookplayer is a CLI that plays sounds in response to events — designed for use with
Claude Code hooks (or any tool that can invoke a shell command). Configuration lives at
`~/.config/hookplayer/config.toml`.

## User configuration

`~/.config/hookplayer/config.toml` controls the two top-level settings and the
event-to-sound mapping:

```toml
sounds_dir = "~/.config/hookplayer/sounds"
volume = 0.5

[events]
start      = ["my_pack/hello.mp3"]
stop       = ["my_pack/goodbye.mp3"]
notify     = ["my_pack/ping.wav"]
permission = ["my_pack/denied.mp3"]
error      = ["my_pack/error.mp3"]
unknown    = ["my_pack/default.mp3"]
```

Each event maps to a list of files relative to `sounds_dir`. hookplayer picks one at
random when the event fires. The `unknown` event is the fallback for unrecognized events.

## Ports

### `PLAYER`

**What it does:** The audio playback backend.
**Default:** [`rodio`](https://github.com/RustAudio/rodio) — cross-platform, no system
dependencies, supports mp3/wav/ogg/flac.
**How to customize:** Replace `fn play(path, volume)` in `src/player.rs`. The function
signature is the only contract — swap in any backend (e.g. `afplay` on macOS via
`std::process::Command`, or `symphonia` for more format support).

### `REGISTRY_URL`

**What it does:** The remote index hookplayer fetches when you run `hookplayer list` or
`hookplayer download <pack>`.
**Default:** `https://peonping.github.io/registry/index.json`
**How to customize:** Change `const REGISTRY_URL` in `src/registry.rs` to point at your
own registry. The registry must be a JSON file with the shape:
```json
{ "packs": [{ "name": "...", "display_name": "...", "source_repo": "...",
              "source_ref": "...", "source_path": "..." }] }
```

### `CATEGORY_MAP`

**What it does:** Maps openpeon sound pack categories to hookplayer event names. This
determines which sounds from a pack get assigned to which events when you run
`hookplayer use <pack>`.
**Default:**
| openpeon category | hookplayer event |
|---|---|
| `session.start` | `start` |
| `task.complete` | `stop` |
| `task.acknowledge` | `notify` |
| `input.required`, `resource.limit` | `permission` |
| `task.error` | `error` |
| `user.spam` | `unknown` |

**How to customize:** Edit `fn category_to_event(category)` in `src/registry.rs`.

### `EVENTS`

**What it does:** The canonical list of event names and the order they're written when
`hookplayer use <pack>` updates your config.
**Default:** `["start", "stop", "notify", "permission", "error", "unknown"]`
**How to customize:** Edit the `order` array in `fn set_events()` in `src/config.rs`.
Add new event names here if you want them recognized and written by `hookplayer use`.
Custom events can also be added directly to `config.toml` without touching the source —
hookplayer will play them if invoked with the matching name.

## Getting Started

```sh
curl -fsSL https://raw.githubusercontent.com/nickagliano/hookplayer/main/install.sh | sh
hookplayer list
hookplayer download <pack-name>
hookplayer use <pack-name>
```

Then add to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse":  [{ "matcher": "", "hooks": [{ "type": "command", "command": "hookplayer permission" }] }],
    "PostToolUse": [{ "matcher": "", "hooks": [{ "type": "command", "command": "hookplayer notify" }] }],
    "Stop":        [{ "matcher": "", "hooks": [{ "type": "command", "command": "hookplayer stop" }] }]
  }
}
```
