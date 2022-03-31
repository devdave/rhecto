use crate::terminal::Terminal;
use crate::position::Position;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

const VERSION: &str = env!("CARGO_PKG_VERSION");





pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    cursor_position: Position,
}

impl Editor {

    pub fn default() -> Self {
        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize the terminal."),
            cursor_position: Position {x: 0, y: 0},
        }
    }

    pub fn startup(&mut self) {

        // self.draw_rows();
        // self.terminal.set_pos(0, 0).expect("Failed to move cursor position.");
        // println!("({}, {})", self.terminal.size().columns, self.terminal.size().rows);
        self.refresh_screen();

    }

    pub fn run(&mut self) {

        self.startup();

        loop {
            if let Err(error) = self.process_keypress() {
               println!("Oh no!");

            } else if self.should_quit {
                break;
            };
        }

        self.shutdown();
    }

    fn refresh_screen(&self) {
        self.terminal.cursor_hide();
        self.terminal.clear_screen();
        self.terminal.cursor_position(&Position { x: 0, y:0 });
        if self.should_quit {
            println!("Good bye!");
        } else {
            self.draw_rows();
            self.terminal.cursor_position(&Position {x: 0, y: 0});
        }

        self.terminal.cursor_show();
        self.terminal.flush();
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

    pub fn draw_rows(&self) {
        self.terminal.clear_screen();

        let rows = self.terminal.size().rows;

        for row in 0..rows {
            self.terminal.clear_current_line();
            if row == rows / 3 {
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
                (_, KeyCode::Char(c)) => {
                    print!("{}", c);
                    self.terminal.flush();
                },
                _ => {
                    println!("No idea {:?}", pressed_key);
                },

            }
        }

        Ok(())
    }

    fn move_cursor(&mut self, pressed_key: KeyEvent) {
        let Position { mut x, mut y } = self.cursor_position;

        match pressed_key.code {
            KeyCode::Up => y = y.saturating_sub(1),
            KeyCode::Down => y = y.saturating_add(1),
            KeyCode::Left => x = x.saturating_sub(1),
            KeyCode::Right => x = x.saturating_add(1),
            _ => (),
        }
        self.cursor_position = Position { x, y }
        
    }


}