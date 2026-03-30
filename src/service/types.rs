use stahlwerk_extension::{Time, Date, ff01::Entry};

#[derive(Debug, Default)]
pub enum State {
    #[default]
    Zero,
    One(StateOneData),
    Two(StateTwoData),
}

impl State {
    pub fn index(&self) -> u32 {
        match self {
            State::Zero => 0,
            State::One(_) => 1,
            State::Two(_) => 2,
        }
    }
}

#[derive(Debug)]
pub struct StateOneData {
    pub entry: Entry,
    pub start_date: Date,
    pub from_time:  Time,
}

#[derive(Debug)]
pub struct StateTwoData {
    pub state_one_data: StateOneData,

    pub personnel_id: String,
    pub quantity_scrap: f64,
}