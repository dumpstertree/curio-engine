use std::sync::Arc;

pub struct Deck {
    stacks: Vec<Stack>,
}

impl Deck {
    pub fn new(stacks: Vec<Arc<Stack>>) {}
    pub fn shuffle() {}
    // pub fn moves() {}  pub fn shuffle() {}
    pub fn moves() {}
    pub fn get_stack_by_name(name: &str) {}
    pub fn get_stack_guid(guid: &i32) {}
}

pub struct Stack {
    pub guid: i32,
    pub name: String,
    cards: Vec<Arc<Card>>,
}
impl Stack {
    pub fn new(cards: Vec<Arc<Card>>) {}
    pub fn shuffle() {}
}

pub struct Card {
    pub guid: i32,
    pub name: String,
}

pub struct Modifier {}
pub struct Event {}
pub struct Hook {}
