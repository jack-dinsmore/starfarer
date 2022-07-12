use serde::{Serialize, Deserialize};
use super::{PartID};

pub type FunctionID = u32;

#[derive(Serialize, Deserialize)]
pub struct Function {
    actions: Vec<Action>
}
impl Function {
    pub fn new(actions: Vec<Action>) -> Self {
        Function { actions }
    }

    pub fn execute(&self) {
        for action in &self.actions {
            match action {
                _ => unimplemented!(),
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum Action {
    Execute(PartID, FunctionID)
}

