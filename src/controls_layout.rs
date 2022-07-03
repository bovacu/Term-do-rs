use tui::backend::Backend;
use tui::Frame;
use tui::layout::{Alignment, Rect};
use tui::style::Style;
use tui::text::Spans;
use tui::widgets::{Block, Borders, Paragraph};
use crate::{App, DataManager, LayoutCommonTrait};

pub struct ControlsLayout;

impl ControlsLayout {
    pub fn new() -> ControlsLayout {
        ControlsLayout {  }
    }
}

impl LayoutCommonTrait for ControlsLayout {
    fn handle_input(&mut self, _data_manager: &mut DataManager, _key_code: crossterm::event::KeyEvent) {  }

    fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>, _frame_size: &Rect) {
        ControlsLayout::create_and_render_base_block(f, app, chunk);
    }

    fn create_and_render_base_block<B: Backend>(f: &mut Frame<B>, _app: &mut App, chunk: &Vec<Rect>) {
        let text = vec![
            Spans::from(""),
            Spans::from("In here you will find the controls of the app and some useful information"),
            Spans::from("The app saves the data with each modification, no client side save is needed"),
            Spans::from(""),
            Spans::from("-- Global controls --"),
            Spans::from("RightArrow to select the layout of the tasks"),
            Spans::from("LefArrow to select the layout of the groups"),
            Spans::from("'?' to show or hide controls info"),
            Spans::from("Esc/'q' to quit app or to hide controls info"),
            Spans::from("'u' undo"),
            Spans::from("'r' redo"),
            Spans::from(""),
            Spans::from("-- Groups controls --"),
            Spans::from("UpArrow to select the upper group"),
            Spans::from("DownArrow to select the lower group"),
            Spans::from("'a' to show input to create a new group"),
            Spans::from("Enter on 'a' to create the new group"),
            Spans::from("Esc on 'a' to cancel the new group"),
            Spans::from("'e' to show input to edit selected group"),
            Spans::from("Enter on 'a' to apply name change to group"),
            Spans::from("Esc on 'a' to cancel the changes to group"),
            Spans::from("'d' to delete a group (all tasks too)"),
            Spans::from(""),
            Spans::from("-- Tasks controls --"),
            Spans::from("UpArrow to select the upper task"),
            Spans::from("DownArrow to select the lower task"),
            Spans::from("'a' to show input to create a new task"),
            Spans::from("Enter on 'a' to create the new task"),
            Spans::from("Esc on 'a' to cancel the new task"),
            Spans::from("'A' to show input to create a new subtask on selected task"),
            Spans::from("Enter on 'A' to create the new subtask on selected task"),
            Spans::from("Esc on 'A' to cancel the new subtask on selected task"),
            Spans::from("'e' to show input to edit selected task or subtask"),
            Spans::from("Enter on 'a' to apply name change to task or subtask"),
            Spans::from("Esc on 'a' to cancel the changes to task or subtask"),
            Spans::from("'d' to delete a task (all subtasks too)"),
            Spans::from("'f' to fold a tasks containing subtasks"),
            Spans::from("'c' to mark/unmark a task (and all subtasks) as completed"),

        ];

        let groups_block = Block::default()
            .title("Controls and important info")
            .borders(Borders::ALL)
            .style(Style::default());


        let p = Paragraph::new(text).block(groups_block).alignment(Alignment::Center);

        f.render_widget(p, chunk[1]);
    }

    fn create_and_render_item_list<B: Backend>(_f: &mut Frame<B>, _app: &mut App, _chunk: &Vec<Rect>, _frame_size: &Rect) {  }

    fn create_and_render_edit_mode<B: Backend>(_f: &mut Frame<B>, _app: &mut App, _chunk: &Vec<Rect>) {  }
}