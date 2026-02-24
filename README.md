# hookplayer

A lightweight CLI that plays sounds in response to events — designed to work with [Claude Code hooks](https://docs.anthropic.com/en/docs/claude-code/hooks) (or any tool that can invoke a shell command).

## Installation

```sh
curl -fsSL https://raw.githubusercontent.com/nickagliano/hookplayer/main/install.sh | sh
```

This installs the binary to `~/.local/bin/hookplayer` and creates a default config at `~/.config/hookplayer/config.toml` if one doesn't exist.

Make sure `~/.local/bin` is in your `PATH`.

To update to the latest release:

```sh
hookplayer update
```

## Configuration

Config lives at `~/.config/hookplayer/config.toml`:

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

Each event maps to a list of sound files relative to `sounds_dir`. hookplayer picks one at random each time the event fires.

## Sound Packs

hookplayer uses a registry of community sound packs. To browse and install them:

```sh
# List available packs
hookplayer list

# Download a pack
hookplayer download <pack-name>

# List locally installed packs
hookplayer packs
```

### Managing sounds manually

You can also add sounds manually — just drop files into your sounds directory. To find or change it:

```sh
# Print current sounds directory
hookplayer dir

# Change sounds directory
hookplayer set-dir ~/my-sounds
```

`set-dir` updates your config file in-place and warns you if the directory doesn't exist yet.

```sh
# Open sounds directory in Finder (macOS)
open $(hookplayer dir)

# List installed packs
ls $(hookplayer dir)
```

### Overriding the sounds directory

You can override `sounds_dir` for a single session using an environment variable — useful for testing or scripting:

```sh
HOOKPLAYER_SOUNDS_DIR=~/alt-sounds hookplayer start
```

This does not modify your config file.

## Usage

hookplayer is invoked with an event name:

```sh
hookplayer start
hookplayer stop
hookplayer notify
hookplayer error
hookplayer permission
```

If the event has no sounds configured, hookplayer exits silently. Unknown events fall back to the `unknown` sound if configured.

### Claude Code integration

Add hooks to your `~/.claude/settings.json`:

```json
{
  "hooks": {
    "PreToolUse": [{ "matcher": "", "hooks": [{ "type": "command", "command": "hookplayer permission" }] }],
    "PostToolUse": [{ "matcher": "", "hooks": [{ "type": "command", "command": "hookplayer notify" }] }],
    "Notification": [{ "matcher": "", "hooks": [{ "type": "command", "command": "hookplayer notify" }] }],
    "Stop": [{ "matcher": "", "hooks": [{ "type": "command", "command": "hookplayer stop" }] }]
  }
}
```

## Other commands

```sh
hookplayer --version         # Print version
hookplayer update            # Self-update to latest GitHub release
hookplayer dir               # Print sounds directory path
hookplayer set-dir <path>    # Update sounds directory in config
hookplayer list              # List available packs in the registry
hookplayer download <pack>   # Download a pack from the registry
hookplayer packs             # List locally installed packs
```

You can also override the sounds directory for a single invocation without modifying your config:

```sh
HOOKPLAYER_SOUNDS_DIR=~/alt-sounds hookplayer start
```

## Sound licensing

**hookplayer does not host or distribute any sounds.** The binary is just a player.

When you use `hookplayer download`, sounds are fetched from third-party repositories listed in the community registry. Each pack's licensing is the responsibility of its publisher — check the pack's source repo before using sounds in any public or commercial context.

Many packs contain clips from games (e.g. Dota 2, League of Legends). These are copyrighted by their respective owners (Valve, Riot Games, etc.) and are generally fine for **personal use only**. Do not redistribute them.

If you're adding your own sounds, make sure you have the right to use them.
