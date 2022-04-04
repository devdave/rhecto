use crate::terminal::Terminal;
use crate::position::Position;
use crate::document::Document;
use crate::row::Row;
use crate::statusmessage::StatusMessage;

use std::env;
use std::fs::read_dir;
use std::io::ErrorKind::HostUnreachable;
// use std::io::{repeat, stdout};
use std::time::{Instant, Duration};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::event::Event::Key;
use crossterm::event::KeyCode::PageDown;
// use crossterm::{execute};
use crossterm::style::{PrintStyledContent, Print, SetForegroundColor, SetBackgroundColor, ResetColor, Color, Attribute, Stylize};


const VERSION: &str = env!("CARGO_PKG_VERSION");
const STATUS_BG_COLOR: Color = Color::Rgb{r: 239, g: 239, b: 239};
const STATUS_FG_COLOR: Color = Color::Rgb{r: 63, g: 63, b: 63};




pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
    status_message: StatusMessage,
}

impl Editor {

    pub fn default() -> Self {

        let args: Vec<String> = env::args().collect();
        let mut initial_status = String::from("HELP: Ctrl+s == save | Ctrl+q = quit");
        let document = if args.len() > 1 {
            let file_name = &args[1];
            // Document::open(&file_name).unwrap_or_default()
            let doc = Document::open(&file_name);
            if doc.is_ok() {
                doc.unwrap()
            } else {
                initial_status = format!("ERR: Could not open file: {}", file_name);
                Document::default()
            }
        } else {
            Document::default()
        };


        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize the terminal."),
            document,
            cursor_position: Position::default(),
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),

        }
    }


    pub fn run(&mut self) {



        loop {
            if let Err(error) = self.process_keypress() {
               println!("Oh no!");

            }
            self.refresh_screen();

            if self.should_quit {
                break;
            };


        }

        self.shutdown();
    }

    fn refresh_screen(&self) {
        self.terminal.cursor_hide();
        self.terminal.clear_screen();
        self.terminal.cursor_position(&Position::default());
        if self.should_quit {
            println!("Good bye!");
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();

            self.terminal.cursor_position( &Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            } );
        }

        self.terminal.cursor_show();
        self.terminal.flush();
    }

    fn save(&mut self) {
        if self.document.file_name.is_none() {

            let new_name = self.prompt("Save as: ").unwrap_or(None);

            if new_name.is_none() {
                self.status_message = StatusMessage::from("Save aborted!".to_string());
                return;
            }
            self.document.file_name = Some(new_name);
        }

        if self.document.save().is_ok() {
            self.status_message = StatusMessage::from("File saved successfully".to_string());
        } else {
            self.status_message = StatusMessage::from("Error writing file!".to_string());
        }
    }

    fn draw_status_bar(&self) {
        // let spaces = " ".repeat(self.terminal.size().columns as usize);

        let mut status;
        let width = self.terminal.size().columns as usize;
        let mut file_name = "[No Name]".to_string();
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);

        }


        status = format!("{} - {} lines", file_name, self.document.len());



        let line_indicator = format!("{}/{}",
                                     self.cursor_position.y.saturating_add(1),
                                     self.document.len(),
                                    );

        let len = status.len() + line_indicator.len();
        if width > len {
            status.push_str(&" ".repeat(width - len));
        }

        status = format!("{}{}", status, line_indicator);

        status.truncate(width);

        self.terminal.set_bg_color(STATUS_BG_COLOR);
        self.terminal.set_fg_color(STATUS_FG_COLOR);
        println!("{}\r", status);
        self.terminal.reset_bg_color();
        self.terminal.reset_fg_color();
    }

    fn draw_message_bar(&self) {
        self.terminal.clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().columns as usize);
            print!("{}",text);
        }

    }

    pub fn shutdown(&self) {
        self.terminal.shutdown();
    }

    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("Hecto Editor -- version({})\r", VERSION);
        let width = self.terminal.size().columns as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));

        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);

    }

    pub fn draw_row(&self, row: &Row) {

        let width = self.terminal.size().columns as usize;
        let start = self.offset.x;
        let end = self.offset.x + width;
        let row = row.render(start, end);

        println!("{}\r", row)
    }

    pub fn  draw_rows(&self) {
        self.terminal.clear_screen();

        let height = self.terminal.size().rows;
        // let width = self.terminal.size().columns;
        let is_empty = self.document.is_empty();

        for terminal_row in 0..height {
            self.terminal.clear_current_line();

            if let Some(row) = self.document.row(terminal_row as usize + self.offset.y) {
                self.draw_row(row);
            } else if is_empty && terminal_row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }

        }
    }

    pub fn process_keypress(&mut self) -> Result<(), std::io::Error> {
        let event: Event = self.terminal.read_key().unwrap();

        if let Event::Key(pressed_key) = event {

            match (pressed_key.modifiers, pressed_key.code) {

                (KeyModifiers::CONTROL, KeyCode::Char('q')) => {
                    self.should_quit = true;
                },
                (KeyModifiers::CONTROL, KeyCode::Char('s')) => self.save(),
                (_, KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right | KeyCode::PageUp | KeyCode:: PageDown | KeyCode::Home | KeyCode::End) => {
                    self.move_cursor(pressed_key);
                },
                (_, KeyCode::Delete) => {
                  self.document.delete(&self.cursor_position);
                },
                (_, KeyCode::Backspace) => {
                    if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                        self.move_cursor_by_key(KeyCode::Left);
                        self.document.delete(&self.cursor_position);
                    }
                },
                (_, KeyCode::Enter) => {
                    self.document.insert_newline(&self.cursor_position);
                    self.move_cursor_by_key(KeyCode::Down);
                },
                (_, KeyCode::Char(c)) => {
                    self.document.insert(&self.cursor_position, c);
                    self.move_cursor_by_key(KeyCode::Right);

                },
                _ => {
                    // println!("No idea {:?}", pressed_key);
                },


            }
        }

        self.scroll();
        Ok(())
    }

    fn prompt(&mut self, prompt: &str) -> Result<Option<String>, std::io::Error> {
        let mut result = String::new();

        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_screen();
            if let Event::Key(pressed_key) = self.terminal.read_key()? {

                match pressed_key.code {
                    KeyCode::Enter => break,
                    KeyCode::Char(c) => {
                        if pressed_key.modifiers != KeyModifiers::CONTROL {
                            result.push(c);
                        }
                    },
                    KeyCode::Backspace => {
                        if !result.is_empty() {
                            result.truncate(result.len()-1);
                        }
                    },
                    KeyCode::Esc => {
                        result.truncate(0);
                        break;
                    },
                    _ => (),

                }


            }

        }

        self.status_message = StatusMessage::from(String::new());
        if result.is_empty() {
            return Ok(None);
        }

        Ok(Some(result))

    }

    fn scroll(&mut self) {
        let Position {x, y} = self.cursor_position;
        let width = self.terminal.size().columns as usize;
        let height = self.terminal.size().rows as usize;

        let mut offset = &mut self.offset;
        if y < offset.y {
            offset.y = y;

        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }

        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    pub fn move_cursor_by_key(&mut self, key: KeyCode) {
        let terminal_height = self.terminal.size().rows as usize;
        let Position { mut x, mut y } = self.cursor_position;

        let height = self.document.len();
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        match key {
            KeyCode::Up => y = y.saturating_sub(1),
            KeyCode::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            },
            KeyCode::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }

            },
            KeyCode::Right => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            },
            KeyCode::PageUp => {
                y = if y > terminal_height {
                    y - terminal_height
                } else {
                    0
                };
            },
            KeyCode::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y + terminal_height as usize
                } else {
                    height
                };
            },
            KeyCode::Home => x = 0,
            KeyCode::End => x = width,
            _ => (),
        }

        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        if x > width {
            x = width;
        }

        self.cursor_position = Position { x, y }
    }


    pub fn move_cursor(&mut self, pressed_key: KeyEvent) {
        let terminal_height = self.terminal.size().rows as usize;
        let Position { mut x, mut y } = self.cursor_position;

        let height = self.document.len();
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };


        match pressed_key.code {
            KeyCode::Up => y = y.saturating_sub(1),
            KeyCode::Down => {
                if y < height {
                    y = y.saturating_add(1);
                }
            },
            KeyCode::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }

            },
            KeyCode::Right => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            },
            KeyCode::PageUp => {
                y = if y > terminal_height {
                    y - terminal_height
                } else {
                    0
                };
            },
            KeyCode::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y + terminal_height as usize
                } else {
                    height
                };
            },
            KeyCode::Home => x = 0,
            KeyCode::End => x = width,
            _ => (),
        }

        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        if x > width {
            x = width;
        }

        self.cursor_position = Position { x, y }

    }


}