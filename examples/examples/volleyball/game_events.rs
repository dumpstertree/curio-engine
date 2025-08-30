#[derive(Clone)]
pub enum GameEvents {
    Begin,
    TurnBegin(i32),
    TurnEnd(i32),
}
