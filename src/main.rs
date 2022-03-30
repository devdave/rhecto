mod editor;
mod terminal;
mod position;

use crate::editor::Editor;
use crate::position::Position;


fn main() {


    Editor::default().run();

}
