//! Preset packs: pro templates + Steam-guide community presets (donk, nuro).
//! Rush Hour (Sep 2026) uses cl_crosshair_length / _thickness / _gap (pixels).
use crate::settings::*;

pub struct Preset {
    pub name: &'static str,
    pub source: &'static str,
    pub summary: &'static str,
    pub apply: fn(&mut Settings),
}

pub fn all() -> &'static [Preset] {
    &[
        Preset {
            name: "donk (Oct 2026)",
            source: "Resquart — steamcommunity.com/sharedfiles/filedetails/?id=3822534275",
            summary: "Static Cross (Shot Feedback) L2 T1 G0 white + outline 27,27,27, dot · 1280×960 4:3 stretched · 600 FPS · 8x MSAA · sens 1.25",
            apply: apply_donk,
        },
        Preset {
            name: "nuro",
            source: "nuro — steamcommunity.com/sharedfiles/filedetails/?id=3832356687",
            summary: "Static green L2 T0.5 G-3 · 1440×1080 4:3 · 400 FPS · 4x MSAA · radar 0.4 · nade binds · telemetry always",
            apply: apply_nuro,
        },
        Preset {
            name: "Competitive Low Spec",
            source: "CS2 Settings — pro_base + low GPU overlay",
            summary: "Max FPS: low shadows/textures, no MSAA/AO/FSR, Reflex Boost, cyan static cross · 1920×1080",
            apply: apply_competitive_low_spec,
        },
        Preset {
            name: "Competitive High Spec",
            source: "CS2 Settings — pro_base + high GPU overlay",
            summary: "Clarity pack: High shadows, 4x MSAA, Aniso 8x, boost contrast · teal static cross",
            apply: apply_competitive_high_spec,
        },
        Preset {
            name: "Faceit/Premier Aim",
            source: "CS2 Settings — aim duel focus",
            summary: "Small green cross L1.5 T0.5 G-2 · 4:3 1280×960 · low clutter HUD · high rate",
            apply: apply_faceit_premier_aim,
        },
        Preset {
            name: "AWP/Scout Focus",
            source: "CS2 Settings — sniper zoom + thick sniper lines",
            summary: "Static cross + sniper width 2 · zoom sens 0.9 · yellow tip · high texture filter",
            apply: apply_awp_scout_focus,
        },
        Preset {
            name: "Spray Control",
            source: "CS2 Settings — dynamic follow-recoil feedback",
            summary: "Dynamic Cross + follow recoil · cyan · slightly thicker for spray tracking",
            apply: apply_spray_control,
        },
        Preset {
            name: "Visibility Max",
            source: "CS2 Settings — contrast / brightness push",
            summary: "Boost player contrast, brighter gamma, AO off, high MSAA · magenta cross for dark maps",
            apply: apply_visibility_max,
        },
        Preset {
            name: "Minimal HUD",
            source: "CS2 Settings — clean screen",
            summary: "Tiny HUD/radar, no net graph/avatars · tiny white cross · music muted",
            apply: apply_minimal_hud,
        },
        Preset {
            name: "Streaming/Content",
            source: "CS2 Settings — look good on stream",
            summary: "Windowed borderless, High textures, music on, FPS 240 · soft green cross",
            apply: apply_streaming_content,
        },
        Preset {
            name: "Laptop Power",
            source: "CS2 Settings — battery / thermals",
            summary: "Laptop power savings, FSR Balanced, FPS 144, Low everything · subtle cyan cross",
            apply: apply_laptop_power,
        },
        Preset {
            name: "Practice/Grenade",
            source: "CS2 Settings — offline practice (sv_cheats commented)",
            summary: "Nade binds + practice cmds disabled by default · large gap cross for lineups",
            apply: apply_practice_grenade,
        },
    ]
}

