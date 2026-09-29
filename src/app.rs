//! Main rain loop.

use crate::charset::Charset;
use crate::column::RainField;
use crate::config::Settings;
use crate::input::{self, Action};
use crate::palette::Palette;
use crate::presets::PresetId;
use crate::renderer::{self, TerminalGuard, HELP_OVERLAY};
use crossterm::terminal::size;
use rand::thread_rng;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub fn run(mut settings: Settings, running: Arc<AtomicBool>) -> anyhow::Result<()> {
    settings.clamp();

    let mut rng = thread_rng();
    let mut charset = build_charset(&settings)?;
    let mut palette = Palette::from_id(settings.palette);

    let (w, h) = size()?;
    let mut field = RainField::new(
        w,
        h,
        settings.density,
        settings.speed,
        settings.trail,
        &mut rng,
    );

    let mut guard = TerminalGuard::enter()?;
    let mut paused = false;
    let mut show_help = false;

    while running.load(Ordering::SeqCst) {
        let frame_start = Instant::now();
        let frame_budget = Duration::from_secs_f64(1.0 / settings.fps.max(1) as f64);

        // Drain input within frame budget.
        let mut remaining = frame_budget;
        loop {
            let t0 = Instant::now();
            let action = input::poll_action(remaining.min(Duration::from_millis(2)))?;
            match action {
                Action::None => break,
                Action::Quit => {
                    running.store(false, Ordering::SeqCst);
                    break;
                }
                Action::TogglePause => paused = !paused,
                Action::ToggleHelp => show_help = !show_help,
                Action::SpeedUp => {
                    settings.speed = (settings.speed * 1.15).min(8.0);
                    field.set_speed(settings.speed, &mut rng);
                }
                Action::SpeedDown => {
                    settings.speed = (settings.speed / 1.15).max(0.05);
                    field.set_speed(settings.speed, &mut rng);
                }
                Action::DensityUp => {
                    settings.density = (settings.density + 0.05).min(1.0);
                    let (w, h) = size()?;
                    field.rebuild(
                        w,
                        h,
                        settings.density,
                        settings.speed,
                        settings.trail,
                        &mut rng,
                    );
                }
                Action::DensityDown => {
                    settings.density = (settings.density - 0.05).max(0.05);
                    let (w, h) = size()?;
                    field.rebuild(
                        w,
                        h,
                        settings.density,
                        settings.speed,
                        settings.trail,
                        &mut rng,
                    );
                }
                Action::CyclePalette => {
                    settings.palette = settings.palette.next();
                    palette = Palette::from_id(settings.palette);
                }
                Action::CyclePreset => {
                    let next = settings.preset.unwrap_or(PresetId::Hacker).next();
                    apply_preset(
                        &mut settings,
                        next,
                        &mut charset,
                        &mut palette,
                        &mut field,
                        &mut rng,
                    )?;
                }
                Action::Resize => {
                    let (w, h) = size()?;
                    field.resize(
                        w,
                        h,
                        settings.density,
                        settings.speed,
                        settings.trail,
                        &mut rng,
                    );
                }
            }
            let spent = t0.elapsed();
            if spent >= remaining {
                break;
            }
            remaining -= spent;
        }

        if !running.load(Ordering::SeqCst) {
            break;
        }

        // Catch resize even if no event (some terminals).
        let (w, h) = size()?;
        if w != field.width || h != field.height {
            field.resize(
                w,
                h,
                settings.density,
                settings.speed,
                settings.trail,
                &mut rng,
            );
        }

        if !paused {
            field.tick(&charset, &mut rng);
        }

        let overlay = if show_help { Some(HELP_OVERLAY) } else { None };
        renderer::draw_frame(
            guard.stdout(),
            &field,
            &palette,
            settings.truecolor,
            paused,
            overlay,
        )?;

        let elapsed = frame_start.elapsed();
        if elapsed < frame_budget {
            std::thread::sleep(frame_budget - elapsed);
        }
    }

    guard.restore();
    Ok(())
}

fn build_charset(settings: &Settings) -> anyhow::Result<Charset> {
    if let Some(ref custom) = settings.custom_chars {
        Charset::from_custom(custom)
    } else {
        Ok(Charset::from_id(settings.charset))
    }
}

fn apply_preset(
    settings: &mut Settings,
    preset: PresetId,
    charset: &mut Charset,
    palette: &mut Palette,
    field: &mut RainField,
    rng: &mut impl rand::Rng,
) -> anyhow::Result<()> {
    let p = preset.apply();
    settings.preset = Some(preset);
    settings.speed = p.speed;
    settings.density = p.density;
    settings.fps = p.fps;
    settings.charset = p.charset;
    settings.palette = p.palette;
    settings.trail = p.trail;
    settings.custom_chars = None;
    settings.clamp();
    *charset = Charset::from_id(settings.charset);
    *palette = Palette::from_id(settings.palette);
    let (w, h) = size()?;
    field.rebuild(w, h, settings.density, settings.speed, settings.trail, rng);
    Ok(())
}
