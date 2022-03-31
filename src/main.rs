mod editor;
mod terminal;
mod position;
mod document;
mod row;

use crate::editor::Editor;
use crate::position::Position;


fn main() {


    Editor::default().run();

}
