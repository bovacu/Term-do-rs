use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;

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





#[derive(Serialize, Deserialize, Clone)]
pub struct TaskItem {
    pub(crate) id: usize,
    pub(crate) done: bool,
    pub(crate) name: String,
    pub(crate) indentation: usize,
    pub(crate) parent: isize,
    pub(crate) tasks: Vec<TaskItem>,
    pub(crate) folded: bool
}

impl TaskItem {
    pub fn new(task_name: String, id_value: usize, parent_id: isize) -> TaskItem {
        TaskItem {
            id: id_value,
            done: false,
            name: task_name,
            indentation: 0,
            tasks: Vec::new(),
            parent: parent_id,
            folded: false
        }
    }

    pub fn get_tasks(&self) -> &Vec<TaskItem> {
        return &self.tasks;
    }

    pub fn are_all_sub_tasks_done(&self, task: &Vec<TaskItem>) -> bool {
        let mut completed = true;
        if self.tasks.is_empty() {
            completed = self.done;
        }

        for i in 0..task.len() {
            completed &= task[i].done;
            completed &= TaskItem::are_all_sub_tasks_done(self, &task[i].tasks);
        }

        return completed;
    }

    pub fn fold(&mut self) {
        if self.tasks.is_empty() {
            return;
        }

        self.folded = !self.folded;
    }
}





#[derive(Serialize, Deserialize)]
pub struct GroupItem {
    id: usize,
    pub name: String,
    tasks: Vec<TaskItem>
}

impl GroupItem {
    pub fn new(data_manager: &DataManager) -> GroupItem {
        GroupItem {
            id: data_manager.get_group_items().len(),
            name: String::new(),
            tasks: Vec::new()
        }
    }

    pub fn get_tasks(&self) -> &Vec<TaskItem> {
        return &self.tasks;
    }

    pub fn get_tasks_mut(&mut self) -> &mut Vec<TaskItem> {
        return &mut self.tasks;
    }

    pub fn add_task(&mut self, task_name: String) {
        let task = TaskItem::new(task_name, GroupItem::get_tasks_and_subtasks_count(self).0, -1);
        self.tasks.push(task);
    }

    pub  fn remove_task(&mut self, task: (&TaskItem, isize)) -> usize {
        let amount_removed: usize;
        if task.0.parent != -1 {
            let parent = GroupItem::get_task_recursive(task.0.parent as usize, &mut self.tasks).unwrap();
            amount_removed = GroupItem::get_tasks_and_subtasks_count_recursive(&task.0.get_tasks()).0 + 1;
            parent.0.tasks.remove(task.1 as usize);
        } else {
            amount_removed = GroupItem::get_tasks_and_subtasks_count_recursive(&task.0.get_tasks()).0 + 1;
            self.tasks.remove(task.1 as usize);
        }

        GroupItem::recalculate_tasks_ids_on_remove(&mut self.tasks,task.0.id, amount_removed);

        return amount_removed;
    }

    pub  fn add_subtask(&mut self, task_name: String, parent_id: usize) -> usize {
        let parent_task = GroupItem::get_task_recursive(parent_id, &mut self.tasks).unwrap();
        let mut new_task = TaskItem::new(task_name, parent_id + GroupItem::get_tasks_and_subtasks_count_recursive(&parent_task.0.tasks).0 + 1, parent_id as isize);
        new_task.indentation = parent_task.0.indentation + 1;
        let new_id = new_task.id;
        parent_task.0.tasks.push(new_task);

        GroupItem::recalculate_tasks_ids_on_add(parent_id, new_id, &mut self.tasks, false);
        GroupItem::recalculate_parent_tasks_ids_on_add(0, &mut self.tasks);

        return new_id;
    }

    pub  fn edit_sub_task(&mut self, task_id: usize, new_text: String) {
        let task = GroupItem::get_task_recursive(task_id, &mut self.tasks).unwrap();
        task.0.name = new_text;
    }

    pub fn get_tasks_and_subtasks_count(&self) -> (usize, usize) {
        return GroupItem::get_tasks_and_subtasks_count_recursive(&self.tasks);
    }

    pub fn get_tasks_and_subtasks_count_specific(&self, tasks: &Vec<TaskItem>) -> (usize, usize) {
        return GroupItem::get_tasks_and_subtasks_count_recursive(tasks);
    }

    pub  fn set_task_and_subtasks_done_or_undone(&mut self, task_id: usize, completed: Option<bool>) {
        let parent_task = GroupItem::get_task_recursive(task_id, &mut self.tasks).unwrap();
        if completed.is_some() {
            parent_task.0.done = completed.unwrap();
        } else {
            parent_task.0.done = !parent_task.0.done;
        }
        GroupItem::set_task_completed_recursive(parent_task.0.done, &mut parent_task.0.tasks);

        let mut top_parent = parent_task.0.parent;
        let mut all_done = true;
        while top_parent != -1 {
            let top_parent_task = GroupItem::get_task_recursive(top_parent as usize, &mut self.tasks).unwrap();
            all_done &= top_parent_task.0.are_all_sub_tasks_done(top_parent_task.0.get_tasks());
            top_parent_task.0.done = all_done;
            top_parent = top_parent_task.0.parent;
        }
    }

    pub  fn update_parents_to_check_if_all_completed(&mut self, task_id: usize) {
        let parent_task = GroupItem::get_task_recursive(task_id, &mut self.tasks).unwrap();
        let mut all_done = true;
        all_done &= parent_task.0.are_all_sub_tasks_done(parent_task.0.get_tasks());
        parent_task.0.done = all_done;
        let mut top_parent = parent_task.0.parent;

        while top_parent != -1 {
            let top_parent_task = GroupItem::get_task_recursive(top_parent as usize, &mut self.tasks).unwrap();
            all_done &= top_parent_task.0.are_all_sub_tasks_done(top_parent_task.0.get_tasks());
            top_parent_task.0.done = all_done;
            top_parent = top_parent_task.0.parent;
        }
    }



