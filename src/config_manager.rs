use std::fs;
use std::fs::File;
use crossterm::event::KeyCode;
use ini::{Ini, Properties};
use tui::style::Color;

pub struct ConfigManager {
    pub(crate) ini: Ini,
    pub(crate) task: Properties,
    pub(crate) group: Properties,
    pub(crate) path: Properties,
    pub(crate) default_settings_file: String,
    input: Properties
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self { ini: Default::default(), task: Default::default(), group: Default::default(), input: Default::default(), default_settings_file: Default::default(), path: Default::default() }
    }
}

impl ConfigManager {
    pub fn new() -> ConfigManager {
        let conf = Ini::load_from_file("settings.ini");
        let content = r#"
[group]
selected_color="(6, 152, 154)"
non_selected_color="(255, 255, 255)"
border_color="(255, 255, 255)"
icon=""

[task]
selected_color="(196, 160, 0)"
non_selected_color="(255, 255, 255)"
completed_color="(78, 154, 6)"
border_color="(255, 255, 255)"
icon_uncompleted=""
icon_completed=""
vertical_child_char_icon="║"
turn_right_child_char_icon="╚"
horizontal_child_char_icon="═"

[key_bindings]
add_group='a'
edit_group='e'
delete_group='d'
apply_add_or_edit_group="Enter"
down_group="Down"
up_group="Up"
add_task='a'
add_subtask='A'
delete_task_or_subtask='d'
edit_task_or_subtask='e'
complete_or_uncomplete_task='c'
apply_add_or_edit_task_or_subtask="Enter"
down_task_or_subtask="Down"
up_task_or_subtask="Up"
fold_subtasks='f'
undo='u'
redo='r'

[paths]
settings_path='.'
data_path='.'
            "#;

        if conf.is_err() {
            File::create("settings.ini").expect("Couldn't create config.ini");
            fs::write("settings.ini", content).expect("Couldn't write contents");
        }

        let (conf, task_conf, group_conf, input_conf, path_conf) = ConfigManager::load_config(content, "settings.ini");

        ConfigManager {
            ini: conf.clone(),
            task: task_conf.clone(),
            group: group_conf.clone(),
            input: input_conf.clone(),
            path: path_conf.clone(),
            default_settings_file: content.parse().unwrap(),
        }
    }

    pub fn get_color(&self, section: &str, key: &str) -> Color {
        let properties : &Properties;
        if section.eq("group") {
            properties = &self.group;
        } else {
            properties = &self.task;
        }

        let color = properties.get(key).unwrap().replace('(', "").replace(')', "");
        let values: Vec<&str> = color.split(",").collect();

        return Color::Rgb(values[0].trim().parse().unwrap(), values[1].trim().parse().unwrap(), values[2].trim().parse().unwrap());
    }

    pub fn get_key(&self, key: &str) -> KeyCode {
        let key_value = self.input.get(key).unwrap();
        return if key_value.len() > 1 {
            let kv_lower = key_value.to_lowercase();

            match kv_lower.as_str() {
                "backspace" => KeyCode::Backspace,
                "enter" => KeyCode::Enter,
                "end" => KeyCode::End,
                "esc" => KeyCode::Esc,
                "delete" => KeyCode::Delete,
                "up" => KeyCode::Up,
                "down" => KeyCode::Down,
                "right" => KeyCode::Right,
                "left" => KeyCode::Left,
                "backtab" => KeyCode::BackTab,
                "tab" => KeyCode::Tab,
                "home" => KeyCode::Home,
                "pageup" => KeyCode::PageUp,
                "pagedown" => KeyCode::PageDown,
                "insert" => KeyCode::Insert,
                _ => KeyCode::Null
            }
        } else {
            KeyCode::Char(key_value.chars().nth(0).unwrap())
        }
    }

    fn load_config(content: &str, path_to_search: &str) -> (Ini, Properties, Properties, Properties, Properties) {
        let mut conf_conf = Ini::load_from_file(path_to_search).unwrap();
        let mut task_conf = conf_conf.section(Some("task"));
        let mut group_conf = conf_conf.section(Some("group"));
        let mut input_conf = conf_conf.section(Some("key_bindings"));
        let mut path_conf = conf_conf.section(Some("paths"));

        if task_conf.is_none() || group_conf.is_none() || input_conf.is_none() || path_conf.is_none() {
            fs::write("settings.ini", content).expect("Couldn't write contents");
            conf_conf = Ini::load_from_file("settings.ini").unwrap();
            task_conf = conf_conf.section(Some("task"));
            group_conf = conf_conf.section(Some("group"));
            input_conf = conf_conf.section(Some("key_bindings"));
            path_conf = conf_conf.section(Some("paths"));
        }

        let conf : Ini;
        let task : Properties;
        let group : Properties;
        let input : Properties;
        let path : Properties;

        if !path_conf.unwrap().get("settings_path").unwrap().eq(".") {
            let (c, t, g, i, p) = ConfigManager::load_config(content, format!("{}/{}", path_conf.unwrap().get("settings_path").unwrap(), "settings.ini").as_str());
            conf = c;
            task = t;
            group = g;
            input = i;
            path = p;
        } else {
            conf = conf_conf.clone();
            task = task_conf.unwrap().clone();
            group = group_conf.unwrap().clone();
            input = input_conf.unwrap().clone();
            path = path_conf.unwrap().clone();
        }

        return (conf.clone(), task.clone(), group.clone(), input.clone(), path.clone());
    }
}