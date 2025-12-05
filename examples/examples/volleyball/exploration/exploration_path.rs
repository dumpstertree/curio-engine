use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Hash, Default, Clone, Serialize, Deserialize)]
pub struct Exploration {
    all_rooms: Vec<Room>,
    cur_room: Vec<i32>,
}

impl Exploration {
    pub fn random() -> Exploration {
        let mut rooms = Vec::new();
        for i in 0..3 {
            rooms.push(Room {
                guid: i,
                room_type: if i == 0 || i == 2 { RoomTypes::Heal } else { RoomTypes::Combat },
                prev_room: i - 1,
                next_rooms: vec![i + 1],
                is_start: i == 0,
            });
        }

        Exploration { all_rooms: rooms, cur_room: Vec::new() }
    }
    pub fn get_cur_room(&self) -> Room {
        for room in &self.all_rooms {
            if &room.guid == self.cur_room.last().unwrap() {
                return room.clone();
            }
        }

        Room::default()
    }
    pub fn get_next_room(&self) -> Vec<Room> {
        let mut rooms = Vec::new();
        for room in &self.all_rooms {
            // is cur room
            if &room.guid == self.cur_room.last().unwrap() {
                // for each room in visited
                for r in &self.all_rooms {
                    let is_in_next = room.next_rooms.contains(&r.guid);
                    if is_in_next {
                        rooms.push(r.clone());
                    }
                }
                break;
            }
        }
        rooms
    }
    pub fn get_previous_rooms(&self) -> Vec<Room> {
        let mut rooms = Vec::new();
        for room in &self.all_rooms {
            // is cur room
            if &room.guid == self.cur_room.last().unwrap() {
                // for each room in visited
                for i in (0..self.cur_room.len() - 1).rev() {
                    for r in &self.all_rooms {
                        if r.guid == self.cur_room[i] {
                            rooms.push(r.clone())
                        }
                    }
                }

                break;
            }
        }
        rooms
    }
    pub fn start(&mut self) -> Room {
        for room in &self.all_rooms {
            if room.is_start {
                self.cur_room.push(room.guid);
                break;
            }
        }
        self.get_cur_room()
    }
    pub fn next(&mut self, next_room_guid: &i32) -> Room {
        for room in &self.all_rooms {
            if *next_room_guid == room.guid {
                self.cur_room.push(*next_room_guid);
                break;
            }
        }
        self.get_cur_room()
    }
}
#[derive(PartialEq, Eq, Hash, Default, Clone, Serialize, Deserialize)]
pub struct Room {
    pub guid: i32,
    pub room_type: RoomTypes,
    pub prev_room: i32,
    pub next_rooms: Vec<i32>,
    pub is_start: bool,
}
#[derive(PartialEq, Eq, Hash, Default, Clone, Serialize, Deserialize)]
pub enum RoomTypes {
    #[default]
    Invalid,
    Combat,
    Heal,
    Shop,
    Boss,
}
