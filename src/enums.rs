#[derive(Copy, Clone, PartialEq)]
pub enum FocusedLayout {
    None,
    GroupsLayout,
    TasksLayout,
    ControlsLayout
}

#[derive(Copy, Clone, PartialEq)]
pub enum InputMode {
    Navigate,
    Add,
    Edit
}