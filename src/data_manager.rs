use std::fs;
use std::fs::File;
use std::ptr::null_mut;

use serde::{Deserialize, Serialize};
use crate::enums::InputMode;

use unicode_width::UnicodeWidthStr;

pub struct LayoutCommon {
    pub(crate) input_mode: InputMode,
    pub(crate) input: String,
    pub(crate) cursor_pos: usize,
    pub(crate) starting_rendering_input_point: usize,
    pub(crate) max_string_width: usize
}

impl LayoutCommon {
    pub fn new() -> LayoutCommon {
        LayoutCommon {
            input_mode: InputMode::Navigate,
            input: String::new(),
            cursor_pos: 0,
            starting_rendering_input_point: 0,
            max_string_width: 0
        }
    }

    pub fn recalculate_input_string_starting_point(layout_common: &mut LayoutCommon) {
        if layout_common.input.width() > layout_common.max_string_width {
            layout_common.starting_rendering_input_point = layout_common.input.width() - layout_common.max_string_width;
        } else {
            layout_common.starting_rendering_input_point = 0;
        }
    }

    pub fn is_in_edit_mode(&self) -> bool {
        return self.input_mode == InputMode::Add || self.input_mode == InputMode::Edit;
    }
}





#[derive(Serialize, Deserialize)]
pub struct TaskItem {
    pub(crate) id: usize,
    pub(crate) done: bool,
    pub(crate) name: String,
    pub(crate) indentation: usize,
    pub(crate) parent: isize,
    tasks: Vec<Box<TaskItem>>
}

impl TaskItem {
    pub fn new(task_name: String, id_value: usize, parent_id: isize) -> TaskItem {
        TaskItem {
            id: id_value,
            done: false,
            name: task_name,
            indentation: 0,
            tasks: Vec::new(),
            parent: parent_id
        }
    }

    pub fn get_tasks(&self) -> &Vec<Box<TaskItem>> {
        return &self.tasks;
    }

    pub fn are_all_sub_tasks_done(&self, task: &Vec<Box<TaskItem>>) -> bool {
        let mut completed = true;
        for i in 0..task.len() {
            completed &= task[i].done;
            completed &= TaskItem::are_all_sub_tasks_done(self, &task[i].tasks);
        }

        return completed;
    }
}





#[derive(Serialize, Deserialize)]
pub struct GroupItem {
    id: usize,
    pub name: String,
    tasks: Vec<Box<TaskItem>>
}

impl GroupItem {
    pub fn new(data_manager: &DataManager) -> GroupItem {
        GroupItem {
            id: data_manager.get_group_items().len(),
            name: String::new(),
            tasks: Vec::new()
        }
    }

    pub fn get_tasks(&self) -> &Vec<Box<TaskItem>> {
        return &self.tasks;
    }

    pub fn add_task(&mut self, task_name: String) {
        let task = TaskItem::new(task_name, GroupItem::get_tasks_and_subtasks_count(self).0, -1);
        self.tasks.push(Box::new(task));
    }

    pub unsafe fn remove_task(&mut self, task_id: usize) -> usize {
        let task = GroupItem::get_task_recursive(task_id, &mut self.tasks);

        let amount_removed : usize;
        if (*task.0).parent != -1 {
            let parent = GroupItem::get_task_recursive((*task.0).parent as usize, &mut self.tasks);
            amount_removed = GroupItem::get_tasks_and_subtasks_count_recursive(&(*task.0).get_tasks()).0 + 1;
            (*parent.0).tasks.remove(task.1 as usize);
        } else {
            amount_removed = GroupItem::get_tasks_and_subtasks_count_recursive(&(*task.0).get_tasks()).0 + 1;
            self.tasks.remove(task.1 as usize);
        }

        GroupItem::recalculate_tasks_ids_on_remove(&mut self.tasks,task_id, amount_removed);

        return amount_removed;
    }

    pub fn get_task(&mut self, task_id: usize) -> (*mut Box<TaskItem>, isize) {
        return GroupItem::get_task_recursive(task_id, &mut self.tasks);
    }

    pub unsafe fn add_subtask(&mut self, task_name: String, parent_id: usize) -> usize {
        let parent_task = GroupItem::get_task_recursive(parent_id, &mut self.tasks);
        let mut new_task = Box::new(TaskItem::new(task_name, parent_id + GroupItem::get_tasks_and_subtasks_count_recursive(&(*parent_task.0).tasks).0 + 1, parent_id as isize));
        new_task.indentation = (*parent_task.0).indentation + 1;
        let new_id = new_task.id;
        (*parent_task.0).tasks.push(new_task);

        GroupItem::recalculate_tasks_ids_on_add(parent_id, new_id, &mut self.tasks);

        return new_id;
    }

    pub unsafe fn edit_sub_task(&mut self, task_id: usize, new_text: String) {
        let task = GroupItem::get_task_recursive(task_id, &mut self.tasks);
        (*task.0).name = new_text;
    }

    pub fn get_tasks_and_subtasks_count(&self) -> (usize, usize) {
        return GroupItem::get_tasks_and_subtasks_count_recursive(&self.tasks);
    }

    pub fn get_tasks_and_subtasks_count_specific(&self, tasks: &Vec<Box<TaskItem>>) -> (usize, usize) {
        return GroupItem::get_tasks_and_subtasks_count_recursive(tasks);
    }

