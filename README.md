# CS2 Settings

**NO** / **EN**

Native desktop settings app for Counter-Strike 2 — a Rust/egui recreation of the in-game Settings menu with pro presets and real config export.

> **Disclaimer:** Not affiliated with Valve, Steam, or Counter-Strike. Original UI recreation — **no Valve game assets**. Fonts: [Barlow Condensed](https://github.com/jpt/barlow) (SIL OFL).

## Download (Windows)

Latest release: **[CS2Settings.exe](https://github.com/xdkurbie/cs2-settings/releases/latest)**

Download `CS2Settings.exe` from Releases and run it. No installer required.

## Features

- Settings tabs styled after the CS2 Panorama menu (Video, Audio, Game, Keyboard/Mouse, Controller, …)
- Pro / community presets (including donk, nuro, and other templates)
- **Export** real configs: `autoexec.cfg`, `cs2_video.txt`, `binds.cfg`, plus a short README
- Custom Commands section for your own cvars
- Cross-platform source (Windows exe via Releases; macOS/Linux via `cargo`)

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

## Using the exported configs

1. In the app footer, choose **Export config** (files go to `Documents/CS2Settings-Export/` or a folder you pick).
2. Copy `autoexec.cfg` into your CS2 `cfg` folder, e.g.  
   `…/Counter-Strike Global Offensive/game/csgo/cfg/`
3. Steam → CS2 → Properties → Launch Options: `+exec autoexec.cfg` (keep any other options you already use).
4. For video settings, place `cs2_video.txt` under Steam userdata `730/local/cfg/` — **backup the original first**.

## License

MIT — see [LICENSE](LICENSE).

Counter-Strike and Steam are trademarks of Valve Corporation. This project is an independent fan tool.
