use std::io::{stdout, Write};
use crossterm::{execute};
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

        enable_raw_mode().ok();
        execute!(stdout(), EnterAlternateScreen, MoveTo(0, 0)).expect("Failed to enter Alternate screen mode.");


        let sz = size()?;
        Ok(Self {
            size: Size { columns: sz.0, rows: sz.1 },
        })
    }

    pub fn shutdown(&self) {
        execute!(stdout(), LeaveAlternateScreen).expect("Issue leaving alternate screen mode.");
        disable_raw_mode().ok();
    }

    pub fn size(&self) -> &Size {
        &self.size
    }

    pub fn read_key(&self) -> Result<Event, std::io::Error>  {
        read()
    }

    pub fn clear_screen(&self) {
        execute!(stdout(), Clear(ClearType::All)).ok();
    }

    pub fn clear_current_line(&self) {
        execute!(stdout(), Clear(ClearType::CurrentLine)).ok();
    }

     #[allow(clippy::cast_possible_truncation)]
    pub fn cursor_position(position: &Position) {
         let Position{mut x, mut y} = position;
         let x = x as u16;
         let y = y as u16;

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