/// Shared competitive baseline: audio, mouse, radar, HUD, viewmodel, net, binds, launch.
fn pro_base(s: &mut Settings) {
    // Audio — competitive (music off, voice clear)
    s.volume = 1.0;
    s.music_volume = 0.0;
    s.menumusic_volume = 0.0;
    s.menumap_volume = 0.0;
    s.roundstart_volume = 0.0;
    s.roundaction_volume = 0.0;
    s.roundend_volume = 0.0;
    s.mvp_volume = 0.0;
    s.mapobjective_volume = 0.05;
    s.tensecond_warning = 0.2;
    s.deathcamera_volume = 0.0;
    s.mute_mvp_music_live_players = true;
    s.headphone = true;
    s.enable_voice = true;
    s.eq_profile = EqProfile::Headphones;
    s.lr_isolation = 0.0;
    s.perspective_correction = true;
    s.mute_losefocus = false;
    s.voip_volume = 1.0;
    s.voice_threshold = 0.0;
    s.voice_loopback = false;

    // Mouse
    s.sensitivity = 1.25;
    s.zoom_sensitivity_ratio = 1.0;
    s.m_rawinput = true;
    s.m_customaccel = 0;
    s.m_mouseaccel1 = 0.0;
    s.m_mouseaccel2 = 0.0;
    s.mouse_inverty = false;
    s.option_duck_method = false;
    s.option_speed_method = false;
    s.cl_debounce_zoom = true;

    // Radar / HUD
    s.radar_always_centered = true;
    s.radar_rotate = true;
    s.radar_scale = 0.7;
    s.radar_hud_size = 1.0;
    s.radar_icon_scale = 0.7;
    s.radar_square_with_scoreboard = true;
    s.hud_scaling = 0.9;
    s.safezonex = 1.0;
    s.safezoney = 1.0;
    s.cl_hud_color = 0;
    s.close_buymenu_after_purchase = true;
    s.always_show_inventory = false;
    s.net_graph = false;
    s.game_instructor = false;
    s.hide_avatar_images = false;
    s.teamcounter_playercount = false;
    s.mute_enemy_team = false;
    s.telemetry_fps = TelemetryMode::MatchOnly;
    s.telemetry_ping = TelemetryMode::MatchOnly;
    s.telemetry_packet_loss = TelemetryMode::MatchOnly;
    s.telemetry_net_quality = TelemetryMode::Never;

    // Viewmodel / game
    s.enable_console = true;
    s.cl_righthand = true;
    s.viewmodel_fov = 68.0;
    s.viewmodel_offset_x = 2.5;
    s.viewmodel_offset_y = 0.0;
    s.viewmodel_offset_z = -1.5;
    s.viewmodel_presetpos = 1;
    s.cl_autowepswitch = false;
    s.cl_join_advertise = 2;
    s.mm_dedicated_search_maxping = 80;
    s.maxping = 80;

    // Net
    s.rate = 786432;
    s.cl_interp_ratio = 1;
    s.cl_net_buffer_ticks = 0;

    // Display defaults (templates may override)
    s.display_mode = DisplayMode::Fullscreen;
    s.laptop_power_savings = false;
    s.vsync = false;
    s.multicore = true;
    s.motion_blur = false;
    s.fxaa_cmaa = false;
    s.boost_player_contrast = true;
    s.nvidia_gsync = false;
    s.video_preset = 0;

    // Crosshair defaults (Rush Hour pixel cvars) — overlays replace
    {
        let c = &mut s.crosshair;
        c.share_code.clear();
        c.style = CrosshairStyle::StaticCross;
        c.length = 2.0;
        c.gap = 1.0;
        c.thickness = 1.0;
        c.screen_height = 1080.0;
        c.color_r = 0;
        c.color_g = 255;
        c.color_b = 0;
        c.color_a = 255;
        c.outline = OutlineMode::Full;
        c.outline_r = 0;
        c.outline_g = 0;
        c.outline_b = 0;
        c.outline_a = 255;
        c.outlinethickness = 1.0;
        c.dot = false;
        c.t_style = false;
        c.follow_recoil = false;
        c.friendly_warning = true;
        c.sniper_width = 1.0;
        c.sniper_show_normal_inaccuracy = false;
        c.ironsight_dot_scale = 1.0;
        c.ironsight_use_crosshair_color = false;
        c.show_observer_crosshair = 2;
        c.observed_bot_crosshair = 1;
        c.dynamic_spread_limit = 255.0;
        c.dynamic_splitdist = 3.0;
        c.dynamic_splitalpha_innermod = 0.0;
        c.dynamic_splitalpha_outermod = 1.0;
        c.dynamic_maxdist_splitratio = 1.0;
    }

    s.launch_options = "-novid -fullscreen -console".into();
    s.custom_commands = vec![
        CustomCommand {
            line: "// cl_showfps 1".into(),
            enabled: false,
        },
    ];
}

