// For testing purposes only (binary classification).

use num::FromPrimitive;
use num_derive::FromPrimitive;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use strum::EnumCount;

use crate::{
    core::{
        instruction::{Mode, Modes},
        registers::Registers,
    },
    examples::iris::ops::IRIS_EXECUTABLES,
    extensions::classification::ClassificationInput,
};

use super::{
    common_traits::{Compare, Executables, Show, ValidInput},
    random::generator,
};

#[derive(PartialEq, PartialOrd, Ord, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct TestInput(pub [usize; 5]);

impl Into<Registers> for TestInput {
    fn into(self) -> Registers {
        todo!()
    }
}
impl Compare for TestInput {}
impl Show for TestInput {}

#[derive(Eq, PartialEq, Ord, PartialOrd, FromPrimitive, Hash, Clone, EnumCount)]
pub enum TestRepresent {
    One = 0,
    Two = 1,
}

impl Compare for TestRepresent {}

impl ValidInput for TestInput {
    type Actions = TestRepresent;

    fn argmax(ties: Vec<usize>) -> Option<Self::Actions> {
        FromPrimitive::from_usize(*ties.choose(&mut generator()).unwrap())
    }

    const AVAILABLE_EXECUTABLES: Executables = IRIS_EXECUTABLES;

    const AVAILABLE_MODES: Modes = Mode::ALL;
}

impl ClassificationInput for TestInput {
    const N_INPUTS: usize = 4;

    fn get_class(&self) -> TestRepresent {
        FromPrimitive::from_usize(self.0[Self::N_INPUTS]).unwrap()
    }
}
