use crate::traits::ui_panel::UIPanel;
use std::{fmt::Display, hash::Hash};

pub trait IUIEvent: Clone + Copy + Display + Sync + PartialEq + Eq + Hash {
    fn new_instance(&self) -> Box<dyn UIPanel>;
}
