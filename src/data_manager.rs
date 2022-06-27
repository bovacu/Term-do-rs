use std::fs;
use std::ptr::null_mut;

use serde::{Deserialize, Serialize};


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

    // pub fn get_task(&mut self, task_id: usize) -> *mut Box<TaskItem> {
    //     return GroupItem::get_task_recursive(task_id, &mut self.tasks);
    // }

    pub fn add_task(&mut self, task_name: String) {
        let task = TaskItem::new(task_name, GroupItem::get_tasks_and_subtasks_count(self).0, -1);
        self.tasks.push(Box::new(task));
    }

    pub unsafe fn add_subtask(&mut self, task_name: String, parent_id: usize) {
        let parent_task = GroupItem::get_task_recursive(parent_id, &mut self.tasks);
        let mut new_task = Box::new(TaskItem::new(task_name, parent_id + (*parent_task).tasks.len() + 1, parent_id as isize));
        new_task.indentation = (*parent_task).indentation + 1;
        let new_id = new_task.id;
        (*parent_task).tasks.push(new_task);

        GroupItem::recalculate_tasks_ids(parent_id, new_id,  &mut self.tasks);
    }

    pub fn get_tasks_and_subtasks_count(&self) -> (usize, usize) {
        return GroupItem::get_tasks_and_subtasks_count_recursive(&self.tasks);
    }

    pub fn get_tasks_and_subtasks_count_specific(&self, tasks: &Vec<Box<TaskItem>>) -> (usize, usize) {
        return GroupItem::get_tasks_and_subtasks_count_recursive(tasks);
    }

    pub unsafe fn set_task_and_subtasks_done_or_undone(&mut self, task_id: usize) {
        let parent_task = GroupItem::get_task_recursive(task_id, &mut self.tasks);
        (*parent_task).done = !(*parent_task).done;
        GroupItem::set_task_completed_recursive((*parent_task).done, &mut (*parent_task).tasks);

        let mut top_parent = (*parent_task).parent;
        let mut all_done = true;
        while top_parent != -1 {
            let top_parent_task = GroupItem::get_task_recursive(top_parent as usize, &mut self.tasks);
            all_done &= (*top_parent_task).are_all_sub_tasks_done((*top_parent_task).get_tasks());
            (*top_parent_task).done = all_done;
            top_parent = (*top_parent_task).parent;
        }
    }

    fn get_task_recursive(task_id: usize, tasks: &mut Vec<Box<TaskItem>>) -> *mut Box<TaskItem> {
        for i in 0..tasks.len() {
            let task = tasks[i].id;
            if task == task_id {
                return &mut tasks[i];
            }
        }

        for i in 0..tasks.len() {
            let result = GroupItem::get_task_recursive(task_id, &mut tasks[i].tasks);
            if result == null_mut() {
                continue;
            }
            return result;
        }

        return null_mut();
    }

    fn set_task_completed_recursive(completed: bool, tasks: &mut Vec<Box<TaskItem>>) {
        for i in 0..tasks.len() {
            tasks[i].done = completed;
            GroupItem::set_task_completed_recursive(completed, &mut tasks[i].tasks);
        }
    }

    fn recalculate_tasks_ids(parent_id: usize, new_id: usize, tasks: &mut Vec<Box<TaskItem>>) {
        for i in 0..tasks.len() {
            if tasks[i].id >= new_id  {
                tasks[i].id += 1;
            }

            if tasks[i].id != parent_id {
                GroupItem::recalculate_tasks_ids(parent_id, new_id, &mut tasks[i].tasks);
            }
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
        let file = fs::read_to_string("src/data/data.json").expect("Couldn't open file");
        let full_json : DataManager = serde_json::from_str(&file).unwrap();
        self.groups = full_json.groups;
    }

    pub fn save_state(&mut self) {
        let full_json = serde_json::to_string_pretty(self).expect("Couldn't serialized");
        fs::write("src/data/data.json", full_json).expect("Couldn't write to data file");

    }
}