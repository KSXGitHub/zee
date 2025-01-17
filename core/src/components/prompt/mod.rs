pub mod buffers;
pub mod picker;

mod matcher;
mod status;

use std::{borrow::Cow, path::PathBuf, rc::Rc};
use zi::{
    components::text::{Text, TextProperties},
    Background, Callback, Component, ComponentExt, ComponentLink, Foreground, Layout, Rect,
    ShouldRender, Style,
};

use self::{
    buffers::{BufferEntry, BufferPicker, Properties as BufferPickerProperties},
    picker::{FilePicker, FileSource, Properties as FilePickerProperties},
};
use crate::editor::{BufferId, Context};

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub action: Style,
    pub input: Style,
    pub cursor: Style,
    pub mode: Foreground,
    pub file_size: Foreground,
    pub item_focused_background: Background,
    pub item_unfocused_background: Background,
    pub item_file_foreground: Foreground,
    pub item_directory_foreground: Foreground,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    None,
    Log {
        message: String,
    },
    SwitchBuffer {
        entries: Vec<BufferEntry>,
        on_select: Callback<BufferId>,
        on_change_height: Callback<usize>,
    },
    OpenFile {
        source: FileSource,
        on_open: Callback<PathBuf>,
        on_change_height: Callback<usize>,
    },
}

impl Action {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn is_interactive(&self) -> bool {
        !matches!(self, Self::None | Self::Log { .. })
    }

    pub fn initial_height(&self) -> usize {
        match self {
            Self::SwitchBuffer { ref entries, .. } => {
                1 + std::cmp::min(std::cmp::max(entries.len(), 1), PROMPT_MAX_HEIGHT)
            }
            _ => 1,
        }
    }

    // pub fn notify_height_change(&self, height: usize) {
    //     use Action::*;
    //     match self {
    //         OpenFile {
    //             ref on_change_height,
    //             ..
    //         } => on_change_height.emit(height),
    //         _ => {}
    //     }
    // }
}

#[derive(Clone)]
pub struct Properties {
    pub context: Rc<Context>,
    pub theme: Cow<'static, Theme>,
    pub action: Action,
}

pub struct Prompt {
    properties: Properties,
}

impl Component for Prompt {
    type Message = ();
    type Properties = Properties;

    fn create(properties: Self::Properties, _frame: Rect, _link: ComponentLink<Self>) -> Self {
        Self { properties }
    }

    fn change(&mut self, properties: Self::Properties) -> ShouldRender {
        let should_render = (self.properties.action != properties.action
            || self.properties.theme != properties.theme)
            .into();
        self.properties = properties;

        // if initial_height != self.height() {
        //     self.properties.action.notify_height_change(self.height());
        // }

        should_render
    }

    fn view(&self) -> Layout {
        log::info!("Prompt action: {:?}", self.properties.action);
        match self.properties.action {
            Action::None => Text::with(TextProperties::new().style(self.properties.theme.input)),
            Action::Log { ref message } => Text::with(
                TextProperties::new()
                    .content(message.clone())
                    .style(self.properties.theme.input),
            ),
            Action::SwitchBuffer {
                ref entries,
                ref on_select,
                ref on_change_height,
            } => {
                let on_change_height = on_change_height.clone();
                let on_filter = (move |size| {
                    on_change_height.emit(1 + std::cmp::min(15, std::cmp::max(1, size)));
                })
                .into();

                BufferPicker::with(BufferPickerProperties {
                    context: self.properties.context.clone(),
                    theme: self.properties.theme.clone(),
                    entries: entries.clone(),
                    on_select: on_select.clone(),
                    on_filter,
                })
            }
            Action::OpenFile {
                source,
                ref on_change_height,
                ref on_open,
            } => FilePicker::with(FilePickerProperties {
                context: self.properties.context.clone(),
                theme: self.properties.theme.clone(),
                source,
                on_open: on_open.clone(),
                on_change_height: on_change_height.clone(),
            }),
        }
    }

    fn has_focus(&self) -> bool {
        false
    }
}

pub const PROMPT_INACTIVE_HEIGHT: usize = 1;
const PROMPT_MAX_HEIGHT: usize = 15;