fn apply_donk(s: &mut Settings) {
    let c = &mut s.crosshair;
    c.share_code = "CSGf8YDobjep5Bk3LiFQUu8ZupRL26TnO4diZ9Z9EcVeyk".into();
    c.style = CrosshairStyle::StaticCrossShotFeedback;
    c.length = 2.0;
    c.thickness = 1.0;
    c.gap = 0.0;
    c.color_r = 255;
    c.color_g = 255;
    c.color_b = 255;
    c.color_a = 255;
    c.outline = OutlineMode::Full;
    c.outline_r = 27;
    c.outline_g = 27;
    c.outline_b = 27;
    c.outline_a = 255;
    c.dot = true;
    c.t_style = false;
    s.resolution_w = 1280;
    s.resolution_h = 960;
    s.aspect_mode = 2;
    s.display_mode = DisplayMode::Fullscreen;
    s.brightness = 1.0 + 1.6 * 0.87;
    s.boost_player_contrast = true;
    s.vsync = false;
    s.nvidia_reflex = LowLatency::Disabled;
    s.nvidia_gsync = false;
    s.fps_max = 600.0;
    s.msaa = Msaa::X8;
    s.shadow_quality = Quality::High;
    s.dynamic_shadows = true;
    s.model_texture = Quality::Low;
    s.texture_filter = TextureFilter::Bilinear;
    s.shader_detail = Quality::Low;
    s.particle_detail = Quality::Low;
    s.ambient_occlusion = false;
    s.hdr_detail = 0;
    s.fsr_quality = 0;
    s.sensitivity = 1.25;
    s.zoom_sensitivity_ratio = 1.0;
    s.viewmodel_fov = 68.0;
    s.viewmodel_offset_x = 2.5;
    s.viewmodel_offset_y = 0.0;
    s.viewmodel_offset_z = -1.5;
    s.viewmodel_presetpos = 2;
    s.launch_options = String::new();
}

fn apply_nuro(s: &mut Settings) {
    let c = &mut s.crosshair;
    c.share_code.clear();
    c.style = CrosshairStyle::StaticCross;
    c.length = 2.0;
    c.thickness = 0.5;
    c.gap = -3.0;
    c.dot = false;
    c.t_style = false;
    c.outline = OutlineMode::Off;
    c.color_r = 0;
    c.color_g = 255;
    c.color_b = 0;
    c.color_a = 255;
    s.resolution_w = 1440;
    s.resolution_h = 1080;
    s.aspect_mode = 2;
    s.display_mode = DisplayMode::Fullscreen;
    s.fps_max = 400.0;
    s.msaa = Msaa::X4;
    s.shadow_quality = Quality::Low;
    s.model_texture = Quality::Low;
    s.shader_detail = Quality::Low;
    s.particle_detail = Quality::Low;
    s.texture_filter = TextureFilter::Aniso4x;
    s.vsync = false;
    s.boost_player_contrast = true;
    s.fsr_quality = 0;
    s.nvidia_reflex = LowLatency::EnabledBoost;
    s.sensitivity = 1.0;
    s.zoom_sensitivity_ratio = 1.0;
    s.radar_scale = 0.4;
    s.radar_rotate = false;
    s.radar_always_centered = false;
    s.radar_icon_scale = 0.6;
    s.radar_square_with_scoreboard = true;
    for (k, cmd) in [
        ("x", "slot6"),
        ("c", "slot8"),
        ("v", "slot7"),
        ("z", "slot10"),
        ("space", "+jump"),
        ("mwheelup", "+jump"),
        ("mwheeldown", "+jump"),
    ] {
        if let Some(b) = s.binds.iter_mut().find(|b| b.key.eq_ignore_ascii_case(k)) {
            b.command = cmd.into();
        } else {
            s.binds.push(Bind {
                key: k.into(),
                command: cmd.into(),
            });
        }
    }
    s.hide_avatar_images = true;
    s.teamcounter_playercount = true;
    s.mute_enemy_team = true;
    s.telemetry_fps = TelemetryMode::Always;
    s.telemetry_ping = TelemetryMode::Always;
    s.telemetry_net_quality = TelemetryMode::Always;
    s.telemetry_packet_loss = TelemetryMode::Always;
    s.launch_options = "-novid -fullscreen -console +fps_max 400".into();
}

