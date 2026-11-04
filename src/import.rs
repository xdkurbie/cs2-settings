//! Read CS2 cfg folders into [`Settings`] (inverse of [`crate::export`]).
//! Import only reads. Export remains the write path.
use crate::settings::{
    Bind, CrosshairStyle, CustomCommand, DisplayMode, EqProfile, LowLatency, Msaa, OutlineMode,
    Quality, Settings, TelemetryMode, TextureFilter,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ImportPaths {
    /// `…/Counter-Strike Global Offensive/game/csgo/cfg`
    pub game_cfg: String,
    /// `…/userdata/<id>/730/local/cfg` or `…/730/cfg`
    pub userdata_cfg: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportReport {
    pub ok: bool,
    pub summary: String,
    pub files_read: Vec<String>,
    pub warnings: Vec<String>,
    pub cvars_applied: usize,
    pub binds_applied: usize,
    pub custom_commands: usize,
}

/// Trim quotes and accept both `\` and `/` so a pasted Windows path works on macOS/Linux.
pub fn normalize_user_path(raw: &str) -> PathBuf {
    let trimmed = raw
        .trim()
        .trim_matches(|c: char| c == '"' || c == '\'')
        .trim();
    let unified = if cfg!(windows) {
        trimmed.replace('/', "\\")
    } else {
        trimmed.replace('\\', "/")
    };
    PathBuf::from(unified)
}

pub fn default_import_paths_file() -> PathBuf {
    let base = dirs::config_dir()
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        })
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("cs2-settings").join("import_paths.json")
}

pub fn load_import_paths(path: &Path) -> ImportPaths {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_import_paths(path: &Path, paths: &ImportPaths) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let json = serde_json::to_string_pretty(paths)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, json)
}

/// Read cfg / video files from the two folders and merge them into `settings`.
/// Does not create or modify anything under those folders.
pub fn import_settings(settings: &mut Settings, paths: &ImportPaths) -> ImportReport {
    let mut warnings = Vec::new();
    let game = usable_dir(&paths.game_cfg, "game/csgo/cfg", &mut warnings);
    let user = usable_dir(&paths.userdata_cfg, "Steam userdata 730 cfg", &mut warnings);

    if game.is_none() && user.is_none() {
        let summary = if warnings.is_empty() {
            "Paste the game/csgo/cfg folder and the Steam userdata 730 cfg folder.".to_string()
        } else {
            format!("Import failed. {}", warnings.join(" "))
        };
        return ImportReport {
            ok: false,
            summary,
            files_read: Vec::new(),
            warnings,
            cvars_applied: 0,
            binds_applied: 0,
            custom_commands: 0,
        };
    }

    let mut roots = Vec::new();
    if let Some(dir) = &user {
        roots.push(dir.clone());
    }
    if let Some(dir) = &game {
        roots.push(dir.clone());
    }

    let mut draft = settings.clone();
    // config.cfg is a full engine dump: take known cvars and binds, skip the rest.
    // autoexec.cfg (and files it execs) is the user's script: unknown lines become custom commands.
    // game/csgo/cfg is applied after userdata so the install autoexec wins.
    // cs2_video.txt is applied game-then-userdata so the live 730 file wins.
    let outcome = {
        let mut session = Session {
            settings: &mut draft,
            binds: settings.binds.clone(),
            custom: Vec::new(),
            cvars_applied: 0,
            binds_applied: 0,
            files_read: Vec::new(),
            seen: Vec::new(),
            saw_user_script: false,
        };
        for dir in [&user, &game] {
            if let Some(dir) = dir {
                if let Some(path) = existing_file(dir, "config.cfg") {
                    session.run_file(&path, false, &roots);
                }
            }
        }
        for dir in [&user, &game] {
            if let Some(dir) = dir {
                if let Some(path) = existing_file(dir, "autoexec.cfg") {
                    session.run_file(&path, true, &roots);
                }
            }
        }
        for dir in [&user, &game] {
            if let Some(dir) = dir {
                for path in list_bind_cfgs(dir) {
                    session.run_file(&path, false, &roots);
                }
            }
        }
        for dir in [&game, &user] {
            if let Some(dir) = dir {
                if let Some(path) = existing_file(dir, "cs2_video.txt") {
                    session.read_video(&path);
                }
            }
        }
        ImportOutcome {
            binds: session.binds,
            custom: session.custom,
            cvars_applied: session.cvars_applied,
            binds_applied: session.binds_applied,
            files_read: session.files_read,
            saw_user_script: session.saw_user_script,
        }
    };

    if outcome.files_read.is_empty() {
        warnings.push("No autoexec.cfg, config.cfg, bind cfg, or cs2_video.txt found.".to_string());
        return ImportReport {
            ok: false,
            summary: format!("Import failed. {}", warnings.join(" ")),
            files_read: Vec::new(),
            warnings,
            cvars_applied: 0,
            binds_applied: 0,
            custom_commands: 0,
        };
    }

    let custom_commands = if outcome.saw_user_script {
        outcome.custom.len()
    } else {
        0
    };
    if outcome.saw_user_script {
        draft.custom_commands = outcome.custom;
    }
    draft.binds = outcome.binds;
    let summary = build_summary(
        &outcome.files_read,
        outcome.cvars_applied,
        outcome.binds_applied,
        custom_commands,
        &warnings,
    );
    *settings = draft;
    ImportReport {
        ok: true,
        summary,
        files_read: outcome.files_read,
        warnings,
        cvars_applied: outcome.cvars_applied,
        binds_applied: outcome.binds_applied,
        custom_commands,
    }
}

struct ImportOutcome {
    binds: Vec<Bind>,
    custom: Vec<CustomCommand>,
    cvars_applied: usize,
    binds_applied: usize,
    files_read: Vec<String>,
    saw_user_script: bool,
}

struct Session<'a> {
    settings: &'a mut Settings,
    binds: Vec<Bind>,
    custom: Vec<CustomCommand>,
    cvars_applied: usize,
    binds_applied: usize,
    files_read: Vec<String>,
    seen: Vec<PathBuf>,
    saw_user_script: bool,
}

impl<'a> Session<'a> {
    fn run_file(&mut self, path: &Path, capture_unknown: bool, roots: &[PathBuf]) {
        if self.files_read.len() >= 32 {
            return;
        }
        let key = canonical_key(path);
        if self.seen.iter().any(|seen| seen == &key) {
            return;
        }
        self.seen.push(key);
        let text = match read_text(path) {
            Ok(text) => text,
            Err(_) => return,
        };
        self.files_read.push(path.display().to_string());
        if capture_unknown {
            self.saw_user_script = true;
        }
        let search = search_dirs(path, roots);
        self.run_text(&text, capture_unknown, &search);
    }

    fn read_video(&mut self, path: &Path) {
        let key = canonical_key(path);
        if self.seen.iter().any(|seen| seen == &key) {
            return;
        }
        self.seen.push(key);
        let text = match read_text(path) {
            Ok(text) => text,
            Err(_) => return,
        };
        self.files_read.push(path.display().to_string());
        self.cvars_applied += apply_video_text(self.settings, &text);
    }