    pub unsafe fn set_task_and_subtasks_done_or_undone(&mut self, task_id: usize, completed: Option<bool>) {
        let parent_task = GroupItem::get_task_recursive(task_id, &mut self.tasks);
        if completed.is_some() {
            (*parent_task.0).done = completed.unwrap();
        } else {
            (*parent_task.0).done = !(*parent_task.0).done;
        }
        GroupItem::set_task_completed_recursive((*parent_task.0).done, &mut (*parent_task.0).tasks);

        let mut top_parent = (*parent_task.0).parent;
        let mut all_done = true;
        while top_parent != -1 {
            let top_parent_task = GroupItem::get_task_recursive(top_parent as usize, &mut self.tasks);
            all_done &= (*top_parent_task.0).are_all_sub_tasks_done((*top_parent_task.0).get_tasks());
            (*top_parent_task.0).done = all_done;
            top_parent = (*top_parent_task.0).parent;
        }
    }

    fn get_task_recursive(task_id: usize, tasks: &mut Vec<Box<TaskItem>>) -> (*mut Box<TaskItem>, isize) {
        for i in 0..tasks.len() {
            let task = tasks[i].id;
            if task == task_id {
                return (&mut tasks[i], i as isize);
            }
        }

        for i in 0..tasks.len() {
            let result = GroupItem::get_task_recursive(task_id, &mut tasks[i].tasks);
            if result.0 == null_mut() {
                continue;
            }
            return result;
        }

        return (null_mut(), -1);
    }

    fn set_task_completed_recursive(completed: bool, tasks: &mut Vec<Box<TaskItem>>) {
        for i in 0..tasks.len() {
            tasks[i].done = completed;
            GroupItem::set_task_completed_recursive(completed, &mut tasks[i].tasks);
        }
    }

    fn recalculate_tasks_ids_on_add(parent_id: usize, new_id: usize, tasks: &mut Vec<Box<TaskItem>>) {
        for i in 0..tasks.len() {
            if tasks[i].id >= new_id  {
                tasks[i].id += 1;
            }

            if tasks[i].id != parent_id {
                GroupItem::recalculate_tasks_ids_on_add(parent_id, new_id, &mut tasks[i].tasks);
            }
        }
    }

    fn recalculate_tasks_ids_on_remove(tasks: &mut Vec<Box<TaskItem>>, removed_id: usize, amount_removed: usize) {
        for i in 0..tasks.len() {
            if tasks[i].id > removed_id  {
                tasks[i].id -= amount_removed;
                if tasks[i].parent > removed_id as isize {
                    tasks[i].parent -= amount_removed as isize;
                }
            }

            GroupItem::recalculate_tasks_ids_on_remove(&mut tasks[i].tasks, removed_id, amount_removed);
        }
    }

    fn get_tasks_and_subtasks_count_recursive(tasks: &Vec<Box<TaskItem>>) -> (usize, usize) {
        let mut count = 0;
        let mut completed = 0;
        for task in tasks {

            if task.done {
                completed += 1;
            }

            if !task.tasks.is_empty() {
                let result = GroupItem::get_tasks_and_subtasks_count_recursive(task.get_tasks());
                count += result.0;
                completed += result.1;
            }
            count += 1;
        }

        return (count, completed);
    }


}





#[derive(Serialize, Deserialize)]
pub struct DataManager {
    groups: Vec<GroupItem>,
    pub selected_group: usize,
    pub selected_task: usize
}

impl DataManager {
    pub fn new() -> DataManager {
        DataManager {
            groups: Vec::new(),
            selected_group: 0,
            selected_task: 0
        }
    }

    pub fn add_group_item(&mut self, group_item: GroupItem) {
        self.groups.push(group_item);
    }

    pub fn edit_group_item(&mut self, group_id: usize, new_text: String) {
        self.groups[group_id].name = new_text;
    }

    pub fn delete_group_item(&mut self, group_id: usize) {
        self.groups.remove(group_id);
    }

    pub fn get_group_items(&self) -> &Vec<GroupItem> {
        return &self.groups;
    }

    pub fn get_group(&mut self, id: usize) -> &mut GroupItem {
        return &mut self.groups[id];
    }

    pub fn get_group_read_only(&self, id: usize) -> &GroupItem {
        return &self.groups[id];
    }

    pub fn load_state(&mut self) {
        let read_file = fs::read_to_string("data/data.json");
        match read_file {
            Err(_error) => {
                fs::create_dir("data").expect("Couldn't create dir 'data'");
                File::create("data/data.json").expect("Couldn't create file data/data.json");
                let base_data_manager = DataManager::new();
                let full_json = serde_json::to_string_pretty(&base_data_manager).expect("Couldn't serialized");
                fs::write("data/data.json", full_json).expect("Couldn't write to data file");
            },
            Ok(file) => {
                let full_json : DataManager = serde_json::from_str(&file).unwrap();
                self.groups = full_json.groups;
            }
        }
    }

    pub fn save_state(&mut self) {
        let full_json = serde_json::to_string_pretty(self).expect("Couldn't serialized");
        fs::write("data/data.json", full_json).expect("Couldn't write to data file");
    }
}