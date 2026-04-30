use std::{
    io::{self, Write},
    time::Duration,
};

use anyhow::{Result, bail};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEventKind},
    execute, queue,
    style::{
        Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};

use crate::version;

#[derive(Debug, Eq, PartialEq)]
pub enum Selection {
    Selected(usize),
    Back,
    Quit,
}

pub fn select_item(title: &str, subtitle: &str, items: &[String]) -> Result<Selection> {
    select_entries(title, subtitle, items, false)
}

pub fn select_version(title: &str, subtitle: &str, items: &[String]) -> Result<Selection> {
    select_entries(title, subtitle, items, true)
}

fn select_entries(
    title: &str,
    subtitle: &str,
    items: &[String],
    alpha_toggle: bool,
) -> Result<Selection> {
    if items.is_empty() {
        bail!("no items to select");
    }

    let mut terminal = TerminalSession::new()?;
    let mut selected = 0usize;
    let mut show_alpha = alpha_toggle && !items.iter().any(|item| !version::has_alpha(item));

    loop {
        let visible = visible_indices(items, show_alpha, alpha_toggle);
        selected = selected.min(visible.len().saturating_sub(1));
        terminal.draw(
            title,
            subtitle,
            items,
            &visible,
            selected,
            alpha_toggle,
            show_alpha,
        )?;

        if !event::poll(Duration::from_millis(250))? {
            continue;
        }

        let Event::Key(key) = event::read()? else {
            continue;
        };

        if key.kind == KeyEventKind::Release {
            continue;
        }

        match key.code {
            KeyCode::Char('q') => return Ok(Selection::Quit),
            KeyCode::Esc => return Ok(Selection::Back),
            KeyCode::Char('.') if alpha_toggle => {
                show_alpha = !show_alpha;
                selected = 0;
            }
            KeyCode::Char('j') | KeyCode::Down => selected = (selected + 1).min(visible.len() - 1),
            KeyCode::Char('k') | KeyCode::Up => selected = selected.saturating_sub(1),
            KeyCode::PageDown => selected = (selected + 10).min(visible.len() - 1),
            KeyCode::PageUp => selected = selected.saturating_sub(10),
            KeyCode::Home => selected = 0,
            KeyCode::End => selected = visible.len() - 1,
            KeyCode::Enter => return Ok(Selection::Selected(visible[selected])),
            _ => {}
        }
    }
}

fn visible_indices(items: &[String], show_alpha: bool, alpha_toggle: bool) -> Vec<usize> {
    items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            if alpha_toggle && !show_alpha && version::has_alpha(item) {
                None
            } else {
                Some(index)
            }
        })
        .collect()
}

struct TerminalSession {
    stdout: io::Stdout,
}

impl TerminalSession {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, Hide)?;
        Ok(Self { stdout })
    }

    fn draw(
        &mut self,
        title: &str,
        subtitle: &str,
        items: &[String],
        visible: &[usize],
        selected: usize,
        alpha_toggle: bool,
        show_alpha: bool,
    ) -> Result<()> {
        let (_, height) = crossterm::terminal::size()?;
        let list_height = height.saturating_sub(6).max(1) as usize;
        let start = selected.saturating_sub(list_height.saturating_sub(1));
        let end = (start + list_height).min(visible.len());
        let status = if alpha_toggle {
            if show_alpha {
                "  [. alpha shown]"
            } else {
                "  [. alpha hidden]"
            }
        } else {
            ""
        };
        let footer = if alpha_toggle {
            "Enter select  Up/Down move  PgUp/PgDn jump  . alpha  Esc back  q quit"
        } else {
            "Enter select  Up/Down move  PgUp/PgDn jump  Esc back  q quit"
        };

        queue!(
            self.stdout,
            MoveTo(0, 0),
            Clear(ClearType::All),
            SetForegroundColor(Color::Cyan),
            SetAttribute(Attribute::Bold),
            Print("minit"),
            ResetColor,
            SetAttribute(Attribute::Reset),
            Print(" / "),
            SetAttribute(Attribute::Bold),
            Print(title),
            SetAttribute(Attribute::Reset),
            MoveTo(0, 1),
            Print(format!("{subtitle}{status}")),
            MoveTo(0, 2),
            Print("-".repeat(80))
        )?;

        for (row, visible_index) in (start..end).enumerate() {
            let item_index = visible[visible_index];
            let alpha = version::has_alpha(&items[item_index]);
            queue!(self.stdout, MoveTo(0, (row + 3) as u16))?;
            if visible_index == selected {
                queue!(
                    self.stdout,
                    SetForegroundColor(Color::Black),
                    SetBackgroundColor(Color::Cyan),
                    SetAttribute(Attribute::Bold),
                    Print(format!("> {}", items[item_index])),
                    ResetColor,
                    SetAttribute(Attribute::Reset)
                )?;
            } else if alpha {
                queue!(
                    self.stdout,
                    SetForegroundColor(Color::DarkGrey),
                    Print(format!("  {}", items[item_index])),
                    ResetColor
                )?;
            } else {
                queue!(self.stdout, Print(format!("  {}", items[item_index])))?;
            }
        }

        queue!(
            self.stdout,
            MoveTo(0, height.saturating_sub(2)),
            Print("-".repeat(80)),
            MoveTo(0, height.saturating_sub(1)),
            SetForegroundColor(Color::DarkGrey),
            Print(footer),
            ResetColor
        )?;
        self.stdout.flush()?;
        Ok(())
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.stdout, Show, LeaveAlternateScreen);
    }
}