    pub fn get_task_recursive(task_id: usize, tasks: &mut [TaskItem], ) -> Result<(&mut TaskItem, isize), bool> {
        for i in 0..tasks.len() {
            let task = tasks[i].id;
            if task == task_id {
                let result = (&mut tasks[i], i as isize);
                return Ok(result);
            }
        }

        for task in tasks {
            if let Ok(res) = GroupItem::get_task_recursive(task_id, &mut task.tasks) {
                return Ok(res);
            }
        }

        Err(true)
    }

    pub fn get_task_recursive_read_only(task_id: usize, tasks: &Vec<TaskItem>) -> Result<(&TaskItem, isize), bool> {
        for i in 0..tasks.len() {
            let task = tasks[i].id;
            if task == task_id {
                let result = (&tasks[i], i as isize);
                return Ok(result);
            }
        }

        for task in tasks {
            if let Ok(res) = GroupItem::get_task_recursive_read_only(task_id, &task.tasks) {
                return Ok(res);
            }
        }

        Err(true)
    }

    fn set_task_completed_recursive(completed: bool, tasks: &mut Vec<TaskItem>) {
        for i in 0..tasks.len() {
            tasks[i].done = completed;
            GroupItem::set_task_completed_recursive(completed, &mut tasks[i].tasks);
        }
    }

    fn recalculate_tasks_ids_on_add(parent_id: usize, new_id: usize, tasks: &mut Vec<TaskItem>, found: bool) {
        for i in 0..tasks.len() {
            if tasks[i].id >= new_id  {
                tasks[i].id += 1;
            }

            if tasks[i].id != parent_id {
                GroupItem::recalculate_tasks_ids_on_add(parent_id, new_id, &mut tasks[i].tasks, found);
            }
        }
    }

    fn recalculate_parent_tasks_ids_on_add(parent_id: usize, tasks: &mut Vec<TaskItem>) {
        for i in 0..tasks.len() {
            GroupItem::recalculate_parent_tasks_ids_on_add(tasks[i].id, &mut tasks[i].tasks);
            if tasks[i].parent != -1 {
                tasks[i].parent = parent_id as isize;
            }
        }
    }

    fn recalculate_tasks_ids_on_remove(tasks: &mut Vec<TaskItem>, removed_id: usize, amount_removed: usize) {
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

    fn get_tasks_and_subtasks_count_recursive(tasks: &Vec<TaskItem>) -> (usize, usize) {
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
    pub folded_state: HashMap<usize, HashSet<usize>>,
    pub selected_group: usize,
    pub selected_task: usize
}

impl DataManager {
    pub fn new() -> DataManager {
        DataManager {
            groups: Vec::new(),
            selected_group: 0,
            selected_task: 0,
            folded_state: HashMap::new(),
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
                self.folded_state.clear();

                self.load_folding(self.groups[self.selected_group].id);
            }
        }
    }

    pub fn save_state(&mut self) {
        if !DataManager::check_data_integrity(self) {
            eprintln!("Data integrity has been compromised! No serialization is being applied...");
            return;
        }
        let full_json = serde_json::to_string_pretty(self).expect("Couldn't serialized");
        fs::write("data/data.json", full_json).expect("Couldn't write to data file");
    }

    pub fn check_data_integrity(&self) -> bool {
        let mut integrity_ok = true;

        for group in &self.groups {
            if group.tasks.is_empty() {
                continue;
            }

            integrity_ok &= DataManager::check_data_integrity_recursive(&group.tasks, 0).0;
        }

        return integrity_ok;
    }

    fn check_data_integrity_recursive(tasks: &Vec<TaskItem>, mut id: usize) -> (bool, usize) {
        let mut ok = true;

        for task in tasks {
            if id != task.id {
                ok &= false;
            }

            id += 1;

            let (new_ok, new_id) = DataManager::check_data_integrity_recursive(&task.tasks, id);
            id = new_id;
            ok &= new_ok;
        }

        return (ok, id);
    }

    pub fn load_folding(&mut self, group_id: usize) {
        self.folded_state.clear();
        let tasks = self.groups[group_id].tasks.clone();
        DataManager::load_folding_recursive(self, &tasks);
    }

    fn load_folding_recursive(&mut self, tasks: &Vec<TaskItem>) {
        for task in tasks {
            if task.folded {
                DataManager::calculate_folded_hasmap(self, task.id);
            }
            DataManager::load_folding_recursive(self, &task.tasks);
        }
    }

    pub fn calculate_folded_hasmap(&mut self, selected_task: usize) {
        let selected_group = self.selected_group;
        let gi = self.get_group_read_only(selected_group);
        if gi.tasks.is_empty() {
            return;
        }

        let task = GroupItem::get_task_recursive_read_only(selected_task, gi.get_tasks()).unwrap();
        if task.0.folded {
            let elements_to_skip = gi.get_tasks_and_subtasks_count_specific(task.0.get_tasks()).0;

            let mut folded_state_entry : HashSet<usize> = HashSet::new();
            for i in 0..elements_to_skip {
                folded_state_entry.insert(selected_task + i + 1);
            }
            self.folded_state.insert(selected_task, folded_state_entry);
        } else {
            self.folded_state.remove(&selected_task);
        }
    }
}