fn apply_competitive_low_spec(s: &mut Settings) {
    pro_base(s);
    s.resolution_w = 1920;
    s.resolution_h = 1080;
    s.aspect_mode = 1;
    s.refresh_hz = 144;
    s.brightness = 1.6;
    s.fps_max = 400.0;
    s.fps_max_ui = 120.0;
    s.msaa = Msaa::None;
    s.shadow_quality = Quality::Low;
    s.dynamic_shadows = true;
    s.model_texture = Quality::Low;
    s.texture_filter = TextureFilter::Bilinear;
    s.shader_detail = Quality::Low;
    s.particle_detail = Quality::Low;
    s.ambient_occlusion = false;
    s.hdr_detail = -1;
    s.fsr_quality = 0;
    s.nvidia_reflex = LowLatency::EnabledBoost;
    s.video_preset = 1;
    {
        let c = &mut s.crosshair;
        c.style = CrosshairStyle::StaticCross;
        c.length = 2.0;
        c.thickness = 1.0;
        c.gap = 0.0;
        c.color_r = 0;
        c.color_g = 255;
        c.color_b = 255;
        c.color_a = 255;
        c.outline = OutlineMode::Full;
        c.dot = false;
    }
    s.launch_options = "-novid -fullscreen -console -high +fps_max 400".into();
}

fn apply_competitive_high_spec(s: &mut Settings) {
    pro_base(s);
    s.resolution_w = 1920;
    s.resolution_h = 1080;
    s.aspect_mode = 1;
    s.refresh_hz = 240;
    s.brightness = 1.7;
    s.fps_max = 0.0; // uncapped
    s.fps_max_ui = 200.0;
    s.msaa = Msaa::X4;
    s.shadow_quality = Quality::High;
    s.dynamic_shadows = true;
    s.model_texture = Quality::Medium;
    s.texture_filter = TextureFilter::Aniso8x;
    s.shader_detail = Quality::High;
    s.particle_detail = Quality::Medium;
    s.ambient_occlusion = false;
    s.hdr_detail = 0;
    s.fsr_quality = 0;
    s.nvidia_reflex = LowLatency::Enabled;
    s.video_preset = 3;
    {
        let c = &mut s.crosshair;
        c.style = CrosshairStyle::StaticCross;
        c.length = 2.0;
        c.thickness = 1.0;
        c.gap = 1.0;
        c.color_r = 74;
        c.color_g = 192;
        c.color_b = 192;
        c.color_a = 255;
        c.outline = OutlineMode::Full;
        c.dot = false;
    }
}

fn apply_faceit_premier_aim(s: &mut Settings) {
    pro_base(s);
    s.resolution_w = 1280;
    s.resolution_h = 960;
    s.aspect_mode = 2;
    s.refresh_hz = 240;
    s.brightness = 1.75;
    s.fps_max = 400.0;
    s.msaa = Msaa::None;
    s.shadow_quality = Quality::Low;
    s.dynamic_shadows = true;
    s.model_texture = Quality::Low;
    s.texture_filter = TextureFilter::Bilinear;
    s.shader_detail = Quality::Low;
    s.particle_detail = Quality::Low;
    s.ambient_occlusion = false;
    s.hdr_detail = -1;
    s.nvidia_reflex = LowLatency::EnabledBoost;
    s.hud_scaling = 0.8;
    s.radar_scale = 0.55;
    s.rate = 1000000;
    s.mm_dedicated_search_maxping = 50;
    s.maxping = 50;
    {
        let c = &mut s.crosshair;
        c.style = CrosshairStyle::StaticCross;
        c.length = 1.5;
        c.thickness = 0.5;
        c.gap = -2.0;
        c.color_r = 0;
        c.color_g = 255;
        c.color_b = 0;
        c.color_a = 255;
        c.outline = OutlineMode::Off;
        c.dot = false;
        c.t_style = false;
    }
    s.hide_avatar_images = true;
    s.teamcounter_playercount = true;
    s.launch_options = "-novid -fullscreen -console +fps_max 400".into();
}

