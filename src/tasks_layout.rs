use std::collections::HashMap;
use std::ops::Add;
use crossterm::event::KeyEvent;
use tui::backend::Backend;
use tui::Frame;
use tui::layout::Rect;
use tui::style::{Modifier, Style};
use tui::widgets::{Block, Borders, BorderType, List, ListItem};
use crate::{App, centered_rect, DataManager, FocusedLayout, LayoutCommon, LayoutCommonTrait};

use crate::data_manager::{GroupItem, TaskItem};

use unicode_width::UnicodeWidthStr;
use crate::enums::InputMode;

pub struct TaskLayout {
    pub(crate) layout_common: LayoutCommon,
    is_adding_subtask: bool,
    width_of_chunk: usize
}

struct ParentInfo {
    needs_vertical_bar: bool,
    is_folded: bool
}

impl TaskLayout {
    pub fn new() -> TaskLayout {
        TaskLayout {
            layout_common: LayoutCommon::new(),
            is_adding_subtask: false,
            width_of_chunk: 0
        }
    }

    pub  fn recursive_sub_tasks<'a>(&self, data_manager: &'a DataManager, tasks: &'a Vec<TaskItem>, frame_size: &Rect) -> Vec<ListItem<'a>> {
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
                iconed_line = format!("{}{}", data_manager.config.task.get("icon_completed").unwrap(), "  ").to_string();
                default_style = default_style.fg(data_manager.config.get_color("task", "completed_color"));
            } else {
                default_style = default_style.fg(data_manager.config.get_color("task", "non_selected_color"));
                iconed_line = format!("{}{}", data_manager.config.task.get("icon_uncompleted").unwrap(), "  ").to_string();
            }

            iconed_line.push_str(line);

            if tasks[i].indentation > 1 {

                let mut top_parent = tasks[i].parent;
                let mut amount_of_fucking_vertical_sticks : HashMap<usize, ParentInfo> = HashMap::new();
                let selected_group = data_manager.selected_group;
                let gi = data_manager.get_group_read_only(selected_group);

                while top_parent != -1 {
                    let top_parent_task = GroupItem::get_task_recursive_read_only(top_parent as usize, gi.get_tasks()).unwrap();
                    for t in top_parent_task.0.get_tasks() {
                        if t.id > tasks[i].id {
                            amount_of_fucking_vertical_sticks.insert((*top_parent_task.0).indentation, ParentInfo { needs_vertical_bar: true, is_folded: top_parent_task.0.folded });
                            continue;
                        }
                        amount_of_fucking_vertical_sticks.insert((*top_parent_task.0).indentation, ParentInfo { needs_vertical_bar: false, is_folded: top_parent_task.0.folded });
                    }

                    top_parent = top_parent_task.0.parent;

                }

                let mut repeated = String::new();
                let indentation_string : String;

                let parent_folded = amount_of_fucking_vertical_sticks.iter().any(|e| e.1.is_folded);

                if !parent_folded {
                    for i in 0..tasks[i].indentation - 1 {
                        if amount_of_fucking_vertical_sticks.contains_key(&i) && !amount_of_fucking_vertical_sticks[&i].needs_vertical_bar {
                            repeated = repeated.add("     ");
                        } else {
                            repeated = repeated.add(format!("{}{}", data_manager.config.task.get("vertical_child_char_icon").unwrap(), "    ").as_str());
                        }
                    }

                    indentation_string = repeated.clone();

                    repeated = repeated.add(format!("{}{}{}", data_manager.config.task.get("turn_right_child_char_icon").unwrap(), std::iter::repeat(data_manager.config.task.get("horizontal_child_char_icon").unwrap()).take(3).collect::<String>(), " ").as_str()).add(iconed_line.as_str());

                    indented_line.push_str(repeated.as_str());

                    let sub_tasks_string = TaskLayout::sub_tasks_string(data_manager, tasks[i].get_tasks());
                    indented_line.push_str(sub_tasks_string.as_str());

                    indented_line = TaskLayout::break_line_if_needed(self, indented_line, &indentation_string);
                }
            } else {
                let mut folded = false;
                if tasks[i].parent >= 0 {
                    let selected_group = data_manager.selected_group;
                    let gi = data_manager.get_group_read_only(selected_group);
                    let top_parent_task = GroupItem::get_task_recursive_read_only(tasks[i].parent as usize, gi.get_tasks()).unwrap();
                    folded = top_parent_task.0.folded;
                }

                if !folded {
                    let l = format!("{}{}{}", data_manager.config.task.get("turn_right_child_char_icon").unwrap(), std::iter::repeat(data_manager.config.task.get("horizontal_child_char_icon").unwrap()).take(3).collect::<String>(), " ").to_string();
                    indented_line = std::iter::repeat(l).take(tasks[i].indentation).collect::<String>().add(iconed_line.as_str());
                    let sub_tasks_string = TaskLayout::sub_tasks_string(data_manager, tasks[i].get_tasks());
                    indented_line.push_str(sub_tasks_string.as_str());
                    indented_line = TaskLayout::break_line_if_needed(self, indented_line, "     ");
                }
            }

