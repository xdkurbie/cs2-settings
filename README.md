# CS2 Settings

**NO** / **EN**

Native desktop settings app for Counter-Strike 2 — a Rust/egui recreation of the in-game Settings menu with pro presets and real config export.

> **Disclaimer:** Not affiliated with Valve, Steam, or Counter-Strike. Original UI recreation — **no Valve game assets**. Fonts: [Barlow Condensed](https://github.com/jpt/barlow) (SIL OFL).

## Download

Latest release: **[Releases](https://github.com/xdkurbie/cs2-settings/releases/latest)**

| Platform | Artifact |
|----------|----------|
| Windows | `CS2Settings.exe` — download and run (no installer) |
| macOS (Intel) | `CS2Settings-macos-intel.dmg` or `.zip` — open the DMG (or unzip) and drag **CS2 Settings** to Applications |

> Unsigned macOS builds: right-click → Open the first time, or allow in System Settings → Privacy & Security.

## Features

- Settings tabs styled after the CS2 Panorama menu (Video, Audio, Game, Keyboard/Mouse, Controller, …)
- Pro / community presets (including donk, nuro, and other templates)
- **Import** real configs from your `game/csgo/cfg` and Steam userdata `730` cfg folders (read-only)
- **Export** real configs: `autoexec.cfg`, `cs2_video.txt`, `binds.cfg`, plus a short README
- Custom Commands section for your own cvars
- Cross-platform: Windows + Intel macOS via Releases; Linux via `cargo`

## Build from source

Requires [Rust](https://rustup.rs/) (stable).

```bash
git clone https://github.com/xdkurbie/cs2-settings.git
cd cs2-settings
cargo run --release
```

Windows release binary name after build: `target/release/cs2-settings.exe` (CI renames the artifact to `CS2Settings.exe`).

### macOS app bundle (optional)

```bash
cargo build --release
./scripts/bundle_app.sh
open ~/Applications/CS2\ Settings.app
```

## Import from your CS2 folders

On the **NEW** tab, **Import Settings** reads your real configs into the menu. Paste both folders. They are stored in `import_paths.json` under the app config directory (`%APPDATA%\cs2-settings` on Windows, `~/Library/Application Support/cs2-settings` on macOS, `~/.config/cs2-settings` on Linux), so you do not re-paste them every launch:

1. **game/csgo/cfg** — install cfg, where `autoexec.cfg`, `config.cfg`, and bind cfgs live.
   - Windows: `C:\Program Files (x86)\Steam\steamapps\common\Counter-Strike Global Offensive\game\csgo\cfg`
   - macOS: `~/Library/Application Support/Steam/steamapps/common/Counter-Strike Global Offensive/game/csgo/cfg`
   - Linux: `~/.steam/steam/steamapps/common/Counter-Strike Global Offensive/game/csgo/cfg`
2. **Steam userdata 730 cfg** — your account's CS2 cfg. `<SteamID>` is the numeric folder under `userdata` (not your profile name). `730/local/cfg` and `730/cfg` both work.
   - Windows: `C:\Program Files (x86)\Steam\userdata\<SteamID>\730\local\cfg`
   - macOS: `~/Library/Application Support/Steam/userdata/<SteamID>/730/local/cfg`
   - Linux: `~/.steam/steam/userdata/<SteamID>/730/local/cfg`

**Import** reads `autoexec.cfg`, `config.cfg` when present, bind cfgs, and `cs2_video.txt`. Known cvars fill the settings tabs; other autoexec lines land in Custom Commands. Import does **not** write to the game folders — use **Export config** for that.

Steam → CS2 → Manage → Browse local files opens the game install. The userdata cfg sits beside `steamapps`, at `Steam/userdata/<SteamID>/730/local/cfg`.

## Using the exported configs

1. In the app footer, choose **Export config** (files go to `Documents/CS2Settings-Export/` or a folder you pick).
2. Copy `autoexec.cfg` into your CS2 `cfg` folder, e.g.  
   `…/Counter-Strike Global Offensive/game/csgo/cfg/`
3. Steam → CS2 → Properties → Launch Options: `+exec autoexec.cfg` (keep any other options you already use).
4. For video settings, place `cs2_video.txt` under Steam userdata `730/local/cfg/` — **backup the original first**.

## License

MIT — see [LICENSE](LICENSE).

Counter-Strike and Steam are trademarks of Valve Corporation. This project is an independent fan tool.
