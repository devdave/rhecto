use crate::terminal::Terminal;
use crate::position::Position;
use crate::document::Document;
use crate::row::Row;

use std::env;
use std::io::stdout;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::{execute};
use crossterm::style::{PrintStyledContent, Print, SetForegroundColor, SetBackgroundColor, ResetColor, Color, Attribute, Stylize};


const VERSION: &str = env!("CARGO_PKG_VERSION");
const STATUS_BG_COLOR: Color = Color::Rgb{r: 239, g: 239, b: 239};





pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
    offset: Position,
    document: Document,
}

impl Editor {

    pub fn default() -> Self {

        let args: Vec<String> = env::args().collect();
        let document = if args.len() > 1 {
            let file_name = &args[1];
            Document::open(&file_name).unwrap_or_default()
        } else {
            Document::default()
        };


        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize the terminal."),
            document,
            cursor_position: Position::default(),
            offset: Position::default(),

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

    fn draw_status_bar(&self) {
        let spaces = " ".repeat(self.terminal.size().columns as usize);
        // execute!(stdout(),
        //     SetBackgroundColor(STATUS_BG_COLOR),
        //     Print(spaces),
        //     ResetColor
        // );
        println!("{}{}\r", spaces.on(STATUS_BG_COLOR), "".reset());

    }

    fn draw_message_bar(&self) {
        self.terminal.clear_current_line();
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

        let height = self.terminal.size().rows - 1;
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
                (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
                    self.terminal.clear_screen();
                },
                (_, KeyCode::Enter) => {
                    print!("\r\n");
                },
                (_, KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right | KeyCode::PageUp | KeyCode:: PageDown | KeyCode::Home | KeyCode::End) => {
                    self.move_cursor(pressed_key);
                },
                (_, KeyCode::Char(c)) => {
                    print!("{}", c);
                    self.terminal.flush();
                },
                _ => {
                    println!("No idea {:?}", pressed_key);
                },


            }
        }

        self.scroll();
        Ok(())
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

    fn move_cursor(&mut self, pressed_key: KeyEvent) {
        let terminal_height = self.terminal.size().rows as usize;
        let Position { mut x, mut y } = self.cursor_position;
        let size = self.terminal.size();
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