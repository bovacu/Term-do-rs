use std::ops::Add;
use crossterm::event::KeyCode;
use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, BorderType, List, ListItem, Paragraph};
use crate::{App, centered_rect, DataManager, FocusedLayout, LayoutState};

use crate::data_manager::TaskItem;

use unicode_width::UnicodeWidthStr;

pub struct TaskLayout {
    edit_mode: bool,
    is_adding_subtask: bool,
    input: String
}

impl TaskLayout {
    pub fn new() -> TaskLayout {
        TaskLayout {
            edit_mode: false,
            is_adding_subtask: false,
            input: String::new()
        }
    }

    pub fn recursive_sub_tasks<'a>(data_manager: &'a DataManager, tasks: &'a Vec<Box<TaskItem>>) -> Vec<ListItem<'a>> {
        let mut item_list : Vec<ListItem> = Vec::new();

        for i in 0..tasks.len() {
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
                let repeated = std::iter::repeat("     ").take(tasks[i].indentation - 1).collect::<String>().add("╚═══ ").add(iconed_line.as_str());
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
                let mut new_task_items = TaskLayout::recursive_sub_tasks(data_manager, tasks[i].get_tasks());
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

impl LayoutState for TaskLayout {

    fn is_in_edit_mode(&self) -> bool {
        return self.edit_mode;
    }

    fn handle_input(&mut self, data_manager: &mut DataManager, key_code: crossterm::event::KeyEvent) {
        if !self.edit_mode {
            match key_code.code {
                KeyCode::Char('a') => {
                    self.edit_mode = true;
                    self.is_adding_subtask = false;
                },
                KeyCode::Char('A') => {
                    self.edit_mode = true;
                    self.is_adding_subtask = true;
                },
                KeyCode::Char('d') => unsafe {
                    let selected_task = data_manager.selected_task;
                    let selected_group = data_manager.selected_group;
                    let gi = data_manager.get_group(selected_group);
                    gi.remove_task(selected_task);
                    data_manager.selected_task = 0;
                },
                KeyCode::Char('q') | KeyCode::Esc => {
                    if !self.edit_mode { return; }
                    self.edit_mode = false;
                },
                KeyCode::Char('c') => unsafe {
                    let selected_task = data_manager.selected_task;
                    let gi = data_manager.get_group(data_manager.selected_group);
                    gi.set_task_and_subtasks_done_or_undone(selected_task);

                    data_manager.save_state();
                },
                KeyCode::Up => {
                    if data_manager.selected_task > 0 {
                        data_manager.selected_task -= 1;
                    }
                },
                KeyCode::Down => {
                    let tasks = data_manager.get_group_items()[data_manager.selected_group].get_tasks_and_subtasks_count();
                    if data_manager.selected_task < tasks.0 - 1 {
                        data_manager.selected_task += 1;
                    }
                }
                _ => {}
            }
        } else {
            match key_code.code {
                KeyCode::Enter => unsafe {
                    let selected_task = data_manager.selected_task;
                    let gi = data_manager.get_group(data_manager.selected_group);
                    if !self.is_adding_subtask {
                        gi.add_task(self.input.drain(..).collect());
                    } else {
                        gi.add_subtask(self.input.drain(..).collect(), selected_task);
                    }
                    self.edit_mode = false;
                    data_manager.save_state();
                }
                KeyCode::Char(c) => {
                    self.input.push(c);
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Esc => {
                    self.edit_mode = false;
                },
                _ => {}
            }
        }
    }

    fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, lower_chunk: &Rect, chunk: &Vec<Rect>) {
        TaskLayout::create_and_render_base_block(f, app, chunk);
        TaskLayout::create_and_render_item_list(f, app, chunk);
        TaskLayout::create_and_render_edit_mode(f, app, chunk, lower_chunk);

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

    fn create_and_render_item_list<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>) {
        let area = centered_rect(95, 90, chunk[1]);

        let tasks = app.data_manager.get_group_items()[ app.data_manager.selected_group].get_tasks();
        let items_list = TaskLayout::recursive_sub_tasks(&app.data_manager, tasks);

        let items = List::new(items_list)
            .block(Block::default().borders(Borders::NONE));

        f.render_widget(items, area);
    }

    fn create_and_render_edit_mode<B: Backend>(f: &mut Frame<B>, app: &mut App, _chunk: &Vec<Rect>, lower_chunk: &Rect) {
        if app.task_layout.edit_mode {
            let options_block = Block::default().title("Add task").borders(Borders::ALL);
            let area = centered_rect(40, 10, *lower_chunk);

            let input = Paragraph::new(app.task_layout.input.as_ref())
                .style( if app.task_layout.edit_mode {
                    Style::default().add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                })
                .block(options_block);

            f.render_widget(input, area);


            if app.task_layout.edit_mode {
                f.set_cursor(
                    // Put cursor past the end of the input text
                    area.x + app.task_layout.input.width() as u16 + 1,
                    // Move one line down, from the border to the input line
                    area.y + 1,
                )
            }
        }
    }
}