// ANCHOR: state
use crate::books::Book;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct AppState {
    pub books: Arc<Mutex<Vec<Book>>>,
}
// ANCHOR_END: state
