//! In-memory CS2 settings backed by real / documented convars.
use serde::{Deserialize, Serialize};

/// Top-level settings tabs matching live CS2 (plus Custom Commands).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tab {
    New,
    Video,
    Audio,
    Game,
    KeyboardMouse,
    Controller,
    CustomCommands,
}

impl Tab {
    pub fn all() -> &'static [Tab] {
        &[
            Tab::New,
            Tab::Video,
            Tab::Audio,
            Tab::Game,
            Tab::KeyboardMouse,
            Tab::Controller,
            Tab::CustomCommands,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            Tab::New => "NEW",
            Tab::Video => "VIDEO",
            Tab::Audio => "AUDIO",
            Tab::Game => "GAME",
            Tab::KeyboardMouse => "KEYBOARD / MOUSE",
            Tab::Controller => "CONTROLLER",
            Tab::CustomCommands => "CUSTOM COMMANDS",
        }
    }
}

/// Sub-tabs under GAME (live CS2 layout).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameSubTab {
    Game,
    Hud,
    Team,
    Communication,
    Spectator,
    Item,
    Radar,
    Crosshair,
    Telemetry,
}

impl GameSubTab {
    pub fn all() -> &'static [GameSubTab] {
        &[
            GameSubTab::Game,
            GameSubTab::Hud,
            GameSubTab::Team,
            GameSubTab::Communication,
            GameSubTab::Spectator,
            GameSubTab::Item,
            GameSubTab::Radar,
            GameSubTab::Crosshair,
            GameSubTab::Telemetry,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            GameSubTab::Game => "GAME",
            GameSubTab::Hud => "HUD",
            GameSubTab::Team => "TEAM",
            GameSubTab::Communication => "COMMUNICATION",
            GameSubTab::Spectator => "SPECTATOR / SCOREBOARD",
            GameSubTab::Item => "ITEM",
            GameSubTab::Radar => "RADAR",
            GameSubTab::Crosshair => "CROSSHAIR",
            GameSubTab::Telemetry => "TELEMETRY",
        }
    }
}

/// Sub-tabs under VIDEO (live CS2 layout from nocries / in-game).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoSubTab {
    Video,
    AdvancedVideo,
    HudEdgePositions,
}

impl VideoSubTab {
    pub fn all() -> &'static [VideoSubTab] {
        &[
            VideoSubTab::Video,
            VideoSubTab::AdvancedVideo,
            VideoSubTab::HudEdgePositions,
        ]
    }

    pub fn label(self) -> &'static str {
        match self {
            VideoSubTab::Video => "VIDEO",
            VideoSubTab::AdvancedVideo => "ADVANCED VIDEO",
            VideoSubTab::HudEdgePositions => "HUD EDGE POSITIONS",
        }
    }
}

/// Rush Hour (Sep 2026) crosshair style ids 0–9.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CrosshairStyle {
    DynamicCross = 0,
    DynamicCircle = 1,
    DynamicCrossLegacy = 2,
    StaticCircle = 3,
    StaticCross = 4,
    StaticCrossShotFeedback = 5,
    DotOnly = 6,
    DynamicQuad = 7,
    StaticSquare = 8,
    StaticQuad = 9,
}

