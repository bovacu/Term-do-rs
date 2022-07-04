use std::fs;
use std::fs::File;
use ini::{Ini, Properties};
use tui::style::Color;

pub struct ConfigManager {
    pub(crate) task: Properties,
    pub(crate) group: Properties,
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self { task: Default::default(), group: Default::default() }
    }
}

impl ConfigManager {
    pub fn new() -> ConfigManager {
        let conf = Ini::load_from_file("settings.ini");

        if conf.is_err() {
            File::create("settings.ini").expect("Couldn't create config.ini");
            let content = r#"
                [group]
                selected_color="(6, 152, 154)"
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
            "#;

            fs::write("settings.ini", content).expect("Couldn't write contents");
        }

        let conf = Ini::load_from_file("settings.ini").unwrap();
        let task_conf = conf.section(Some("task")).unwrap().clone();
        let group_conf = conf.section(Some("group")).unwrap().clone();

        ConfigManager {
            task: task_conf,
            group: group_conf,
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
}