fn apply_awp_scout_focus(s: &mut Settings) {
    pro_base(s);
    s.resolution_w = 1920;
    s.resolution_h = 1080;
    s.aspect_mode = 1;
    s.brightness = 1.65;
    s.fps_max = 400.0;
    s.msaa = Msaa::X2;
    s.shadow_quality = Quality::Medium;
    s.model_texture = Quality::Medium;
    s.texture_filter = TextureFilter::Aniso16x;
    s.shader_detail = Quality::Medium;
    s.particle_detail = Quality::Low;
    s.nvidia_reflex = LowLatency::Enabled;
    s.zoom_sensitivity_ratio = 0.9;
    s.cl_debounce_zoom = true;
    {
        let c = &mut s.crosshair;
        c.style = CrosshairStyle::StaticCross;
        c.length = 2.0;
        c.thickness = 1.0;
        c.gap = 0.0;
        c.color_r = 255;
        c.color_g = 220;
        c.color_b = 0;
        c.color_a = 255;
        c.outline = OutlineMode::Full;
        c.dot = true;
        c.sniper_width = 2.0;
        c.sniper_show_normal_inaccuracy = false;
        c.ironsight_dot_scale = 1.2;
        c.ironsight_use_crosshair_color = true;
    }
}

fn apply_spray_control(s: &mut Settings) {
    pro_base(s);
    s.resolution_w = 1920;
    s.resolution_h = 1080;
    s.aspect_mode = 1;
    s.brightness = 1.6;
    s.fps_max = 400.0;
    s.msaa = Msaa::None;
    s.shadow_quality = Quality::Low;
    s.model_texture = Quality::Low;
    s.texture_filter = TextureFilter::Aniso4x;
    s.shader_detail = Quality::Low;
    s.particle_detail = Quality::Low;
    s.nvidia_reflex = LowLatency::EnabledBoost;
    {
        let c = &mut s.crosshair;
        c.style = CrosshairStyle::DynamicCross;
        c.length = 3.0;
        c.thickness = 1.5;
        c.gap = 0.0;
        c.color_r = 0;
        c.color_g = 255;
        c.color_b = 255;
        c.color_a = 255;
        c.outline = OutlineMode::Full;
        c.dot = false;
        c.follow_recoil = true;
        c.dynamic_spread_limit = 255.0;
        c.dynamic_splitdist = 7.0;
    }
}

fn apply_visibility_max(s: &mut Settings) {
    pro_base(s);
    s.resolution_w = 1920;
    s.resolution_h = 1080;
    s.aspect_mode = 1;
    s.brightness = 2.0;
    s.boost_player_contrast = true;
    s.fps_max = 400.0;
    s.msaa = Msaa::X8;
    s.shadow_quality = Quality::High;
    s.dynamic_shadows = true;
    s.model_texture = Quality::Medium;
    s.texture_filter = TextureFilter::Aniso8x;
    s.shader_detail = Quality::Low;
    s.particle_detail = Quality::Low;
    s.ambient_occlusion = false;
    s.hdr_detail = 0;
    s.nvidia_reflex = LowLatency::Enabled;
    {
        let c = &mut s.crosshair;
        c.style = CrosshairStyle::StaticCross;
        c.length = 2.5;
        c.thickness = 1.0;
        c.gap = 1.0;
        c.color_r = 255;
        c.color_g = 0;
        c.color_b = 255;
        c.color_a = 255;
        c.outline = OutlineMode::Full;
        c.outline_r = 0;
        c.outline_g = 0;
        c.outline_b = 0;
        c.dot = false;
    }
}