    fn run_text(&mut self, text: &str, capture_unknown: bool, search: &[PathBuf]) {
        for line in text.lines() {
            if let Some(disabled) = disabled_custom(line) {
                if capture_unknown {
                    self.push_custom(disabled, false);
                }
                continue;
            }
            if take_special_comment(line, self.settings) {
                continue;
            }
            let stripped = strip_inline_comment(line).trim();
            if stripped.is_empty() {
                continue;
            }
            for cmd in split_commands(stripped) {
                let cmd = cmd.trim();
                if cmd.is_empty() {
                    continue;
                }
                let tokens = tokenize(cmd);
                if tokens.is_empty() {
                    continue;
                }
                let head = tokens[0].to_ascii_lowercase();
                if head == "exec" {
                    // config.cfg is a generated dump. Its exec lines (often `exec autoexec`)
                    // are handled by the later autoexec / bind passes so those files win.
                    if !capture_unknown {
                        continue;
                    }
                    if tokens.len() >= 2 {
                        if let Some(path) = resolve_exec(&tokens[1], search) {
                            self.run_file(&path, capture_unknown, search);
                        } else if capture_unknown {
                            self.push_custom(cmd.to_string(), true);
                        }
                    }
                    continue;
                }
                if self.apply_tokens(&tokens) {
                    continue;
                }
                if is_boilerplate(&head, cmd) {
                    continue;
                }
                if capture_unknown {
                    self.push_custom(cmd.to_string(), true);
                }
            }
        }
    }