            if tasks[i].id == data_manager.selected_task {
                default_style = default_style.fg(data_manager.config.get_color("task", "selected_color"));
                item_list.push(ListItem::new(indented_line).style(default_style));
            } else {
                item_list.push(ListItem::new(indented_line).style(default_style));
            }

            if !tasks[i].get_tasks().is_empty() {
                let mut new_task_items = TaskLayout::recursive_sub_tasks(self, data_manager, tasks[i].get_tasks(), frame_size);
                item_list.append(&mut new_task_items);
            }
        }

        return item_list;
    }

    fn break_line_if_needed(&self, line: String, indentation: &str) -> String {
        if self.width_of_chunk == 0 {
            return line;
        }
        let number_of_breaks = line.len() / self.width_of_chunk;
        if number_of_breaks == 0 {
            return line;
        }

        let mut broke_line = String::new();
        let mut to_split = line.as_str();


        for _i in 0..number_of_breaks {
            let (left, right) = to_split.split_at(self.width_of_chunk);
            broke_line.push_str(left);
            broke_line.push('\n');
            broke_line.push_str(indentation);
            broke_line.push_str("        ");
            broke_line.push_str(right);
            to_split = right;
        }

        return broke_line;
    }

    fn sub_tasks_string(data_manager: &DataManager, tasks: &Vec<TaskItem>) -> String {
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
                if data_manager.config.get_key("add_task") == key_code.code {
                    self.layout_common.input_mode = InputMode::Add;
                    self.is_adding_subtask = false;
                    self.layout_common.input = String::new();
                    self.layout_common.cursor_pos = self.layout_common.input.width();

                    LayoutCommon::recalculate_input_string_starting_point(&mut self.layout_common);
                    DataManager::calculate_folded_hasmap(data_manager, data_manager.selected_task);
                } else if data_manager.config.get_key("add_subtask") == key_code.code {
                    if data_manager.get_group_items().is_empty() { return; }
                    if data_manager.get_group_items()[data_manager.selected_group].get_tasks().is_empty() { return; }

                    self.layout_common.input_mode = InputMode::Add;
                    self.is_adding_subtask = true;
                    self.layout_common.input = String::new();
                    self.layout_common.cursor_pos = self.layout_common.input.width();

                    LayoutCommon::recalculate_input_string_starting_point(&mut self.layout_common);
                } else if data_manager.config.get_key("edit_task_or_subtask") == key_code.code {
                    if data_manager.get_group_items().is_empty() { return; }
                    if data_manager.get_group_items()[data_manager.selected_group].get_tasks().is_empty() { return; }

                    self.layout_common.input_mode = InputMode::Edit;
                    self.is_adding_subtask = false;

                    let selected_task = data_manager.selected_task;
                    let selected_group = data_manager.selected_group;
                    let gi = data_manager.get_group(selected_group);
                    let task_name = GroupItem::get_task_recursive_read_only(selected_task, gi.get_tasks()).unwrap().0.name.clone();

                    self.layout_common.input = task_name;
                    self.layout_common.cursor_pos = self.layout_common.input.width();

                    LayoutCommon::recalculate_input_string_starting_point(&mut self.layout_common);
                } else if data_manager.config.get_key("delete_task_or_subtask") == key_code.code {
                    if data_manager.get_group_items().is_empty() { return; }
                    if data_manager.get_group_items()[data_manager.selected_group].get_tasks().is_empty() { return; }

                    data_manager.apply();

                    let selected_task = data_manager.selected_task;
                    let selected_group = data_manager.selected_group;
                    let parent_of_deleted : isize;
                    let task_ro : (TaskItem, isize);


                    let gi_ro = data_manager.get_group_read_only(selected_group);
                    let tasks = gi_ro.get_tasks();
                    let task = GroupItem::get_task_recursive_read_only(selected_task, tasks).unwrap();
                    task_ro = (task.0.clone(), task.1);
                    parent_of_deleted = task.0.parent;


                    let gi = data_manager.get_group(selected_group);
                    gi.remove_task((&task_ro.0, task_ro.1));

                    if parent_of_deleted != -1 {
                        gi.update_parents_to_check_if_all_completed(parent_of_deleted as usize);
                    }

                    if selected_task == gi.get_tasks_and_subtasks_count().0 && selected_task > 0 {
                        data_manager.selected_task -= 1;
                    }

                    if data_manager.folded_state.contains_key(&data_manager.selected_task) {
                        data_manager.folded_state.remove(&data_manager.selected_task);
                    }

                    data_manager.load_folding(data_manager.selected_group);
                    data_manager.save_state();
                } else if data_manager.config.get_key("complete_or_uncomplete_task") == key_code.code {
                    if data_manager.get_group_items().is_empty() { return; }
                    if data_manager.get_group_items()[data_manager.selected_group].get_tasks().is_empty() { return; }

                    data_manager.apply();

                    let selected_task = data_manager.selected_task;
                    let gi = data_manager.get_group(data_manager.selected_group);
                    gi.set_task_and_subtasks_done_or_undone(selected_task, None);

                    data_manager.save_state();
                } else if data_manager.config.get_key("fold_subtasks") == key_code.code {
                    if data_manager.get_group_items().is_empty() { return; }
                    if data_manager.get_group_items()[data_manager.selected_group].get_tasks().is_empty() { return; }

                    data_manager.apply();


                    let selected_task = data_manager.selected_task;
                    let selected_group = data_manager.selected_group;
                    let gi = data_manager.get_group(selected_group);

                    TaskItem::fold(GroupItem::get_task_recursive(selected_task, &mut gi.get_tasks_mut()).unwrap().0);
                    DataManager::calculate_folded_hasmap(data_manager, data_manager.selected_task);

                    data_manager.save_state();
                } else if data_manager.config.get_key("down_task_or_subtask") == key_code.code {
                    if data_manager.get_group_items().is_empty() { return; }
                    if data_manager.get_group_items()[data_manager.selected_group].get_tasks().is_empty() { return; }

                    let tasks = data_manager.get_group_items()[data_manager.selected_group].get_tasks_and_subtasks_count();
                    let selected_task = data_manager.selected_task;
                    let gi = data_manager.get_group_read_only(data_manager.selected_group);
                    let task = gi.get_tasks().last().unwrap();
                    let is_last_and_folded = task.id == selected_task && task.folded;

                    if data_manager.selected_task < tasks.0 - 1 && !is_last_and_folded {
                        for (_, entry) in data_manager.folded_state.iter() {
                            let next_task = data_manager.selected_task + 1;
                            if entry.contains(&next_task) {
                                if data_manager.selected_task + entry.len() < tasks.0 - 1 {
                                    data_manager.selected_task += entry.len();
                                }
                                break;
                            }
                        }

                        data_manager.selected_task += 1;
                    }
                } else if data_manager.config.get_key("up_task_or_subtask") == key_code.code {
                    if data_manager.get_group_items().is_empty() { return; }
                    if data_manager.get_group_items()[data_manager.selected_group].get_tasks().is_empty() { return; }

                    if data_manager.selected_task > 0 {

                        let mut biggest_group = 0;
                        for (_, entry) in data_manager.folded_state.iter() {
                            let previous_task = data_manager.selected_task - 1;
                            if entry.contains(&previous_task) && entry.len() > biggest_group {
                                biggest_group = entry.len();
                            }
                        }

                        data_manager.selected_task -= 1 + biggest_group;
                    }
                }
            },
            InputMode::Add => {
                if data_manager.config.get_key("apply_add_or_edit_task_or_subtask") == key_code.code {
                    data_manager.apply();

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
                } else {
                    <TaskLayout as LayoutCommonTrait>::poll_common_keys_input_mode(&key_code, &mut self.layout_common)
                }
            },
            InputMode::Edit => {
                if data_manager.config.get_key("apply_add_or_edit_task_or_subtask") == key_code.code {
                    data_manager.apply();

                    let selected_task = data_manager.selected_task;
                    let gi = data_manager.get_group(data_manager.selected_group);
                    gi.edit_sub_task(selected_task, self.layout_common.input.drain(..).collect());
                    self.layout_common.input_mode = InputMode::Navigate;
                    data_manager.save_state();
                } else {
                    <TaskLayout as LayoutCommonTrait>::poll_common_keys_input_mode(&key_code, &mut self.layout_common)
                }
            }
        }
    }

     fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>, frame_size: &Rect) {
        TaskLayout::create_and_render_base_block(f, app, chunk);
        TaskLayout::create_and_render_item_list(f, app, chunk, frame_size);
    }

    fn create_and_render_base_block<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>) {
        let mut tasks_block = Block::default()
            .title("Tasks")
            .borders(Borders::ALL)
            .style(Style::default().fg(app.data_manager.config.get_color("task", "border_color")));

        if app.focused_layout == FocusedLayout::TasksLayout {
            tasks_block = tasks_block.style(Style::default().add_modifier(Modifier::BOLD).fg(app.data_manager.config.get_color("task", "border_color")))
                .border_type(BorderType::Thick);
        }

        f.render_widget(tasks_block, chunk[1]);
    }

     fn create_and_render_item_list<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>, frame_size: &Rect) {
        let area = centered_rect(95, 90, chunk[1]);

        if app.data_manager.get_group_items().is_empty() {
            return;
        }

        let tasks = app.data_manager.get_group_items()[ app.data_manager.selected_group].get_tasks();
        let items_list = TaskLayout::recursive_sub_tasks(&app.task_layout, &app.data_manager, tasks, frame_size);

        let items = List::new(items_list)
            .block(Block::default().borders(Borders::NONE)).style(Style::default());

        f.render_widget(items, area);
    }

    fn create_and_render_edit_mode<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>) {
        let title : String = if app.task_layout.layout_common.input_mode == InputMode::Add {  if app.task_layout.is_adding_subtask { "Add subtask".to_string() } else { "Add task".to_string() } } else { "Edit task".to_string() };
        app.task_layout.width_of_chunk = chunk[1].width as usize;
        <TaskLayout as LayoutCommonTrait>::render_common_input_mode(f, &mut app.task_layout.layout_common, title.as_str(), chunk);
    }
}