fn apply_minimal_hud(s: &mut Settings) {
    pro_base(s);
    s.resolution_w = 1920;
    s.resolution_h = 1080;
    s.aspect_mode = 1;
    s.brightness = 1.6;
    s.fps_max = 400.0;
    s.msaa = Msaa::None;
    s.shadow_quality = Quality::Low;
    s.model_texture = Quality::Low;
    s.texture_filter = TextureFilter::Bilinear;
    s.shader_detail = Quality::Low;
    s.particle_detail = Quality::Low;
    s.nvidia_reflex = LowLatency::EnabledBoost;
    s.hud_scaling = 0.6;
    s.radar_hud_size = 0.7;
    s.radar_scale = 0.5;
    s.radar_icon_scale = 0.5;
    s.net_graph = false;
    s.hide_avatar_images = true;
    s.teamcounter_playercount = true;
    s.telemetry_fps = TelemetryMode::Never;
    s.telemetry_ping = TelemetryMode::Never;
    s.telemetry_packet_loss = TelemetryMode::Never;
    s.telemetry_net_quality = TelemetryMode::Never;
    s.music_volume = 0.0;
    s.mvp_volume = 0.0;
    {
        let c = &mut s.crosshair;
        c.style = CrosshairStyle::StaticCross;
        c.length = 1.0;
        c.thickness = 0.5;
        c.gap = -1.0;
        c.color_r = 255;
        c.color_g = 255;
        c.color_b = 255;
        c.color_a = 220;
        c.outline = OutlineMode::Off;
        c.dot = false;
    }
}

fn apply_streaming_content(s: &mut Settings) {
    pro_base(s);
    s.resolution_w = 1920;
    s.resolution_h = 1080;
    s.aspect_mode = 1;
    s.display_mode = DisplayMode::FullscreenWindowed;
    s.brightness = 1.55;
    s.fps_max = 240.0;
    s.fps_max_ui = 144.0;
    s.msaa = Msaa::X4;
    s.shadow_quality = Quality::High;
    s.model_texture = Quality::High;
    s.texture_filter = TextureFilter::Aniso16x;
    s.shader_detail = Quality::High;
    s.particle_detail = Quality::High;
    s.ambient_occlusion = true;
    s.hdr_detail = 0;
    s.nvidia_reflex = LowLatency::Enabled;
    s.music_volume = 0.15;
    s.menumusic_volume = 0.1;
    s.mvp_volume = 0.2;
    s.roundend_volume = 0.15;
    s.mute_mvp_music_live_players = false;
    s.telemetry_fps = TelemetryMode::Always;
    {
        let c = &mut s.crosshair;
        c.style = CrosshairStyle::StaticCross;
        c.length = 2.0;
        c.thickness = 1.0;
        c.gap = 1.0;
        c.color_r = 141;
        c.color_g = 182;
        c.color_b = 0;
        c.color_a = 255;
        c.outline = OutlineMode::Full;
        c.dot = false;
    }
    s.launch_options = "-novid -console".into();
}

fn apply_laptop_power(s: &mut Settings) {
    pro_base(s);
    s.resolution_w = 1920;
    s.resolution_h = 1080;
    s.aspect_mode = 1;
    s.laptop_power_savings = true;
    s.brightness = 1.5;
    s.fps_max = 144.0;
    s.fps_max_ui = 60.0;
    s.msaa = Msaa::None;
    s.shadow_quality = Quality::Low;
    s.dynamic_shadows = false;
    s.model_texture = Quality::Low;
    s.texture_filter = TextureFilter::Bilinear;
    s.shader_detail = Quality::Low;
    s.particle_detail = Quality::Low;
    s.ambient_occlusion = false;
    s.hdr_detail = -1;
    s.fsr_quality = 3; // Balanced
    s.nvidia_reflex = LowLatency::Disabled;
    s.vsync = true;
    {
        let c = &mut s.crosshair;
        c.style = CrosshairStyle::StaticCross;
        c.length = 2.0;
        c.thickness = 1.0;
        c.gap = 0.0;
        c.color_r = 100;
        c.color_g = 200;
        c.color_b = 200;
        c.color_a = 230;
        c.outline = OutlineMode::Half;
        c.dot = false;
    }
    s.launch_options = "-novid -console +fps_max 144".into();
}