    fn apply_tokens(&mut self, tokens: &[String]) -> bool {
        let head = tokens[0].to_ascii_lowercase();
        match head.as_str() {
            "bind" => {
                if tokens.len() >= 3 {
                    let key = tokens[1].clone();
                    let command = tokens[2..].join(" ");
                    if !key.is_empty() && !command.is_empty() {
                        upsert_bind(&mut self.binds, key, command);
                        self.binds_applied += 1;
                    }
                    return true;
                }
                false
            }
            "unbind" => {
                if tokens.len() >= 2 {
                    let key = &tokens[1];
                    self.binds.retain(|b| !b.key.eq_ignore_ascii_case(key));
                    return true;
                }
                false
            }
            "unbindall" => {
                self.binds.clear();
                true
            }
            "apply_crosshair_code" => {
                if tokens.len() >= 2 {
                    self.settings.crosshair.share_code = tokens[1..].join(" ");
                    self.cvars_applied += 1;
                    true
                } else {
                    false
                }
            }
            _ => {
                if tokens.len() >= 2 && apply_cvar(self.settings, &head, &tokens[1..].join(" ")) {
                    self.cvars_applied += 1;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn push_custom(&mut self, line: String, enabled: bool) {
        let line = line.trim().to_string();
        if line.is_empty() {
            return;
        }
        if self
            .custom
            .iter()
            .any(|c| c.line == line && c.enabled == enabled)
        {
            return;
        }
        self.custom.push(CustomCommand { line, enabled });
    }
}

fn usable_dir(raw: &str, label: &str, warnings: &mut Vec<String>) -> Option<PathBuf> {
    if raw.trim().is_empty() {
        return None;
    }
    let path = normalize_user_path(raw);
    if path.is_dir() {
        Some(path)
    } else {
        warnings.push(format!("{label} not found: {}", path.display()));
        None
    }
}

fn existing_file(dir: &Path, name: &str) -> Option<PathBuf> {
    let path = dir.join(name);
    if path.is_file() {
        Some(path)
    } else {
        None
    }
}

fn list_bind_cfgs(dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(rd) = fs::read_dir(dir) else {
        return found;
    };
    for ent in rd.flatten() {
        let name = ent.file_name();
        let lower = name.to_string_lossy().to_ascii_lowercase();
        if lower.ends_with(".cfg") && lower.contains("bind") {
            let path = ent.path();
            if path.is_file() {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

fn search_dirs(file: &Path, roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(parent) = file.parent() {
        dirs.push(parent.to_path_buf());
    }
    for root in roots {
        if !dirs.iter().any(|dir| same_path(dir, root)) {
            dirs.push(root.clone());
        }
    }
    dirs
}

fn resolve_exec(token: &str, dirs: &[PathBuf]) -> Option<PathBuf> {
    let name = token.trim().trim_matches('"').trim();
    if name.is_empty() {
        return None;
    }
    let rel = Path::new(name);
    if rel.is_absolute() || rel.components().any(|c| matches!(c, Component::ParentDir)) {
        return None;
    }
    let candidates = if rel
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("cfg"))
    {
        vec![rel.to_path_buf()]
    } else {
        vec![PathBuf::from(format!("{name}.cfg")), rel.to_path_buf()]
    };
    for dir in dirs {
        for candidate in &candidates {
            let full = dir.join(candidate);
            if full.is_file() {
                return Some(full);
            }
        }
    }
    None
}

fn same_path(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

fn canonical_key(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn read_text(path: &Path) -> io::Result<String> {
    let bytes = fs::read(path)?;
    let text = String::from_utf8_lossy(&bytes);
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    Ok(text.to_string())
}

fn build_summary(
    files: &[String],
    cvars: usize,
    binds: usize,
    custom: usize,
    warnings: &[String],
) -> String {
    let names: Vec<String> = files
        .iter()
        .map(|path| {
            Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.clone())
        })
        .collect();
    let mut summary = format!(
        "Imported {} file{} ({}). Applied {} cvars, {} binds, {} custom commands.",
        names.len(),
        if names.len() == 1 { "" } else { "s" },
        names.join(", "),
        cvars,
        binds,
        custom
    );
    if !warnings.is_empty() {
        summary.push(' ');
        summary.push_str(&warnings.join(" "));
    }
    summary
}

fn disabled_custom(line: &str) -> Option<String> {
    let rest = line.trim().strip_prefix("//")?.trim();
    let rest = rest.strip_prefix("[disabled]")?.trim();
    if rest.is_empty() {
        None
    } else {
        Some(rest.to_string())
    }
}

fn take_special_comment(line: &str, settings: &mut Settings) -> bool {
    let Some(rest) = line.trim().strip_prefix("//") else {
        return false;
    };
    let rest = rest.trim();
    if let Some(opts) = rest.strip_prefix("Steam launch options:") {
        let opts = opts.trim().trim_end_matches("+exec autoexec.cfg").trim();
        settings.launch_options = opts.to_string();
        return true;
    }
    if let Some(rest) = rest.strip_prefix("joy aim assist-ish:") {
        let rest = rest.trim();
        if rest.contains("preferred on") {
            settings.joy_aim_assist = true;
        } else if rest.contains("off") {
            settings.joy_aim_assist = false;
        }
        return true;
    }
    false
}

fn is_boilerplate(head: &str, line: &str) -> bool {
    match head {
        "host_writeconfig"
        | "engine_low_latency_sleep_after_client_tick"
        | "cl_sniper_delay_unscope" => true,
        "echo" => line.contains("[CS2 Settings]"),
        _ => false,
    }
}

fn strip_inline_comment(line: &str) -> &str {
    let mut in_quotes = false;
    let mut escaped = false;
    let mut prev_slash_at: Option<usize> = None;
    for (idx, ch) in line.char_indices() {
        if escaped {
            escaped = false;
            prev_slash_at = None;
            continue;
        }
        if in_quotes && ch == '\\' {
            escaped = true;
            prev_slash_at = None;
            continue;
        }
        if ch == '"' {
            in_quotes = !in_quotes;
            prev_slash_at = None;
            continue;
        }
        if !in_quotes && ch == '/' {
            if let Some(at) = prev_slash_at {
                return line[..at].trim_end();
            }
            prev_slash_at = Some(idx);
            continue;
        }
        prev_slash_at = None;
    }
    line.trim_end()
}

fn split_commands(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut in_quotes = false;
    let mut escaped = false;
    for (idx, ch) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if in_quotes && ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            in_quotes = !in_quotes;
            continue;
        }
        if !in_quotes && ch == ';' {
            out.push(&line[start..idx]);
            start = idx + ch.len_utf8();
        }
    }
    out.push(&line[start..]);
    out
}

fn tokenize(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    let mut escaped = false;
    let mut started = false;
    for ch in line.chars() {
        if escaped {
            cur.push(ch);
            escaped = false;
            started = true;
            continue;
        }
        if in_quotes && ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            in_quotes = !in_quotes;
            started = true;
            continue;
        }
        if !in_quotes && ch.is_whitespace() {
            if started {
                out.push(std::mem::take(&mut cur));
                started = false;
            }
            continue;
        }
        cur.push(ch);
        started = true;
    }
    if started {
        out.push(cur);
    }
    out
}

fn upsert_bind(binds: &mut Vec<Bind>, key: String, command: String) {
    if let Some(existing) = binds.iter_mut().find(|b| b.key.eq_ignore_ascii_case(&key)) {
        existing.key = key;
        existing.command = command;
    } else {
        binds.push(Bind { key, command });
    }
}

fn apply_video_text(settings: &mut Settings, text: &str) -> usize {
    let mut applied = 0;
    let mut fullscreen = None;
    let mut nowindow = None;
    let mut coop = None;
    let mut refresh_num = None;
    let mut refresh_den = None;
    for line in text.lines() {
        let line = strip_inline_comment(line).trim();
        if line.is_empty() {
            continue;
        }
        let tokens = tokenize(line);
        if tokens.len() < 2 {
            continue;
        }
        let key = tokens[0].to_ascii_lowercase();
        let value = tokens[1].as_str();
        match key.as_str() {
            "setting.fullscreen" => {
                if let Some(v) = parse_i32(value) {
                    fullscreen = Some(v);
                    applied += 1;
                }
            }
            "setting.nowindowborder" => {
                if let Some(v) = parse_i32(value) {
                    nowindow = Some(v);
                    applied += 1;
                }
            }
            "setting.coop_fullscreen" => {
                if let Some(v) = parse_i32(value) {
                    coop = Some(v);
                    applied += 1;
                }
            }
            "setting.refreshrate_numerator" => {
                if let Some(v) = parse_f64(value) {
                    refresh_num = Some(v);
                    applied += 1;
                }
            }
            "setting.refreshrate_denominator" => {
                if let Some(v) = parse_f64(value) {
                    refresh_den = Some(v);
                    applied += 1;
                }
            }
            _ => {
                if apply_cvar(settings, &key, value) {
                    applied += 1;
                }
            }
        }
    }
    if let Some(mode) = resolve_display(fullscreen, nowindow, coop) {
        settings.display_mode = mode;
    }
    if let Some(num) = refresh_num {
        let den = refresh_den.unwrap_or(1000.0);
        let hz = if den.abs() < f64::EPSILON {
            num
        } else {
            num / den
        };
        let hz = hz.round();
        if (1.0..=1000.0).contains(&hz) {
            settings.refresh_hz = hz as u32;
        }
    }
    applied
}

fn resolve_display(
    fs: Option<i32>,
    nowindow: Option<i32>,
    coop: Option<i32>,
) -> Option<DisplayMode> {
    let borderless = nowindow.unwrap_or(0) == 1 || coop.unwrap_or(0) == 1;
    match fs {
        None if nowindow.is_none() && coop.is_none() => None,
        Some(1) => Some(DisplayMode::Fullscreen),
        Some(_) => {
            if borderless {
                Some(DisplayMode::FullscreenWindowed)
            } else {
                Some(DisplayMode::Windowed)
            }
        }
        None => {
            if borderless {
                Some(DisplayMode::FullscreenWindowed)
            } else {
                None
            }
        }
    }
}

fn apply_cvar(settings: &mut Settings, name: &str, value: &str) -> bool {
    let name = name.to_ascii_lowercase();
    match name.as_str() {
        "fps_max" => set_f32(&mut settings.fps_max, value),
        "fps_max_ui" => set_f32(&mut settings.fps_max_ui, value),
        "r_player_visibility_mode" => set_bool(&mut settings.boost_player_contrast, value),
        "r_fullscreen_gamma" => set_f32(&mut settings.brightness, value),
        "r_dof_override" => set_bool(&mut settings.motion_blur, value),
        "sensitivity" => set_f32(&mut settings.sensitivity, value),
        "zoom_sensitivity_ratio" => set_f32(&mut settings.zoom_sensitivity_ratio, value),
        "m_rawinput" => set_bool(&mut settings.m_rawinput, value),
        "m_customaccel" => set_i32(&mut settings.m_customaccel, value),
        "m_mouseaccel1" => set_f32(&mut settings.m_mouseaccel1, value),
        "m_mouseaccel2" => set_f32(&mut settings.m_mouseaccel2, value),
        "mouse_inverty" => set_bool(&mut settings.mouse_inverty, value),
        "option_duck_method" => set_bool(&mut settings.option_duck_method, value),
        "option_speed_method" => set_bool(&mut settings.option_speed_method, value),
        "cl_debounce_zoom" => set_bool(&mut settings.cl_debounce_zoom, value),
        "volume" => set_f32(&mut settings.volume, value),
        "snd_musicvolume" => set_f32(&mut settings.music_volume, value),
        "snd_menumusic_volume" => set_f32(&mut settings.menumusic_volume, value),
        "snd_menumap_volume" => set_f32(&mut settings.menumap_volume, value),
        "snd_roundstart_volume" => set_f32(&mut settings.roundstart_volume, value),
        "snd_roundaction_volume" => set_f32(&mut settings.roundaction_volume, value),
        "snd_roundend_volume" => set_f32(&mut settings.roundend_volume, value),
        "snd_mvp_volume" => set_f32(&mut settings.mvp_volume, value),
        "snd_mapobjective_volume" => set_f32(&mut settings.mapobjective_volume, value),
        "snd_tensecondwarning_volume" => set_f32(&mut settings.tensecond_warning, value),
        "snd_deathcamera_volume" => set_f32(&mut settings.deathcamera_volume, value),
        "snd_mute_mvp_music_live_players" => {
            set_bool(&mut settings.mute_mvp_music_live_players, value)
        }
        "snd_headphone_eq" => map_i32(value, |v| {
            settings.eq_profile = match v {
                1 => EqProfile::PowerEQ,
                2 => EqProfile::Natural,
                _ => EqProfile::Headphones,
            };
        }),
        "snd_spatialize_lerp" => set_f32(&mut settings.lr_isolation, value),
        "snd_steamaudio_enable_perspective_correction" => {
            set_bool(&mut settings.perspective_correction, value)
        }
        "snd_mute_losefocus" => set_bool(&mut settings.mute_losefocus, value),
        "voice_modenable" => set_bool(&mut settings.enable_voice, value),
        "voice_scale" => set_f32(&mut settings.voip_volume, value),
        "voice_threshold" => set_f32(&mut settings.voice_threshold, value),
        "voice_loopback" => set_bool(&mut settings.voice_loopback, value),
        "con_enable" => set_bool(&mut settings.enable_console, value),
        "mm_dedicated_search_maxping" => set_i32(&mut settings.mm_dedicated_search_maxping, value),
        "cl_righthand" => set_bool(&mut settings.cl_righthand, value),
        "viewmodel_fov" => set_f32(&mut settings.viewmodel_fov, value),
        "viewmodel_offset_x" => set_f32(&mut settings.viewmodel_offset_x, value),
        "viewmodel_offset_y" => set_f32(&mut settings.viewmodel_offset_y, value),
        "viewmodel_offset_z" => set_f32(&mut settings.viewmodel_offset_z, value),
        "viewmodel_presetpos" => set_i32(&mut settings.viewmodel_presetpos, value),
        "cl_radar_always_centered" => set_bool(&mut settings.radar_always_centered, value),
        "cl_radar_rotate" => set_bool(&mut settings.radar_rotate, value),
        "cl_radar_scale" => set_f32(&mut settings.radar_scale, value),
        "cl_hud_radar_scale" => set_f32(&mut settings.radar_hud_size, value),
        "cl_radar_icon_scale_min" => set_f32(&mut settings.radar_icon_scale, value),
        "cl_radar_square_with_scoreboard" => {
            set_bool(&mut settings.radar_square_with_scoreboard, value)
        }
        "hud_scaling" => set_f32(&mut settings.hud_scaling, value),
        "safezonex" => set_f32(&mut settings.safezonex, value),
        "safezoney" => set_f32(&mut settings.safezoney, value),
        "cl_hud_color" => set_i32(&mut settings.cl_hud_color, value),
        "cl_autowepswitch" => set_bool(&mut settings.cl_autowepswitch, value),
        "cl_join_advertise" => set_i32(&mut settings.cl_join_advertise, value),
        "gameinstructor_enable" => set_bool(&mut settings.game_instructor, value),
        "net_graph" => set_bool(&mut settings.net_graph, value),
        "cl_hud_telemetry_frametime_show" => {
            map_i32(value, |v| settings.telemetry_fps = telemetry_from_i32(v))
        }
        "cl_hud_telemetry_ping_show" => {
            map_i32(value, |v| settings.telemetry_ping = telemetry_from_i32(v))
        }
        "cl_hud_telemetry_net_misdelivery_show" => map_i32(value, |v| {
            settings.telemetry_packet_loss = telemetry_from_i32(v)
        }),
        "cl_hud_telemetry_net_quality_graph_show" => map_i32(value, |v| {
            settings.telemetry_net_quality = telemetry_from_i32(v)
        }),
        "cl_hide_avatar_images" => set_bool(&mut settings.hide_avatar_images, value),
        "cl_teamcounter_playercount_instead_of_avatars" => {
            set_bool(&mut settings.teamcounter_playercount, value)
        }
        "cl_mute_enemy_team" => set_bool(&mut settings.mute_enemy_team, value),
        "rate" => set_i32(&mut settings.rate, value),
        "cl_interp_ratio" => set_i32(&mut settings.cl_interp_ratio, value),
        "cl_net_buffer_ticks" => set_i32(&mut settings.cl_net_buffer_ticks, value),
        "cl_crosshairstyle" => map_i32(value, |v| {
            settings.crosshair.style = CrosshairStyle::from_u8(v.clamp(0, 255) as u8);
        }),
        "cl_crosshair_length" => set_f32(&mut settings.crosshair.length, value),
        "cl_crosshair_gap" => set_f32(&mut settings.crosshair.gap, value),
        "cl_crosshair_thickness" => set_f32(&mut settings.crosshair.thickness, value),
        "cl_crosshair_screen_height" => set_f32(&mut settings.crosshair.screen_height, value),
        "cl_crosshaircolor_r" => set_u8(&mut settings.crosshair.color_r, value),
        "cl_crosshaircolor_g" => set_u8(&mut settings.crosshair.color_g, value),
        "cl_crosshaircolor_b" => set_u8(&mut settings.crosshair.color_b, value),
        "cl_crosshaircolor_a" => set_u8(&mut settings.crosshair.color_a, value),
        "cl_crosshair_drawoutline" => map_i32(value, |v| {
            settings.crosshair.outline = OutlineMode::from_u8(v.clamp(0, 255) as u8);
        }),
        "cl_crosshairoutline_r" => set_u8(&mut settings.crosshair.outline_r, value),
        "cl_crosshairoutline_g" => set_u8(&mut settings.crosshair.outline_g, value),
        "cl_crosshairoutline_b" => set_u8(&mut settings.crosshair.outline_b, value),
        "cl_crosshairoutline_a" => set_u8(&mut settings.crosshair.outline_a, value),
        "cl_crosshairdot" => set_bool(&mut settings.crosshair.dot, value),
        "cl_crosshair_t" => set_bool(&mut settings.crosshair.t_style, value),
        "cl_crosshair_recoil" => set_bool(&mut settings.crosshair.follow_recoil, value),
        "cl_crosshair_dynamic_spread_limit" => {
            set_f32(&mut settings.crosshair.dynamic_spread_limit, value)
        }
        "cl_crosshair_dynamic_splitdist" => {
            set_f32(&mut settings.crosshair.dynamic_splitdist, value)
        }
        "cl_crosshair_dynamic_splitalpha_innermod" => {
            set_f32(&mut settings.crosshair.dynamic_splitalpha_innermod, value)
        }
        "cl_crosshair_dynamic_splitalpha_outermod" => {
            set_f32(&mut settings.crosshair.dynamic_splitalpha_outermod, value)
        }
        "cl_crosshair_dynamic_maxdist_splitratio" => {
            set_f32(&mut settings.crosshair.dynamic_maxdist_splitratio, value)
        }
        "cl_crosshair_sniper_width" => set_f32(&mut settings.crosshair.sniper_width, value),
        "cl_crosshair_sniper_show_normal_inaccuracy" => {
            set_bool(&mut settings.crosshair.sniper_show_normal_inaccuracy, value)
        }
        "cl_crosshair_friendly_warning" => {
            set_bool(&mut settings.crosshair.friendly_warning, value)
        }
        "cl_ironsight_dot_scale" => set_f32(&mut settings.crosshair.ironsight_dot_scale, value),
        "cl_ironsight_usecrosshaircolor" => {
            set_bool(&mut settings.crosshair.ironsight_use_crosshair_color, value)
        }
        "cl_show_observer_crosshair" => {
            set_i32(&mut settings.crosshair.show_observer_crosshair, value)
        }
        "cl_observed_bot_crosshair" => {
            set_i32(&mut settings.crosshair.observed_bot_crosshair, value)
        }
        "joystick" => set_bool(&mut settings.joy_enabled, value),
        "joy_sensitivity" => set_f32(&mut settings.joy_sensitivity, value),
        "joy_inverty" => set_bool(&mut settings.joy_invertedy, value),
        "joy_pitchsensitivity" => set_f32(&mut settings.joy_pitchsensitivity, value),
        "joy_yawsensitivity" => set_f32(&mut settings.joy_yawsensitivity, value),
        "joy_movement_stick" => set_i32(&mut settings.joy_movement_stick, value),
        "setting.defaultres" => set_u32(&mut settings.resolution_w, value),
        "setting.defaultresheight" => set_u32(&mut settings.resolution_h, value),
        "setting.mat_vsync" | "mat_vsync" => set_bool(&mut settings.vsync, value),
        "setting.cpu_level" => map_i32(value, |v| settings.multicore = v >= 2),
        "setting.videocfg_shadow_quality" => set_quality(&mut settings.shadow_quality, value),
        "setting.videocfg_texture_detail" => set_quality(&mut settings.model_texture, value),
        "setting.videocfg_ao_detail" => map_i32(value, |v| settings.ambient_occlusion = v > 0),
        "setting.videocfg_particle_detail" => set_quality(&mut settings.particle_detail, value),
        "setting.videocfg_fsr_detail" => set_u8(&mut settings.fsr_quality, value),
        "setting.videocfg_hdr_detail" => set_i8(&mut settings.hdr_detail, value),
        "setting.r_texturefilteringquality" => set_texture(&mut settings.texture_filter, value),
        "setting.r_low_latency" => set_reflex(&mut settings.nvidia_reflex, value),
        "setting.r_csgo_cmaa_enable" => set_bool(&mut settings.fxaa_cmaa, value),
        "setting.shaderquality" => map_i32(value, |v| {
            settings.shader_detail = if v <= 0 { Quality::Low } else { Quality::High };
        }),
        "setting.msaa_samples" => set_msaa(&mut settings.msaa, value),
        "setting.aspectratiomode" => set_u8(&mut settings.aspect_mode, value),
        "setting.videocfg_dynamic_shadows" => set_bool(&mut settings.dynamic_shadows, value),
        "setting.laptop_power_savings" => set_bool(&mut settings.laptop_power_savings, value),
        _ => false,
    }
}

fn telemetry_from_i32(v: i32) -> TelemetryMode {
    match v {
        1 => TelemetryMode::MatchOnly,
        2.. => TelemetryMode::Always,
        _ => TelemetryMode::Never,
    }
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().trim_matches('"').to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_i32(value: &str) -> Option<i32> {
    let value = value.trim().trim_matches('"');
    if let Ok(v) = value.parse::<i32>() {
        return Some(v);
    }
    let v: f64 = value.parse().ok()?;
    if v.is_finite() && v.fract() == 0.0 && (i32::MIN as f64..=i32::MAX as f64).contains(&v) {
        Some(v as i32)
    } else {
        None
    }
}

fn parse_f64(value: &str) -> Option<f64> {
    let v: f64 = value.trim().trim_matches('"').parse().ok()?;
    if v.is_finite() {
        Some(v)
    } else {
        None
    }
}

fn set_f32(slot: &mut f32, value: &str) -> bool {
    match value.trim().trim_matches('"').parse::<f32>() {
        Ok(v) if v.is_finite() => {
            *slot = v;
            true
        }
        _ => false,
    }
}

fn set_bool(slot: &mut bool, value: &str) -> bool {
    match parse_bool(value) {
        Some(v) => {
            *slot = v;
            true
        }
        None => false,
    }
}

fn set_i32(slot: &mut i32, value: &str) -> bool {
    match parse_i32(value) {
        Some(v) => {
            *slot = v;
            true
        }
        None => false,
    }
}

fn set_u32(slot: &mut u32, value: &str) -> bool {
    match parse_i32(value) {
        Some(v) if v >= 0 => {
            *slot = v as u32;
            true
        }
        _ => false,
    }
}

fn set_u8(slot: &mut u8, value: &str) -> bool {
    match parse_i32(value) {
        Some(v) if (0..=255).contains(&v) => {
            *slot = v as u8;
            true
        }
        _ => false,
    }
}

fn set_i8(slot: &mut i8, value: &str) -> bool {
    match parse_i32(value) {
        Some(v) if (-128..=127).contains(&v) => {
            *slot = v as i8;
            true
        }
        _ => false,
    }
}

fn map_i32(value: &str, apply: impl FnOnce(i32)) -> bool {
    match parse_i32(value) {
        Some(v) => {
            apply(v);
            true
        }
        None => false,
    }
}

fn set_quality(slot: &mut Quality, value: &str) -> bool {
    let Some(v) = parse_i32(value) else {
        return false;
    };
    *slot = match v {
        0 => Quality::Low,
        1 => Quality::Medium,
        2 => Quality::High,
        3.. => Quality::VeryHigh,
        _ => return false,
    };
    true
}

fn set_texture(slot: &mut TextureFilter, value: &str) -> bool {
    let Some(v) = parse_i32(value) else {
        return false;
    };
    *slot = match v {
        0 => TextureFilter::Bilinear,
        1 => TextureFilter::Trilinear,
        2 => TextureFilter::Aniso2x,
        3 => TextureFilter::Aniso4x,
        4 => TextureFilter::Aniso8x,
        5.. => TextureFilter::Aniso16x,
        _ => return false,
    };
    true
}

fn set_msaa(slot: &mut Msaa, value: &str) -> bool {
    let Some(v) = parse_i32(value) else {
        return false;
    };
    *slot = match v {
        0 | 1 => Msaa::None,
        2 | 3 => Msaa::X2,
        4..=7 => Msaa::X4,
        8.. => Msaa::X8,
        _ => return false,
    };
    true
}

fn set_reflex(slot: &mut LowLatency, value: &str) -> bool {
    let Some(v) = parse_i32(value) else {
        return false;
    };
    *slot = match v {
        0 => LowLatency::Disabled,
        1 => LowLatency::Enabled,
        2.. => LowLatency::EnabledBoost,
        _ => return false,
    };
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::export_all;
    use crate::settings::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn scratch(name: &str) -> PathBuf {
        static N: AtomicU64 = AtomicU64::new(0);
        let id = N.fetch_add(1, Ordering::Relaxed);
        let p = std::env::temp_dir().join(format!(
            "cs2settings-import-{}-{}-{}",
            name.replace(['/', '\\', ' '], "-"),
            std::process::id(),
            id
        ));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        p
    }

    fn near(a: f32, b: f32) {
        assert!((a - b).abs() < 0.001, "{a} != {b}");
    }

    #[test]
    fn normalizes_quoted_windows_separators() {
        let p = normalize_user_path(
            r#"  "D:\Steam\steamapps\common\Counter-Strike Global Offensive\game\csgo\cfg\"  "#,
        );
        let s = p.to_string_lossy().replace('\\', "/");
        assert!(!s.contains('"'), "{s}");
        assert!(s.contains("game/csgo/cfg"), "{s}");
        assert!(!s.contains('\\'), "{s}");
    }

    #[test]
    fn import_paths_persist_to_json() {
        let file = scratch("persist").join("import_paths.json");
        let paths = ImportPaths {
            game_cfg: r"C:\Games\csgo\cfg".into(),
            userdata_cfg: "/home/me/Steam/userdata/1/730/local/cfg".into(),
        };
        save_import_paths(&file, &paths).unwrap();
        assert_eq!(load_import_paths(&file), paths);
    }

    #[test]
    fn parses_comments_quotes_semicolons_and_binds() {
        let dir = scratch("parse");
        fs::write(
            dir.join("autoexec.cfg"),
            r#"
// sensitivity "9"
sensitivity "1.5" // scoped sens lives elsewhere
volume 0.25; zoom_sensitivity_ratio "0.8"
bind "mouse1" "+attack"
bind "j" "say hello; glhf"
bind "k" "say // not a comment"
echo "keep me"
host_writeconfig
"#,
        )
        .unwrap();
        let mut s = Settings::default();
        let before = s.sensitivity;
        let report = import_settings(
            &mut s,
            &ImportPaths {
                game_cfg: dir.display().to_string(),
                userdata_cfg: String::new(),
            },
        );
        assert!(report.ok, "{}", report.summary);
        assert_ne!(s.sensitivity, before);
        near(s.sensitivity, 1.5);
        near(s.volume, 0.25);
        near(s.zoom_sensitivity_ratio, 0.8);
        let j = s.binds.iter().find(|b| b.key == "j").expect("j bind");
        assert_eq!(j.command, "say hello; glhf");
        let k = s.binds.iter().find(|b| b.key == "k").expect("k bind");
        assert_eq!(k.command, "say // not a comment");
        assert!(s
            .binds
            .iter()
            .any(|b| b.key == "mouse1" && b.command == "+attack"));
        assert!(
            s.custom_commands
                .iter()
                .any(|c| c.enabled && c.line.contains("keep me")),
            "user echo should be kept: {:?}",
            s.custom_commands
        );
        assert!(
            !s.custom_commands
                .iter()
                .any(|c| c.line.contains("host_writeconfig")),
            "host_writeconfig is boilerplate: {:?}",
            s.custom_commands
        );
        assert!(report
            .files_read
            .iter()
            .any(|f| f.ends_with("autoexec.cfg")));
        assert!(report.cvars_applied >= 3);
        assert!(report.binds_applied >= 3);
    }

    #[test]
    fn unbindall_replaces_binds_and_config_unknowns_stay_out_of_custom() {
        let user = scratch("user-cfg");
        let game = scratch("game-cfg");
        fs::write(
            user.join("config.cfg"),
            r#"
unbindall
bind "w" "+forward"
bind "a" "+left"
sensitivity "2"
some_engine_cvar "1"
cl_bob_lower_amt "21"
"#,
        )
        .unwrap();
        fs::write(
            game.join("autoexec.cfg"),
            "bind \"w\" \"+left\"\nmy_practice_alias \"1\"\n// [disabled] cl_showfps 1\n",
        )
        .unwrap();
        let mut s = Settings::default();
        let report = import_settings(
            &mut s,
            &ImportPaths {
                game_cfg: game.display().to_string(),
                userdata_cfg: user.display().to_string(),
            },
        );
        assert!(report.ok, "{}", report.summary);
        assert_eq!(
            s.binds.len(),
            2,
            "defaults must not survive unbindall: {:?}",
            s.binds
        );
        assert_eq!(
            s.binds
                .iter()
                .find(|b| b.key.eq_ignore_ascii_case("w"))
                .unwrap()
                .command,
            "+left"
        );
        assert_eq!(
            s.binds
                .iter()
                .find(|b| b.key.eq_ignore_ascii_case("a"))
                .unwrap()
                .command,
            "+left"
        );
        near(s.sensitivity, 2.0);
        let lines: Vec<_> = s.custom_commands.iter().map(|c| c.line.as_str()).collect();
        assert!(
            lines.iter().any(|l| l.contains("my_practice_alias")),
            "{lines:?}"
        );
        assert!(
            !lines.iter().any(|l| l.contains("some_engine_cvar")),
            "{lines:?}"
        );
        assert!(
            !lines.iter().any(|l| l.contains("cl_bob_lower_amt")),
            "{lines:?}"
        );
        let disabled = s
            .custom_commands
            .iter()
            .find(|c| c.line.contains("cl_showfps"))
            .expect("disabled custom");
        assert!(!disabled.enabled);
    }

    #[test]
    fn reads_video_from_730_local_cfg_or_730_cfg_without_writing() {
        for folder in ["730/local/cfg", "730/cfg"] {
            let root = scratch(folder);
            let dir = root.join(folder);
            fs::create_dir_all(&dir).unwrap();
            let video = dir.join("cs2_video.txt");
            let body = "\
\"video.cfg\"
{
\t\"setting.defaultres\"\t\t\"1440\"
\t\"setting.defaultresheight\"\t\t\"1080\"
\t\"setting.refreshrate_numerator\"\t\t\"165000\"
\t\"setting.refreshrate_denominator\"\t\t\"1000\"
\t\"setting.fullscreen\"\t\t\"0\"
\t\"setting.nowindowborder\"\t\t\"0\"
\t\"setting.coop_fullscreen\"\t\t\"0\"
\t\"setting.mat_vsync\"\t\t\"1\"
\t\"VendorID\"\t\t\"4318\"
\t\"DeviceID\"\t\t\"9344\"
}
";
            fs::write(&video, body).unwrap();
            let before = fs::read(&video).unwrap();
            let names_before: Vec<_> = fs::read_dir(&dir)
                .unwrap()
                .map(|e| e.unwrap().file_name())
                .collect();
            let mut s = Settings::default();
            let report = import_settings(
                &mut s,
                &ImportPaths {
                    game_cfg: String::new(),
                    userdata_cfg: dir.display().to_string(),
                },
            );
            assert!(report.ok, "{}: {}", folder, report.summary);
            assert_eq!(s.resolution_w, 1440);
            assert_eq!(s.resolution_h, 1080);
            assert_eq!(s.refresh_hz, 165);
            assert_eq!(s.display_mode, DisplayMode::Windowed);
            assert!(s.vsync);
            assert_eq!(
                fs::read(&video).unwrap(),
                before,
                "import must not modify {folder}"
            );
            let names_after: Vec<_> = fs::read_dir(&dir)
                .unwrap()
                .map(|e| e.unwrap().file_name())
                .collect();
            assert_eq!(names_before, names_after);
            assert!(
                !s.custom_commands
                    .iter()
                    .any(|c| c.line.contains("VendorID")),
                "machine video keys are not custom commands"
            );
        }
    }

    #[test]
    fn userdata_video_overrides_game_copy_and_game_autoexec_overrides_userdata() {
        let game = scratch("prio-game");
        let user = scratch("prio-user");
        fs::write(game.join("autoexec.cfg"), "sensitivity \"3\"\n").unwrap();
        fs::write(user.join("autoexec.cfg"), "sensitivity \"1\"\n").unwrap();
        fs::write(
            game.join("cs2_video.txt"),
            "\"video.cfg\"\n{\n\t\"setting.defaultres\"\t\t\"800\"\n}\n",
        )
        .unwrap();
        fs::write(
            user.join("cs2_video.txt"),
            "\"video.cfg\"\n{\n\t\"setting.defaultres\"\t\t\"2560\"\n\t\"setting.fullscreen\"\t\t\"0\"\n\t\"setting.nowindowborder\"\t\t\"1\"\n\t\"setting.coop_fullscreen\"\t\t\"1\"\n}\n",
        )
        .unwrap();
        let mut s = Settings::default();
        let report = import_settings(
            &mut s,
            &ImportPaths {
                game_cfg: game.display().to_string(),
                userdata_cfg: user.display().to_string(),
            },
        );
        assert!(report.ok, "{}", report.summary);
        near(s.sensitivity, 3.0);
        assert_eq!(s.resolution_w, 2560);
        assert_eq!(s.display_mode, DisplayMode::FullscreenWindowed);
    }

    #[test]
    fn follows_exec_once_and_loads_dedicated_bind_cfgs() {
        let dir = scratch("exec");
        fs::write(
            dir.join("autoexec.cfg"),
            "exec binds.cfg\nexec autoexec.cfg\nexec binds\nsensitivity \"4\"\n",
        )
        .unwrap();
        fs::write(
            dir.join("binds.cfg"),
            "unbindall\nbind \"f\" \"drop\"\nexec autoexec.cfg\n",
        )
        .unwrap();
        fs::write(dir.join("jumpbind.cfg"), "bind \"mwheelup\" \"+jump\"\n").unwrap();
        let mut s = Settings::default();
        let report = import_settings(
            &mut s,
            &ImportPaths {
                game_cfg: dir.display().to_string(),
                userdata_cfg: String::new(),
            },
        );
        assert!(report.ok, "{}", report.summary);
        near(s.sensitivity, 4.0);
        assert!(s.binds.iter().any(|b| b.key == "f" && b.command == "drop"));
        assert!(s
            .binds
            .iter()
            .any(|b| b.key.eq_ignore_ascii_case("mwheelup") && b.command == "+jump"));
        assert!(report
            .files_read
            .iter()
            .any(|f| f.ends_with("jumpbind.cfg")));
        assert!(
            report
                .files_read
                .iter()
                .filter(|f| f.ends_with("autoexec.cfg"))
                .count()
                <= 1,
            "exec cycle should read autoexec once: {:?}",
            report.files_read
        );
    }

    #[test]
    fn accepts_backslashes_and_wrapping_quotes_for_real_folder() {
        let dir = scratch("winpath");
        fs::write(
            dir.join("autoexec.cfg"),
            "volume \"0.33\"\nm_rawinput false\n",
        )
        .unwrap();
        let shown = dir.display().to_string().replace('/', "\\");
        let raw = format!("\"{shown}\"");
        let mut s = Settings::default();
        let report = import_settings(
            &mut s,
            &ImportPaths {
                game_cfg: raw,
                userdata_cfg: String::new(),
            },
        );
        assert!(report.ok, "{}", report.summary);
        near(s.volume, 0.33);
        assert!(!s.m_rawinput);
    }

    #[test]
    fn missing_or_empty_paths_do_not_change_settings() {
        let mut s = Settings::default();
        s.sensitivity = 4.0;
        let missing = import_settings(
            &mut s,
            &ImportPaths {
                game_cfg: "/no/such/csgo/cfg".into(),
                userdata_cfg: "/no/such/730/local/cfg".into(),
            },
        );
        assert!(!missing.ok);
        near(s.sensitivity, 4.0);
        let blob = format!("{} {}", missing.summary, missing.warnings.join(" "));
        assert!(blob.to_lowercase().contains("not found"), "{blob}");

        let empty = import_settings(
            &mut s,
            &ImportPaths {
                game_cfg: "  ".into(),
                userdata_cfg: String::new(),
            },
        );
        assert!(!empty.ok);
        near(s.sensitivity, 4.0);
    }

    #[test]
    fn config_exec_of_autoexec_still_keeps_custom_lines_and_autoexec_wins() {
        let dir = scratch("cfg-exec");
        fs::write(
            dir.join("config.cfg"),
            "exec autoexec.cfg\nbind \"w\" \"+left\"\nsensitivity \"1\"\n",
        )
        .unwrap();
        fs::write(
            dir.join("autoexec.cfg"),
            "sensitivity \"2.25\"\nmy_alias \"1\"\nexec binds.cfg\n",
        )
        .unwrap();
        fs::write(dir.join("binds.cfg"), "bind \"w\" \"+forward\"\n").unwrap();
        let mut s = Settings::default();
        let report = import_settings(
            &mut s,
            &ImportPaths {
                game_cfg: dir.display().to_string(),
                userdata_cfg: String::new(),
            },
        );
        assert!(report.ok, "{}", report.summary);
        near(s.sensitivity, 2.25);
        assert!(
            s.custom_commands
                .iter()
                .any(|c| c.enabled && c.line.contains("my_alias")),
            "{:?}",
            s.custom_commands
        );
        assert_eq!(
            s.binds
                .iter()
                .find(|b| b.key == "w")
                .map(|b| b.command.as_str()),
            Some("+forward")
        );
    }

    #[test]
    fn video_only_keeps_existing_custom_commands() {
        let dir = scratch("video-only");
        fs::write(
            dir.join("cs2_video.txt"),
            "\"video.cfg\"\n{\n\t\"setting.defaultres\"\t\t\"1600\"\n\t\"setting.fullscreen\"\t\t\"1\"\n}\n",
        )
        .unwrap();
        let mut s = Settings::default();
        let marker = s.custom_commands.clone();
        let report = import_settings(
            &mut s,
            &ImportPaths {
                game_cfg: String::new(),
                userdata_cfg: dir.display().to_string(),
            },
        );
        assert!(report.ok, "{}", report.summary);
        assert_eq!(s.resolution_w, 1600);
        assert_eq!(s.display_mode, DisplayMode::Fullscreen);
        assert_eq!(s.custom_commands, marker);
    }

    #[test]
    fn roundtrip_export_then_import_restores_mapped_fields() {
        let mut original = Settings::default();
        original.sensitivity = 1.77;
        original.zoom_sensitivity_ratio = 0.82;
        original.fps_max = 240.0;
        original.fps_max_ui = 120.0;
        original.volume = 0.4;
        original.music_volume = 0.15;
        original.brightness = 2.2;
        original.m_rawinput = false;
        original.mouse_inverty = true;
        original.m_customaccel = 2;
        original.boost_player_contrast = false;
        original.motion_blur = false;
        original.resolution_w = 1280;
        original.resolution_h = 960;
        original.refresh_hz = 240;
        original.display_mode = DisplayMode::FullscreenWindowed;
        original.vsync = true;
        original.msaa = Msaa::X4;
        original.shadow_quality = Quality::Low;
        original.model_texture = Quality::Medium;
        original.texture_filter = TextureFilter::Aniso16x;
        original.ambient_occlusion = true;
        original.dynamic_shadows = false;
        original.multicore = false;
        original.fxaa_cmaa = true;
        original.nvidia_reflex = LowLatency::EnabledBoost;
        original.fsr_quality = 3;
        original.hdr_detail = 0;
        original.shader_detail = Quality::Low;
        original.particle_detail = Quality::VeryHigh;
        original.aspect_mode = 2;
        original.laptop_power_savings = true;
        original.crosshair.length = 4.0;
        original.crosshair.gap = -2.5;
        original.crosshair.thickness = 1.5;
        original.crosshair.style = CrosshairStyle::StaticCross;
        original.crosshair.dot = true;
        original.crosshair.t_style = true;
        original.crosshair.follow_recoil = false;
        original.crosshair.color_r = 10;
        original.crosshair.color_g = 20;
        original.crosshair.color_b = 30;
        original.crosshair.color_a = 200;
        original.crosshair.outline = OutlineMode::Half;
        original.crosshair.share_code = "CS-TESTCODE".into();
        original.binds = vec![
            Bind {
                key: "w".into(),
                command: "+forward".into(),
            },
            Bind {
                key: "mouse1".into(),
                command: "+attack".into(),
            },
            Bind {
                key: "j".into(),
                command: "say hello; glhf".into(),
            },
        ];
        original.custom_commands = vec![
            CustomCommand {
                line: "cq_print 0".into(),
                enabled: true,
            },
            CustomCommand {
                line: "cl_showfps 1".into(),
                enabled: false,
            },
        ];
        original.launch_options = "-novid -console".into();
        original.joy_aim_assist = true;
        original.joy_enabled = true;
        original.joy_sensitivity = 2.5;
        original.eq_profile = EqProfile::Natural;
        original.telemetry_fps = TelemetryMode::Always;
        original.telemetry_ping = TelemetryMode::Never;
        original.rate = 128000;
        original.cl_righthand = false;
        original.viewmodel_fov = 60.0;
        original.hud_scaling = 0.85;
        original.radar_scale = 0.4;
        original.radar_always_centered = false;
        original.enable_console = false;
        original.voice_loopback = true;
        original.mute_losefocus = true;
        original.cl_hud_color = 5;
        original.net_graph = true;
        original.hide_avatar_images = true;

        let game = scratch("rt-game");
        let user = scratch("rt-user");
        let export_dir = scratch("rt-export");
        export_all(&original, &export_dir).unwrap();
        fs::rename(export_dir.join("autoexec.cfg"), game.join("autoexec.cfg")).unwrap();
        fs::rename(export_dir.join("binds.cfg"), game.join("binds.cfg")).unwrap();
        fs::rename(export_dir.join("cs2_video.txt"), user.join("cs2_video.txt")).unwrap();

        let mut loaded = Settings::default();
        let report = import_settings(
            &mut loaded,
            &ImportPaths {
                game_cfg: game.display().to_string(),
                userdata_cfg: user.display().to_string(),
            },
        );
        assert!(report.ok, "{}", report.summary);
        assert!(
            report.summary.contains("cvar") || report.cvars_applied > 0,
            "{}",
            report.summary
        );
        assert!(report.files_read.len() >= 3, "{:?}", report.files_read);

        near(loaded.sensitivity, original.sensitivity);
        near(
            loaded.zoom_sensitivity_ratio,
            original.zoom_sensitivity_ratio,
        );
        near(loaded.fps_max, original.fps_max);
        near(loaded.fps_max_ui, original.fps_max_ui);
        near(loaded.volume, original.volume);
        near(loaded.music_volume, original.music_volume);
        near(loaded.brightness, original.brightness);
        assert_eq!(loaded.m_rawinput, original.m_rawinput);
        assert_eq!(loaded.mouse_inverty, original.mouse_inverty);
        assert_eq!(loaded.m_customaccel, original.m_customaccel);
        assert_eq!(loaded.boost_player_contrast, original.boost_player_contrast);
        assert!(!loaded.motion_blur);
        assert_eq!(loaded.resolution_w, original.resolution_w);
        assert_eq!(loaded.resolution_h, original.resolution_h);
        assert_eq!(loaded.refresh_hz, original.refresh_hz);
        assert_eq!(loaded.display_mode, original.display_mode);
        assert_eq!(loaded.vsync, original.vsync);
        assert_eq!(loaded.msaa, original.msaa);
        assert_eq!(loaded.shadow_quality, original.shadow_quality);
        assert_eq!(loaded.model_texture, original.model_texture);
        assert_eq!(loaded.texture_filter, original.texture_filter);
        assert_eq!(loaded.ambient_occlusion, original.ambient_occlusion);
        assert_eq!(loaded.dynamic_shadows, original.dynamic_shadows);
        assert_eq!(loaded.multicore, original.multicore);
        assert_eq!(loaded.fxaa_cmaa, original.fxaa_cmaa);
        assert_eq!(loaded.nvidia_reflex, original.nvidia_reflex);
        assert_eq!(loaded.fsr_quality, original.fsr_quality);
        assert_eq!(loaded.hdr_detail, original.hdr_detail);
        assert_eq!(loaded.shader_detail, original.shader_detail);
        assert_eq!(loaded.particle_detail, original.particle_detail);
        assert_eq!(loaded.aspect_mode, original.aspect_mode);
        assert_eq!(loaded.laptop_power_savings, original.laptop_power_savings);
        near(loaded.crosshair.length, original.crosshair.length);
        near(loaded.crosshair.gap, original.crosshair.gap);
        near(loaded.crosshair.thickness, original.crosshair.thickness);
        assert_eq!(loaded.crosshair.style, original.crosshair.style);
        assert_eq!(loaded.crosshair.dot, original.crosshair.dot);
        assert_eq!(loaded.crosshair.t_style, original.crosshair.t_style);
        assert_eq!(
            loaded.crosshair.follow_recoil,
            original.crosshair.follow_recoil
        );
        assert_eq!(loaded.crosshair.color_r, original.crosshair.color_r);
        assert_eq!(loaded.crosshair.color_g, original.crosshair.color_g);
        assert_eq!(loaded.crosshair.color_b, original.crosshair.color_b);
        assert_eq!(loaded.crosshair.color_a, original.crosshair.color_a);
        assert_eq!(loaded.crosshair.outline, original.crosshair.outline);
        assert_eq!(loaded.crosshair.share_code, original.crosshair.share_code);
        for b in &original.binds {
            let got = loaded
                .binds
                .iter()
                .find(|x| x.key.eq_ignore_ascii_case(&b.key))
                .unwrap_or_else(|| panic!("missing bind {} in {:?}", b.key, loaded.binds));
            assert_eq!(got.command, b.command);
        }
        assert!(loaded
            .custom_commands
            .iter()
            .any(|c| c.enabled && c.line.trim() == "cq_print 0"));
        assert!(loaded
            .custom_commands
            .iter()
            .any(|c| !c.enabled && c.line.contains("cl_showfps 1")));
        assert_eq!(loaded.launch_options, original.launch_options);
        assert!(loaded.joy_aim_assist);
        assert!(loaded.joy_enabled);
        near(loaded.joy_sensitivity, original.joy_sensitivity);
        assert_eq!(loaded.eq_profile, original.eq_profile);
        assert_eq!(loaded.telemetry_fps, original.telemetry_fps);
        assert_eq!(loaded.telemetry_ping, original.telemetry_ping);
        assert_eq!(loaded.rate, original.rate);
        assert_eq!(loaded.cl_righthand, original.cl_righthand);
        near(loaded.viewmodel_fov, original.viewmodel_fov);
        near(loaded.hud_scaling, original.hud_scaling);
        near(loaded.radar_scale, original.radar_scale);
        assert_eq!(loaded.radar_always_centered, original.radar_always_centered);
        assert_eq!(loaded.enable_console, original.enable_console);
        assert_eq!(loaded.voice_loopback, original.voice_loopback);
        assert_eq!(loaded.mute_losefocus, original.mute_losefocus);
        assert_eq!(loaded.cl_hud_color, original.cl_hud_color);
        assert_eq!(loaded.net_graph, original.net_graph);
        assert_eq!(loaded.hide_avatar_images, original.hide_avatar_images);
    }
}
