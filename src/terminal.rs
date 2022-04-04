use std::io;
use std::io::{stdout, Write};
use crossterm::{ExecutableCommand, execute};
use crossterm::style::{Stylize, SetBackgroundColor, SetForegroundColor, ResetColor, Color};
use crossterm::cursor::{MoveTo, Show, Hide};
use crossterm::terminal::{size, ClearType, Clear};
use crossterm::event::{Event, read };
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode};
use crate::Position;

pub struct Size {
    pub columns: u16,
    pub rows: u16,
}

pub struct Terminal {
    size: Size,
}

impl Terminal {


    pub fn default() -> Result<Self, std::io::Error> {

        enable_raw_mode().expect("Unable to enter raw mode!");
        execute!(stdout(), EnterAlternateScreen, MoveTo(0, 0)).expect("Failed to enter Alternate screen mode.");



        let sz = size()?;
        Ok(Self {
            size: Size {
                columns: sz.0,
                rows: sz.1.saturating_sub(2),
            },
        })
    }

    pub fn shutdown(&self) {
        disable_raw_mode().ok();
        execute!(stdout(), LeaveAlternateScreen).expect("Issue leaving alternate screen mode.");

    }

    pub fn size(&self) -> &Size {
        &self.size
    }

    pub fn read_key(&self) -> Result<Event, std::io::Error>  {
        read()
    }

    pub fn clear_screen(&self) {
        execute!(stdout(), Clear(ClearType::Purge)).ok();
    }

    pub fn clear_current_line(&self) {
        execute!(stdout(), Clear(ClearType::CurrentLine)).ok();
    }

    pub fn set_bg_color(&self, color: Color) {
        stdout().execute(SetBackgroundColor(color)).expect("Failed changing background color");
    }

    pub fn reset_bg_color(&self) {
        stdout().execute(SetBackgroundColor(Color::Reset)).expect("Unable to reset bg color!");
    }

    pub fn set_fg_color(&self, color: Color) {
        stdout().execute(SetForegroundColor(color)).expect("Failed to set FG color");
    }

    pub fn reset_fg_color(&self) {
        stdout().execute(SetForegroundColor(Color::Reset)).expect("Failed to reset FG color");
    }

     #[allow(clippy::cast_possible_truncation)]
    pub fn cursor_position(&self, position: &Position) {
         let Position{x, y} = position;
         let x = *x as u16;
         let y = *y as u16;

        execute!(stdout(), MoveTo(x, y)).expect("Failed to move cursor");
    }

    pub fn flush(&self) {
        stdout().flush().expect("Failed to flush to stdout!");
    }

    pub fn cursor_hide(&self) {
        execute!(stdout(), Hide).expect("Failed to hide cursor");
    }

    pub fn cursor_show(&self) {
        execute!(stdout(), Show).expect("Failed to show cursor.");
    }
}