fn apply_practice_grenade(s: &mut Settings) {
    pro_base(s);
    s.resolution_w = 1920;
    s.resolution_h = 1080;
    s.aspect_mode = 1;
    s.brightness = 1.6;
    s.fps_max = 300.0;
    s.msaa = Msaa::X2;
    s.shadow_quality = Quality::Medium;
    s.model_texture = Quality::Medium;
    s.texture_filter = TextureFilter::Aniso4x;
    s.shader_detail = Quality::Medium;
    s.particle_detail = Quality::High; // see nade smoke clearly
    s.nvidia_reflex = LowLatency::Enabled;
    // Nade binds
    for (k, cmd) in [
        ("c", "slot6"),  // HE
        ("x", "slot8"),  // smoke
        ("v", "slot7"),  // flash
        ("z", "slot10"), // decoy / molly slot vary
    ] {
        if let Some(b) = s.binds.iter_mut().find(|b| b.key.eq_ignore_ascii_case(k)) {
            b.command = cmd.into();
        } else {
            s.binds.push(Bind {
                key: k.into(),
                command: cmd.into(),
            });
        }
    }
    {
        let c = &mut s.crosshair;
        c.style = CrosshairStyle::StaticCross;
        c.length = 4.0;
        c.thickness = 1.0;
        c.gap = 4.0; // large gap helps lineup markers
        c.color_r = 0;
        c.color_g = 255;
        c.color_b = 0;
        c.color_a = 255;
        c.outline = OutlineMode::Full;
        c.dot = true;
    }
    // Practice / sv_cheats lines — DISABLED by default (commented style + enabled:false)
    s.custom_commands = vec![
        CustomCommand {
            line: "// sv_cheats 1".into(),
            enabled: false,
        },
        CustomCommand {
            line: "// sv_infinite_ammo 1".into(),
            enabled: false,
        },
        CustomCommand {
            line: "// mp_roundtime_defuse 60".into(),
            enabled: false,
        },
        CustomCommand {
            line: "// mp_buy_anywhere 1".into(),
            enabled: false,
        },
        CustomCommand {
            line: "// mp_maxmoney 60000; mp_startmoney 60000".into(),
            enabled: false,
        },
        CustomCommand {
            line: "// sv_grenade_trajectory_prac_pipreview 1".into(),
            enabled: false,
        },
        CustomCommand {
            line: "// sv_showimpacts 1".into(),
            enabled: false,
        },
        CustomCommand {
            line: "// bot_kick; bot_stop 1".into(),
            enabled: false,
        },
        CustomCommand {
            line: "// ammo_grenade_limit_total 5".into(),
            enabled: false,
        },
    ];
    s.launch_options = "-novid -console".into();
}

/// Mark Tella "RNG crosshair", rewritten for Rush Hour names (no cl_crosshairsize).
pub fn randomize_crosshair(c: &mut Crosshair) {
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1)
        | 1;
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed >> 11) as f64 / (1u64 << 53) as f64
    };
    let r1 = |x: f64| (x * 10.0).round() as f32 / 10.0;
    c.share_code.clear();
    c.style = CrosshairStyle::StaticCross;
    c.length = r1(0.5 + next() * 5.5);
    c.thickness = r1(0.5 + next() * 2.5);
    c.gap = r1(-4.0 + next() * 9.0);
    c.dot = next() < 0.5;
    c.color_a = (100.0 + next() * 155.0) as u8;
    c.outline = if next() < 0.5 {
        OutlineMode::Full
    } else {
        OutlineMode::Off
    };
    c.outlinethickness = if c.outline == OutlineMode::Off {
        1.0
    } else {
        r1(0.5 + next() * 2.0)
    };
    c.color_r = (next() * 255.0) as u8;
    c.color_g = (next() * 255.0) as u8;
    c.color_b = (next() * 255.0) as u8;
}
