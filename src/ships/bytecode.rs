use super::primitives::*;

impl Function {
    pub fn new(actions: Vec<Action>) -> Self {
        Function { actions }
    }

    pub fn execute(&self) {
        unimplemented!();
    }
}