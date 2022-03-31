pub struct Row {
    string: String
}

impl Row {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
        }
    }

}