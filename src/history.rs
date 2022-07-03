use std::collections::VecDeque;

pub struct History {
    undo_stack: VecDeque<String>,
    redo_stack: VecDeque<String>
}

impl Default for History {
    fn default() -> Self {
        Self { undo_stack: Default::default(), redo_stack: Default::default() }
    }
}

impl History {

    pub fn new() -> History {
        History {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
        }
    }

    pub fn apply(&mut self, current_state: String) {
        self.undo_stack.push_front(current_state);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, current_state: String) -> Result<String, bool> {
        if self.undo_stack.is_empty() {
            return Err(true);
        }
        self.redo_stack.push_front(current_state);
        return Ok(self.undo_stack.pop_front().unwrap());
    }

    pub fn redo(&mut self, current_state: String) -> Result<String, bool> {
        if self.redo_stack.is_empty() {
            return Err(true);
        }
        self.undo_stack.push_front(current_state);
        return Ok(self.redo_stack.pop_front().unwrap());
    }

}