impl CrosshairStyle {
    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => Self::DynamicCross,
            1 => Self::DynamicCircle,
            2 => Self::DynamicCrossLegacy,
            3 => Self::StaticCircle,
            4 => Self::StaticCross,
            5 => Self::StaticCrossShotFeedback,
            6 => Self::DotOnly,
            7 => Self::DynamicQuad,
            8 => Self::StaticSquare,
            9 => Self::StaticQuad,
            _ => Self::DynamicQuad,
        }
    }
    pub fn as_u8(self) -> u8 {
        self as u8
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::DynamicCross => "Dynamic Cross",
            Self::DynamicCircle => "Dynamic Circle",
            Self::DynamicCrossLegacy => "Dynamic Cross (Legacy)",
            Self::StaticCircle => "Static Circle",
            Self::StaticCross => "Static Cross",
            Self::StaticCrossShotFeedback => "Static Cross (Shot Feedback)",
            Self::DotOnly => "Dot Only",
            Self::DynamicQuad => "Dynamic Quad",
            Self::StaticSquare => "Static Square",
            Self::StaticQuad => "Static Quad",
        }
    }
    pub fn all() -> &'static [CrosshairStyle] {
        &[
            CrosshairStyle::DynamicCross,
            CrosshairStyle::DynamicCircle,
            CrosshairStyle::DynamicCrossLegacy,
            CrosshairStyle::StaticCircle,
            CrosshairStyle::StaticCross,
            CrosshairStyle::StaticCrossShotFeedback,
            CrosshairStyle::DotOnly,
            CrosshairStyle::DynamicQuad,
            CrosshairStyle::StaticSquare,
            CrosshairStyle::StaticQuad,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutlineMode {
    Off = 0,
    Full = 1,
    Half = 2,
}

impl OutlineMode {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Self::Full,
            2 => Self::Half,
            _ => Self::Off,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Full => "Outline",
            Self::Half => "Half Outline",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Crosshair {
    pub style: CrosshairStyle,
    pub length: f32,
    pub gap: f32,
    pub thickness: f32,
    pub screen_height: f32,
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
    pub color_a: u8,
    pub outline: OutlineMode,
    pub outline_r: u8,
    pub outline_g: u8,
    pub outline_b: u8,
    pub outline_a: u8,
    pub outlinethickness: f32,
    pub dot: bool,
    pub t_style: bool,
    pub follow_recoil: bool,
    pub dynamic_spread_limit: f32,
    pub dynamic_splitdist: f32,
    pub dynamic_splitalpha_innermod: f32,
    pub dynamic_splitalpha_outermod: f32,
    pub dynamic_maxdist_splitratio: f32,
    pub sniper_width: f32,
    pub sniper_show_normal_inaccuracy: bool,
    pub friendly_warning: bool,
    pub ironsight_dot_scale: f32,
    pub ironsight_use_crosshair_color: bool,
    pub show_observer_crosshair: i32, // 0 no, 1 friends, 2 everyone
    pub observed_bot_crosshair: i32,  // 0 never, 1 takeover, 2 always
    pub share_code: String,
}

impl Default for Crosshair {
    fn default() -> Self {
        Self {
            style: CrosshairStyle::StaticCross,
            length: 2.0,
            gap: 1.0,
            thickness: 1.0,
            screen_height: 1080.0,
            color_r: 0,
            color_g: 255,
            color_b: 0,
            color_a: 255,
            outline: OutlineMode::Full,
            outline_r: 0,
            outline_g: 0,
            outline_b: 0,
            outline_a: 255,
            outlinethickness: 1.0,
            dot: false,
            t_style: false,
            follow_recoil: true,
            dynamic_spread_limit: 255.0,
            dynamic_splitdist: 3.0,
            dynamic_splitalpha_innermod: 0.0,
            dynamic_splitalpha_outermod: 1.0,
            dynamic_maxdist_splitratio: 1.0,
            sniper_width: 1.0,
            sniper_show_normal_inaccuracy: false,
            friendly_warning: true,
            ironsight_dot_scale: 1.0,
            ironsight_use_crosshair_color: false,
            show_observer_crosshair: 2,
            observed_bot_crosshair: 1,
            share_code: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Quality {
    Low,
    Medium,
    High,
    VeryHigh,
}

impl Quality {
    pub fn label(self) -> &'static str {
        match self {
            Quality::Low => "Low",
            Quality::Medium => "Medium",
            Quality::High => "High",
            Quality::VeryHigh => "Very High",
        }
    }
    pub fn as_u8(self) -> u8 {
        match self {
            Quality::Low => 0,
            Quality::Medium => 1,
            Quality::High => 2,
            Quality::VeryHigh => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisplayMode {
    Fullscreen,
    FullscreenWindowed,
    Windowed,
}

impl DisplayMode {
    pub fn label(self) -> &'static str {
        match self {
            DisplayMode::Fullscreen => "Fullscreen",
            DisplayMode::FullscreenWindowed => "Fullscreen Windowed",
            DisplayMode::Windowed => "Windowed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Msaa {
    None,
    X2,
    X4,
    X8,
}

impl Msaa {
    pub fn samples(self) -> u32 {
        match self {
            Msaa::None => 0,
            Msaa::X2 => 2,
            Msaa::X4 => 4,
            Msaa::X8 => 8,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Msaa::None => "None",
            Msaa::X2 => "2x MSAA",
            Msaa::X4 => "4x MSAA",
            Msaa::X8 => "8x MSAA",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LowLatency {
    Disabled,
    Enabled,
    EnabledBoost,
}

impl LowLatency {
    pub fn as_i32(self) -> i32 {
        match self {
            LowLatency::Disabled => 0,
            LowLatency::Enabled => 1,
            LowLatency::EnabledBoost => 2,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            LowLatency::Disabled => "Disabled",
            LowLatency::Enabled => "Enabled",
            LowLatency::EnabledBoost => "Enabled + Boost",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureFilter {
    Bilinear,
    Trilinear,
    Aniso2x,
    Aniso4x,
    Aniso8x,
    Aniso16x,
}

impl TextureFilter {
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Bilinear => 0,
            Self::Trilinear => 1,
            Self::Aniso2x => 2,
            Self::Aniso4x => 3,
            Self::Aniso8x => 4,
            Self::Aniso16x => 5,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Bilinear => "Bilinear",
            Self::Trilinear => "Trilinear",
            Self::Aniso2x => "Anisotropic 2X",
            Self::Aniso4x => "Anisotropic 4X",
            Self::Aniso8x => "Anisotropic 8X",
            Self::Aniso16x => "Anisotropic 16X",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum TelemetryMode {
    #[default]
    Never = 0,
    MatchOnly = 1,
    Always = 2,
}

impl TelemetryMode {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Never => "Never",
            Self::MatchOnly => "During Match Only",
            Self::Always => "Always",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EqProfile {
    Headphones = 0,
    PowerEQ = 1,
    Natural = 2,
}

impl EqProfile {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Headphones => "Headphones",
            Self::PowerEQ => "Power EQ / Bass Boost",
            Self::Natural => "Natural",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bind {
    pub key: String,
    pub command: String,
}

/// Freeform console line for CUSTOM COMMANDS tab.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomCommand {
    pub line: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    // ── Video ──────────────────────────────────────────────────────────
    pub brightness: f32, // r_fullscreen_gamma ~1.6–2.6
    pub color_mode: u8,  // 0 Computer, 1 Television-ish (best-effort)
    pub resolution_w: u32,
    pub resolution_h: u32,
    pub refresh_hz: u32,
    pub display_mode: DisplayMode,
    pub aspect_mode: u8,
    pub laptop_power_savings: bool,
    pub msaa: Msaa,
    pub shadow_quality: Quality,
    pub model_texture: Quality,
    pub texture_filter: TextureFilter,
    pub shader_detail: Quality,
    pub particle_detail: Quality,
    pub ambient_occlusion: bool,
    pub dynamic_shadows: bool,
    pub vsync: bool,
    pub multicore: bool,
    pub fxaa_cmaa: bool,
    pub motion_blur: bool,
    pub nvidia_reflex: LowLatency,
    pub fps_max: f32,
    pub fps_max_ui: f32,
    pub fsr_quality: u8,
    pub hdr_detail: i8, // -1/0/1/2 typical
    pub boost_player_contrast: bool, // r_player_visibility_mode
    pub video_preset: u8, // 0 custom .. presets
    pub nvidia_gsync: bool,
    pub magnification: f32,
    pub magnification_mode: u8, // 0 zoom

    // ── Audio ──────────────────────────────────────────────────────────
    pub volume: f32,
    pub music_volume: f32,
    pub voip_volume: f32,
    pub menumusic_volume: f32,
    pub menumap_volume: f32,
    pub roundstart_volume: f32,
    pub roundaction_volume: f32,
    pub roundend_volume: f32,
    pub mvp_volume: f32,
    pub mapobjective_volume: f32,
    pub tensecond_warning: f32,
    pub deathcamera_volume: f32,
    pub mute_mvp_music_live_players: bool,
    pub headphone: bool,
    pub enable_voice: bool,
    pub audio_device: String,
    pub eq_profile: EqProfile,
    pub lr_isolation: f32, // snd_spatialize_lerp
    pub perspective_correction: bool,
    pub mute_losefocus: bool,
    pub voice_threshold: f32,
    pub voice_loopback: bool,

    // ── Game / HUD / Radar / Telemetry ─────────────────────────────────
    pub enable_console: bool,
    pub maxping: i32,
    pub radar_always_centered: bool,
    pub radar_rotate: bool,
    pub radar_scale: f32,
    pub radar_hud_size: f32,
    pub radar_icon_scale: f32,
    pub radar_square_with_scoreboard: bool,
    pub cl_righthand: bool,
    pub viewmodel_fov: f32,
    pub viewmodel_offset_x: f32,
    pub viewmodel_offset_y: f32,
    pub viewmodel_offset_z: f32,
    pub viewmodel_presetpos: i32,
    pub net_graph: bool,
    pub rate: i32,
    pub cl_interp_ratio: i32,
    pub cl_net_buffer_ticks: i32,
    pub cl_autowepswitch: bool,
    pub cl_join_advertise: i32,
    pub mm_dedicated_search_maxping: i32,
    pub hud_scaling: f32,
    pub safezonex: f32,
    pub safezoney: f32,
    pub cl_hud_color: i32,
    pub close_buymenu_after_purchase: bool,
    pub always_show_inventory: bool,
    pub game_instructor: bool,
    pub telemetry_fps: TelemetryMode,
    pub telemetry_ping: TelemetryMode,
    pub telemetry_packet_loss: TelemetryMode,
    pub steam_notification_location: i32,
    #[serde(default)]
    pub telemetry_net_quality: TelemetryMode,
    #[serde(default)]
    pub hide_avatar_images: bool,
    #[serde(default)]
    pub teamcounter_playercount: bool,
    #[serde(default)]
    pub mute_enemy_team: bool,
    #[serde(default)]
    pub launch_options: String,

    // ── Mouse / keyboard ───────────────────────────────────────────────
    pub sensitivity: f32,
    pub zoom_sensitivity_ratio: f32,
    pub m_rawinput: bool,
    pub m_customaccel: i32,
    pub m_mouseaccel1: f32,
    pub m_mouseaccel2: f32,
    pub mouse_inverty: bool,
    pub option_duck_method: bool, // hold vs toggle
    pub option_speed_method: bool,
    pub cl_debounce_zoom: bool,
    pub binds: Vec<Bind>,

    // ── Crosshair ──────────────────────────────────────────────────────
    pub crosshair: Crosshair,

    // ── Controller ─────────────────────────────────────────────────────
    pub joy_enabled: bool,
    pub joy_sensitivity: f32,
    pub joy_invertedy: bool,
    pub joy_pitchsensitivity: f32,
    pub joy_yawsensitivity: f32,
    pub joy_movement_stick: i32, // 0 left, 1 right
    pub joy_aim_assist: bool,

    // ── Custom commands ────────────────────────────────────────────────
    pub custom_commands: Vec<CustomCommand>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            brightness: 1.6,
            color_mode: 0,
            resolution_w: 1920,
            resolution_h: 1080,
            refresh_hz: 144,
            display_mode: DisplayMode::Fullscreen,
            aspect_mode: 1,
            laptop_power_savings: false,
            msaa: Msaa::None,
            shadow_quality: Quality::High,
            model_texture: Quality::Low,
            texture_filter: TextureFilter::Aniso4x,
            shader_detail: Quality::Low,
            particle_detail: Quality::Low,
            ambient_occlusion: false,
            dynamic_shadows: true,
            vsync: false,
            multicore: true,
            fxaa_cmaa: false,
            motion_blur: false,
            nvidia_reflex: LowLatency::Enabled,
            fps_max: 400.0,
            fps_max_ui: 200.0,
            fsr_quality: 0,
            hdr_detail: -1,
            boost_player_contrast: true,
            video_preset: 0,
            nvidia_gsync: false,
            magnification: 2.0,
            magnification_mode: 0,

            volume: 1.0,
            music_volume: 0.0,
            voip_volume: 1.0,
            menumusic_volume: 0.0,
            menumap_volume: 0.05,
            roundstart_volume: 0.0,
            roundaction_volume: 0.0,
            roundend_volume: 0.0,
            mvp_volume: 0.0,
            mapobjective_volume: 0.0,
            tensecond_warning: 0.15,
            deathcamera_volume: 0.0,
            mute_mvp_music_live_players: true,
            headphone: true,
            enable_voice: true,
            audio_device: "Default".into(),
            eq_profile: EqProfile::Headphones,
            lr_isolation: 0.0,
            perspective_correction: true,
            mute_losefocus: false,
            voice_threshold: 0.0,
            voice_loopback: false,

            enable_console: true,
            maxping: 80,
            radar_always_centered: true,
            radar_rotate: true,
            radar_scale: 0.7,
            radar_hud_size: 1.0,
            radar_icon_scale: 0.7,
            radar_square_with_scoreboard: true,
            cl_righthand: true,
            viewmodel_fov: 68.0,
            viewmodel_offset_x: 2.5,
            viewmodel_offset_y: 0.0,
            viewmodel_offset_z: -1.5,
            viewmodel_presetpos: 1,
            net_graph: false,
            rate: 786432,
            cl_interp_ratio: 1,
            cl_net_buffer_ticks: 0,
            cl_autowepswitch: false,
            cl_join_advertise: 2,
            mm_dedicated_search_maxping: 80,
            hud_scaling: 0.9,
            safezonex: 1.0,
            safezoney: 1.0,
            cl_hud_color: 0,
            close_buymenu_after_purchase: true,
            always_show_inventory: false,
            game_instructor: false,
            telemetry_fps: TelemetryMode::MatchOnly,
            telemetry_ping: TelemetryMode::MatchOnly,
            telemetry_packet_loss: TelemetryMode::MatchOnly,
            steam_notification_location: 0,
            telemetry_net_quality: TelemetryMode::Never,
            hide_avatar_images: false,
            teamcounter_playercount: false,
            mute_enemy_team: false,
            launch_options: String::new(),

            sensitivity: 1.25,
            zoom_sensitivity_ratio: 1.0,
            m_rawinput: true,
            m_customaccel: 0,
            m_mouseaccel1: 0.0,
            m_mouseaccel2: 0.0,
            mouse_inverty: false,
            option_duck_method: false,
            option_speed_method: false,
            cl_debounce_zoom: true,
            binds: default_binds(),

            crosshair: Crosshair::default(),

            joy_enabled: false,
            joy_sensitivity: 1.0,
            joy_invertedy: false,
            joy_pitchsensitivity: 1.0,
            joy_yawsensitivity: 1.0,
            joy_movement_stick: 0,
            joy_aim_assist: false,

            custom_commands: vec![
                CustomCommand {
                    line: "// Example: cl_showfps 1".into(),
                    enabled: false,
                },
                CustomCommand {
                    line: "cq_print 0".into(),
                    enabled: false,
                },
            ],
        }
    }
}

fn default_binds() -> Vec<Bind> {
    vec![
        Bind { key: "w".into(), command: "+forward".into() },
        Bind { key: "a".into(), command: "+left".into() },
        Bind { key: "s".into(), command: "+back".into() },
        Bind { key: "d".into(), command: "+right".into() },
        Bind { key: "SHIFT".into(), command: "+sprint".into() },
        Bind { key: "CTRL".into(), command: "+duck".into() },
        Bind { key: "SPACE".into(), command: "+jump".into() },
        Bind { key: "mouse1".into(), command: "+attack".into() },
        Bind { key: "mouse2".into(), command: "+attack2".into() },
        Bind { key: "r".into(), command: "+reload".into() },
        Bind { key: "e".into(), command: "+use".into() },
        Bind { key: "g".into(), command: "drop".into() },
        Bind { key: "f".into(), command: "+lookatweapon".into() },
        Bind { key: "i".into(), command: "switchhands".into() },
        Bind { key: "q".into(), command: "lastinv".into() },
        Bind { key: "1".into(), command: "slot1".into() },
        Bind { key: "2".into(), command: "slot2".into() },
        Bind { key: "3".into(), command: "slot3".into() },
        Bind { key: "4".into(), command: "slot4".into() },
        Bind { key: "5".into(), command: "slot5".into() },
        Bind { key: "6".into(), command: "slot6".into() },
        Bind { key: "7".into(), command: "slot7".into() },
        Bind { key: "8".into(), command: "slot8".into() },
        Bind { key: "9".into(), command: "slot9".into() },
        Bind { key: "0".into(), command: "slot10".into() },
        Bind { key: "mwheelup".into(), command: "invprev".into() },
        Bind { key: "mwheeldown".into(), command: "invnext".into() },
        Bind { key: "b".into(), command: "buymenu".into() },
        Bind { key: "f1".into(), command: "autobuy".into() },
        Bind { key: "f2".into(), command: "rebuy".into() },
        Bind { key: "f3".into(), command: "askconnect_accept".into() },
        Bind { key: "tab".into(), command: "+showscores".into() },
        Bind { key: "enter".into(), command: "messagemode".into() },
        Bind { key: "u".into(), command: "messagemode2".into() },
        Bind { key: "k".into(), command: "+voicerecord".into() },
        Bind { key: "mouse3".into(), command: "player_ping".into() },
        Bind { key: "t".into(), command: "+spray_menu".into() },
        Bind { key: "`".into(), command: "toggleconsole".into() },
        Bind { key: "c".into(), command: "+radialradio".into() },
        Bind { key: "z".into(), command: "radio".into() },
        Bind { key: "x".into(), command: "slot12".into() },
        Bind { key: "v".into(), command: "+useserversoftdenyselected".into() },
        Bind { key: "n".into(), command: "teammenu".into() },
        Bind { key: "m".into(), command: "teammenu".into() },
        Bind { key: "p".into(), command: "cl_scoreboard_mouse_enable_binding".into() },
        Bind { key: ",".into(), command: "buyammo1".into() },
        Bind { key: ".".into(), command: "buyammo2".into() },
        Bind { key: "ESCAPE".into(), command: "cancelselect".into() },
        Bind { key: "PAUSE".into(), command: "pause".into() },
    ]
}
