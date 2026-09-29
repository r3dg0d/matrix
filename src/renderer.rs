//! Terminal rendering via crossterm.

use crate::column::RainField;
use crate::palette::Palette;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::io::{stdout, Stdout, Write};

/// RAII guard: restores terminal on drop (incl. panic unwind).
pub struct TerminalGuard {
    stdout: Stdout,
    active: bool,
}

impl TerminalGuard {
    pub fn enter() -> anyhow::Result<Self> {
        let mut out = stdout();
        enable_raw_mode()?;
        execute!(out, EnterAlternateScreen, Hide)?;
        Ok(Self {
            stdout: out,
            active: true,
        })
    }

    pub fn stdout(&mut self) -> &mut Stdout {
        &mut self.stdout
    }

    pub fn restore(&mut self) {
        if !self.active {
            return;
        }
        let _ = execute!(self.stdout, ResetColor, Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
        self.active = false;
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        self.restore();
    }
}

pub fn draw_frame(
    out: &mut Stdout,
    field: &RainField,
    palette: &Palette,
    truecolor: bool,
    paused: bool,
    overlay: Option<&str>,
) -> anyhow::Result<()> {
    queue!(
        out,
        SetBackgroundColor(palette.bg.to_color(truecolor)),
        Clear(ClearType::All)
    )?;

    for col in &field.columns {
        for (y, cell) in col.cells.iter().enumerate() {
            if let Some(cell) = cell {
                let color = palette.trail_color(cell.age, cell.is_lead, truecolor);
                queue!(
                    out,
                    MoveTo(col.x, y as u16),
                    SetForegroundColor(color),
                    Print(cell.ch)
                )?;
            }
        }
    }

    if paused {
        draw_status(
            out,
            field.width,
            field.height,
            "[ paused — space to resume ]",
            truecolor,
            palette,
        )?;
    }

    if let Some(text) = overlay {
        draw_overlay(out, field.width, field.height, text, truecolor, palette)?;
    }

    out.flush()?;
    Ok(())
}

fn draw_status(
    out: &mut Stdout,
    width: u16,
    height: u16,
    msg: &str,
    truecolor: bool,
    palette: &Palette,
) -> anyhow::Result<()> {
    let y = height.saturating_sub(1);
    let x = width
        .saturating_sub(msg.chars().count() as u16)
        .saturating_div(2);
    queue!(
        out,
        MoveTo(x, y),
        SetForegroundColor(palette.lead.to_color(truecolor)),
        Print(msg)
    )?;
    Ok(())
}

fn draw_overlay(
    out: &mut Stdout,
    width: u16,
    height: u16,
    text: &str,
    truecolor: bool,
    palette: &Palette,
) -> anyhow::Result<()> {
    let lines: Vec<&str> = text.lines().collect();
    let box_w = lines
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0)
        .min(width as usize);
    let box_h = lines.len().min(height as usize);
    let start_x = width.saturating_sub(box_w as u16).saturating_div(2);
    let start_y = height.saturating_sub(box_h as u16).saturating_div(2);

    for (i, line) in lines.iter().take(box_h).enumerate() {
        let padded: String = format!("{:<width$}", line, width = box_w);
        queue!(
            out,
            MoveTo(start_x, start_y + i as u16),
            SetBackgroundColor(palette.dim.to_color(truecolor)),
            SetForegroundColor(palette.lead.to_color(truecolor)),
            Print(&padded),
            SetBackgroundColor(palette.bg.to_color(truecolor))
        )?;
    }
    Ok(())
}

pub const HELP_OVERLAY: &str = "\
┌─ matrix controls ─────────────────┐
│ q / Ctrl+C   quit                 │
│ space        pause / resume       │
│ + / =        faster               │
│ - / _        slower               │
│ ]            denser               │
│ [            sparser              │
│ c            cycle palette        │
│ p            cycle preset         │
│ ? / h        toggle this help     │
└───────────────────────────────────┘";
