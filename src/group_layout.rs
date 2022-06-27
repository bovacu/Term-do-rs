use crossterm::event::KeyCode;
use tui::backend::Backend;
use tui::Frame;
use tui::layout::{Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, BorderType, ListItem, Paragraph};
use crate::{App, centered_rect, DataManager, FocusedLayout, LayoutState};

use tui::{widgets::{List}};

use unicode_width::UnicodeWidthStr;
use crate::data_manager::GroupItem;

pub struct GroupLayout {
    edit_mode: bool,
    input: String
}

impl GroupLayout {
    pub fn new() -> GroupLayout {
        GroupLayout {
            edit_mode: false,
            input: String::new()
        }
    }
}

impl LayoutState for GroupLayout {

    fn is_in_edit_mode(&self) -> bool {
        return self.edit_mode;
    }

    fn handle_input(&mut self, data_manager: &mut DataManager, key_code: crossterm::event::KeyEvent) {
        if !self.edit_mode {
            match key_code.code {
                KeyCode::Char('a') => {
                    self.edit_mode = true;
                },
                KeyCode::Char('q') | KeyCode::Esc => {
                    if !self.edit_mode { return; }
                    self.edit_mode = false;
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
        } else {
            match key_code.code {
                KeyCode::Enter => {
                    let mut gi = GroupItem::new(data_manager);
                    gi.name = self.input.drain(..).collect();
                    data_manager.add_group_item(gi);
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
        GroupLayout::create_and_render_base_block(f, app, chunk);
        GroupLayout::create_and_render_item_list(f, app, chunk);
        GroupLayout::create_and_render_edit_mode(f, app, chunk, lower_chunk);
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

    fn create_and_render_item_list<B: Backend>(f: &mut Frame<B>, app: &mut App, chunk: &Vec<Rect>) {
        let area = centered_rect(85, 85, chunk[0]);
        let mut items_list: Vec<ListItem> = Vec::new();
        for i in 0..app.data_manager.get_group_items().len() {
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

    fn create_and_render_edit_mode<B: Backend>(f: &mut Frame<B>, app: &mut App, _chunk: &Vec<Rect>, lower_chunk: &Rect) {
        if app.group_layout.edit_mode {
            let options_block = Block::default().title("Add group").borders(Borders::ALL);
            let area = centered_rect(40, 10, *lower_chunk);

            let input = Paragraph::new(app.group_layout.input.as_ref())
                .style( if app.group_layout.edit_mode {
                            Style::default().add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        })
                .block(options_block);

            f.render_widget(input, area);


            if app.group_layout.edit_mode {
                f.set_cursor(
                    // Put cursor past the end of the input text
                    area.x + app.group_layout.input.width() as u16 + 1,
                    // Move one line down, from the border to the input line
                    area.y + 1,
                )
            }
        }
    }
}

