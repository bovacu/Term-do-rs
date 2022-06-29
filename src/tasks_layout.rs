use std::collections::HashMap;
use std::ops::Add;
use crossterm::event::{KeyCode, KeyEvent};
use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, BorderType, List, ListItem};
use crate::{App, centered_rect, DataManager, FocusedLayout, LayoutCommon, LayoutCommonTrait};

use crate::data_manager::TaskItem;

use unicode_width::UnicodeWidthStr;
use crate::enums::InputMode;

pub struct TaskLayout {
    pub(crate) layout_common: LayoutCommon,
    is_adding_subtask: bool,
}

impl TaskLayout {
    pub fn new() -> TaskLayout {
        TaskLayout {
            layout_common: LayoutCommon::new(),
            is_adding_subtask: false
        }
    }

    pub unsafe fn recursive_sub_tasks<'a>(data_manager: &'a DataManager, tasks: &'a Vec<Box<TaskItem>>, frame_size: &Rect) -> Vec<ListItem<'a>> {
        let mut item_list : Vec<ListItem> = Vec::new();
        let height = ListItem::new("Hello").style(Style::default()).height();
        let max_lines : usize = (frame_size.height as usize / (2 * height)) as usize;
        let showing_start_item = if data_manager.selected_task > max_lines { data_manager.selected_task - max_lines } else { 0 };

        for i in showing_start_item..tasks.len() {
            let line = tasks[i].name.as_str();
            let mut indented_line = String::new();

            let mut iconed_line : String;
            let mut default_style = Style::default().remove_modifier(Modifier::BOLD);

            if tasks[i].done {
                iconed_line = "  ".to_string();
                default_style = default_style.fg(Color::LightGreen);
            } else {
                iconed_line = "  ".to_string();
            }

            iconed_line.push_str(line);

            if tasks[i].indentation > 1 {

                let mut top_parent = tasks[i].parent;
                let mut amount_of_fucking_vertical_sticks : HashMap<usize, usize> = HashMap::new();
                let selected_group = data_manager.selected_group;
                let gi = data_manager.get_group_read_only(selected_group);

                while top_parent != -1 {
                    let top_parent_task = gi.get_task(top_parent as usize);

                    for t in (*top_parent_task.0).get_tasks() {
                        if t.id > tasks[i].id {
                            amount_of_fucking_vertical_sticks.insert((*top_parent_task.0).indentation, 1);
                            continue;
                        }
                        amount_of_fucking_vertical_sticks.insert((*top_parent_task.0).indentation, 0);
                    }

                    top_parent = (*top_parent_task.0).parent;
                }

                let mut repeated = String::new();

                for i in 0..tasks[i].indentation - 1 {
                    if amount_of_fucking_vertical_sticks[&i] == 1 {
                        repeated = repeated.add("║    ");
                    } else {
                        repeated = repeated.add("     ");
                    }
                }

                repeated = repeated.add("╚═══ ").add(iconed_line.as_str());
                indented_line.push_str(repeated.as_str());
                let sub_tasks_string = TaskLayout::sub_tasks_string(data_manager, tasks[i].get_tasks());
                indented_line.push_str(sub_tasks_string.as_str());
            } else {
                indented_line = std::iter::repeat("╚═══ ").take(tasks[i].indentation).collect::<String>().add(iconed_line.as_str());
                let sub_tasks_string = TaskLayout::sub_tasks_string(data_manager, tasks[i].get_tasks());
                indented_line.push_str(sub_tasks_string.as_str());
            }

            if tasks[i].id == data_manager.selected_task {
                default_style = default_style.fg(Color::Yellow);
                item_list.push(ListItem::new(indented_line).style(default_style));
            } else {
                item_list.push(ListItem::new(indented_line).style(default_style));
            }

            if !tasks[i].get_tasks().is_empty() {
                let mut new_task_items = TaskLayout::recursive_sub_tasks(data_manager, tasks[i].get_tasks(), frame_size);
                item_list.append(&mut new_task_items);
            }
        }

        return item_list;
    }

    fn sub_tasks_string(data_manager: &DataManager, tasks: &Vec<Box<TaskItem>>) -> String {
        let selected_group = data_manager.selected_group;
        let gi = data_manager.get_group_read_only(selected_group);
        let sub_tasks_count = gi.get_tasks_and_subtasks_count_specific(tasks);

        if sub_tasks_count.0 == 0 {
            return String::new();
        }

        return format!(" ({}/{})", sub_tasks_count.1, sub_tasks_count.0);
    }
}







impl LayoutCommonTrait for TaskLayout {

