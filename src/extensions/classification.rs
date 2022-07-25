use derive_new::new;
use itertools::Itertools;
use serde::Serialize;

use crate::{
    core::{
        characteristics::Fitness,
        inputs::{Inputs, ValidInput},
        program::Program,
        registers::Registers,
    },
    utils::float_ops,
};

use super::core::ExtensionParameters;

#[derive(Clone, Debug, Serialize, new)]
pub struct ClassificationParameters<InputType>
where
    InputType: ClassificationInput,
{
    inputs: Inputs<InputType>,
}

impl<T> ExtensionParameters for ClassificationParameters<T>
where
    T: ClassificationInput,
{
    fn argmax(registers: &Registers) -> i32 {
        let action_registers = &registers[0..T::N_ACTION_REGISTERS];
        let max_value = float_ops::max_val(action_registers.iter().copied())
            .expect("Action registers to have a max value.");

        let mut indices = action_registers
            .into_iter()
            .enumerate()
            .filter(|(_, value)| **value == max_value)
            .map(|(index, _)| index)
            .collect_vec();

        if indices.len() > 1 {
            -1
        } else {
            indices.remove(0) as i32
        }
    }
}

pub trait ClassificationInput: ValidInput {
    fn get_class(&self) -> usize;
}

impl<T> Fitness for Program<ClassificationParameters<T>>
where
    T: ClassificationInput,
{
    type FitnessParameters = ClassificationParameters<T>;

    fn eval_fitness(&mut self, parameters: &mut Self::FitnessParameters) -> f64 {
        let inputs = &parameters.inputs;

        let mut n_correct = 0.0;

        for input in inputs {
            self.exec(input);

            let predicted_class = ClassificationParameters::<T>::argmax(&self.registers);
            let correct_class = input.get_class() as i32;

            if predicted_class == correct_class {
                n_correct += 1.;
            }

            self.registers.reset();
        }

        let fitness = n_correct / inputs.len() as f64;

        self.fitness = Some(fitness);

        fitness
    }

    fn get_fitness(&self) -> Option<f64> {
        self.fitness
    }
}
