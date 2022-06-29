use crossterm::event::KeyCode;
use tui::backend::Backend;
use tui::Frame;
use tui::layout::{Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, BorderType, ListItem};
use crate::{App, centered_rect, DataManager, FocusedLayout, LayoutCommon, LayoutCommonTrait};

use tui::{widgets::{List}};

use unicode_width::UnicodeWidthStr;
use crate::data_manager::GroupItem;
use crate::enums::InputMode;

pub struct GroupLayout {
    pub(crate) layout_common: LayoutCommon
}

impl GroupLayout {
    pub fn new() -> GroupLayout {
        GroupLayout {
            layout_common: LayoutCommon::new()
        }
    }
}






impl LayoutCommonTrait for GroupLayout {

    fn handle_input(&mut self, data_manager: &mut DataManager, key_code: crossterm::event::KeyEvent) {
        match self.layout_common.input_mode {
            InputMode::Navigate => {
                match key_code.code {
                    KeyCode::Char('a') => {
                        self.layout_common.input_mode = InputMode::Add;
                        self.layout_common.input = String::new();
                        self.layout_common.cursor_pos = self.layout_common.input.width();

                        LayoutCommon::recalculate_input_string_starting_point(&mut self.layout_common);
                    },
                    KeyCode::Esc => {
                        if self.layout_common.input_mode != InputMode::Navigate { return; }
                        self.layout_common.input_mode = InputMode::Navigate;
                    },
                    KeyCode::Char('e') => {
                        self.layout_common.input_mode = InputMode::Edit;
                        self.layout_common.input = data_manager.get_group(data_manager.selected_group).name.clone();
                        self.layout_common.cursor_pos = self.layout_common.input.width();

                        LayoutCommon::recalculate_input_string_starting_point(&mut self.layout_common);
                    },
                    KeyCode::Char('d') => {
                        data_manager.delete_group_item(data_manager.selected_group);
                        data_manager.save_state();
                        data_manager.selected_group = 0;
                    },
                    KeyCode::Up => {
                        if data_manager.selected_group > 0 {
                            data_manager.selected_group -= 1;
                        }
                    },
                    KeyCode::Down => {
                        if data_manager.selected_group < data_manager.get_group_items().len() - 1 {
                            data_manager.selected_group += 1;
                        }
                    }
                    _ => {}
                }
            },
            InputMode::Add => {
                match key_code.code {
                    KeyCode::Enter => {
                        let mut gi = GroupItem::new(data_manager);
                        gi.name = self.layout_common.input.drain(..).collect();
                        data_manager.add_group_item(gi);
                        self.layout_common.input_mode = InputMode::Navigate;
                        data_manager.save_state();
                    },
                    _ => {
                        <GroupLayout as LayoutCommonTrait>::poll_common_keys_input_mode(&key_code, &mut self.layout_common)
                    }
                }
            },
            InputMode::Edit => {
                match key_code.code {
                    KeyCode::Enter => {
                        data_manager.edit_group_item(data_manager.selected_group, self.layout_common.input.drain(..).collect());
                        self.layout_common.input_mode = InputMode::Navigate;
                        data_manager.save_state();
                    },
                    _ => {
                        <GroupLayout as LayoutCommonTrait>::poll_common_keys_input_mode(&key_code, &mut self.layout_common)
                    }
                }
            }
        }
    }

    fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>, frame_size: &Rect) {
        GroupLayout::create_and_render_base_block(f, app, chunk);
        GroupLayout::create_and_render_item_list(f, app, chunk, frame_size);
    }

    fn create_and_render_base_block<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>) {
        let mut groups_block = Block::default()
            .title("Groups")
            .borders(Borders::ALL)
            .style(Style::default());

        if app.focused_layout == FocusedLayout::GroupsLayout {
            groups_block = groups_block.style(Style::default().add_modifier(Modifier::BOLD))
                .border_type(BorderType::Thick);
        }

        f.render_widget(groups_block, chunk[0]);
    }

    fn create_and_render_item_list<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>, frame_size: &Rect) {
        let area = centered_rect(85, 85, chunk[0]);
        let mut items_list: Vec<ListItem> = Vec::new();

        let height = ListItem::new("Hello").style(Style::default()).height();
        let max_lines : usize = (frame_size.height as usize / (2 * height)) as usize;
        let selected_group = app.data_manager.selected_group;
        let showing_start_item = if selected_group > max_lines { selected_group - max_lines } else { 0 };

        for i in showing_start_item..app.data_manager.get_group_items().len() {
            let mut line = String::new();
            let group_name = app.data_manager.get_group_items()[i].name.as_str();
            line.push_str(group_name);

            if i == app.data_manager.selected_group {
                line = "  ".to_string();
                line.push_str(group_name);
                items_list.push(ListItem::new(line).style(Style::default().fg(Color::Cyan).remove_modifier(Modifier::BOLD)));
                continue;
            }

            line = "  ".to_string();
            line.push_str(group_name);
            items_list.push(ListItem::new(line).style(Style::default().remove_modifier(Modifier::BOLD)));
        }

        let items = List::new(items_list)
            .block(Block::default().borders(Borders::NONE))
            .highlight_style(
                Style::default()
                    .fg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_widget(items, area);
    }

    fn create_and_render_edit_mode<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>) {
        let title : String = if app.group_layout.layout_common.input_mode == InputMode::Add { "Add group".to_string() } else { "Edit group".to_string() };
        <GroupLayout as LayoutCommonTrait>::render_common_input_mode(f, &mut app.group_layout.layout_common, title.as_str(), chunk);
    }
}