    fn handle_input(&mut self, data_manager: &mut DataManager, key_code: KeyEvent) {
        match self.layout_common.input_mode {
            InputMode::Navigate => {
                match key_code.code {
                    KeyCode::Char('a') => {
                        self.layout_common.input_mode = InputMode::Add;
                        self.is_adding_subtask = false;
                        self.layout_common.input = String::new();
                        self.layout_common.cursor_pos = self.layout_common.input.width();

                        LayoutCommon::recalculate_input_string_starting_point(&mut self.layout_common);
                    },
                    KeyCode::Char('A') => {
                        if data_manager.get_group_items().is_empty() { return; }
                        self.layout_common.input_mode = InputMode::Add;
                        self.is_adding_subtask = true;
                        self.layout_common.input = String::new();
                        self.layout_common.cursor_pos = self.layout_common.input.width();

                        LayoutCommon::recalculate_input_string_starting_point(&mut self.layout_common);
                    },
                    KeyCode::Char('e') => unsafe {
                        self.layout_common.input_mode = InputMode::Edit;
                        self.is_adding_subtask = false;

                        let selected_task = data_manager.selected_task;
                        let selected_group = data_manager.selected_group;
                        let gi = data_manager.get_group(selected_group);
                        let task_name = (*gi.get_task(selected_task).0).name.clone();

                        self.layout_common.input = task_name;
                        self.layout_common.cursor_pos = self.layout_common.input.width();

                        LayoutCommon::recalculate_input_string_starting_point(&mut self.layout_common);
                    },
                    KeyCode::Char('d') => unsafe {
                        if data_manager.get_group_items().is_empty() { return; }
                        let selected_task = data_manager.selected_task;
                        let selected_group = data_manager.selected_group;
                        let gi = data_manager.get_group(selected_group);
                        let parent_of_deleted = (*gi.get_task(selected_task).0).parent;
                        gi.remove_task(selected_task);

                        if parent_of_deleted != -1 {
                            let gi = data_manager.get_group(data_manager.selected_group);
                            gi.set_task_and_subtasks_done_or_undone(parent_of_deleted as usize, None);
                        }

                        data_manager.selected_task = 0;
                        data_manager.save_state();
                    },
                    KeyCode::Esc => {
                        if self.layout_common.input_mode != InputMode::Navigate { return; }
                        self.layout_common.input_mode = InputMode::Navigate;
                    },
                    KeyCode::Char('c') => unsafe {
                        if data_manager.get_group_items().is_empty() { return; }
                        let selected_task = data_manager.selected_task;
                        let gi = data_manager.get_group(data_manager.selected_group);
                        gi.set_task_and_subtasks_done_or_undone(selected_task, None);

                        data_manager.save_state();
                    },
                    KeyCode::Up => {
                        if data_manager.get_group_items().is_empty() { return; }
                        if data_manager.selected_task > 0 {
                            data_manager.selected_task -= 1;
                        }
                    },
                    KeyCode::Down => {
                        if data_manager.get_group_items().is_empty() { return; }
                        let tasks = data_manager.get_group_items()[data_manager.selected_group].get_tasks_and_subtasks_count();
                        if data_manager.selected_task < tasks.0 - 1 {
                            data_manager.selected_task += 1;
                        }
                    }
                    _ => {}
                }
            },
            InputMode::Add => {
                match key_code.code {
                    KeyCode::Enter => unsafe {
                        let selected_task = data_manager.selected_task;
                        let gi = data_manager.get_group(data_manager.selected_group);
                        if !self.is_adding_subtask {
                            gi.add_task(self.layout_common.input.drain(..).collect());
                        } else {
                            let new_id = gi.add_subtask(self.layout_common.input.drain(..).collect(), selected_task);

                            let selected_task = new_id;
                            let gi = data_manager.get_group(data_manager.selected_group);
                            gi.set_task_and_subtasks_done_or_undone(selected_task, Some(false));
                        }

                        self.layout_common.input_mode = InputMode::Navigate;
                        data_manager.save_state();
                    },
                    _ => {
                        <TaskLayout as LayoutCommonTrait>::poll_common_keys_input_mode(&key_code, &mut self.layout_common)
                    }
                }
            },
            InputMode::Edit => {
                match key_code.code {
                    KeyCode::Enter => unsafe {
                        let selected_task = data_manager.selected_task;
                        let gi = data_manager.get_group(data_manager.selected_group);
                        gi.edit_sub_task(selected_task, self.layout_common.input.drain(..).collect());
                        self.layout_common.input_mode = InputMode::Navigate;
                        data_manager.save_state();
                    },
                    _ => {
                        <TaskLayout as LayoutCommonTrait>::poll_common_keys_input_mode(&key_code, &mut self.layout_common)
                    }
                }
            }
        }
    }

    unsafe fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>, frame_size: &Rect) {
        TaskLayout::create_and_render_base_block(f, app, chunk);
        TaskLayout::create_and_render_item_list(f, app, chunk, frame_size);
    }

    fn create_and_render_base_block<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>) {
        let mut tasks_block = Block::default()
            .title("Tasks")
            .borders(Borders::ALL)
            .style(Style::default());

        if app.focused_layout == FocusedLayout::TasksLayout {
            tasks_block = tasks_block.style(Style::default().add_modifier(Modifier::BOLD))
                .border_type(BorderType::Thick);
        }

        f.render_widget(tasks_block, chunk[1]);
    }

    unsafe fn create_and_render_item_list<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>, frame_size: &Rect) {
        let area = centered_rect(95, 90, chunk[1]);

        if app.data_manager.get_group_items().is_empty() {
            return;
        }

        let tasks = app.data_manager.get_group_items()[ app.data_manager.selected_group].get_tasks();
        let items_list = TaskLayout::recursive_sub_tasks(&app.data_manager, tasks, frame_size);

        let items = List::new(items_list)
            .block(Block::default().borders(Borders::NONE));

        f.render_widget(items, area);
    }

    fn create_and_render_edit_mode<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>) {
        let title : String = if app.task_layout.layout_common.input_mode == InputMode::Add {  if app.task_layout.is_adding_subtask { "Add subtask".to_string() } else { "Add task".to_string() } } else { "Edit task".to_string() };
        <TaskLayout as LayoutCommonTrait>::render_common_input_mode(f, &mut app.task_layout.layout_common, title.as_str(), chunk);
    }
}