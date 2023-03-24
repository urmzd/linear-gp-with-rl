use rand::{distributions::Uniform, prelude::Distribution};

use crate::utils::random::generator;
use itertools::Itertools;

use super::{characteristics::Breed, instruction::Instruction};

impl Breed for Instructions {
    fn two_point_crossover(&self, mate: &Self) -> [Self; 2] {
        let mut instructions_a = self.clone();
        let mut instructions_b = mate.clone();

        let current_generator = &mut generator();

        debug_assert!(instructions_a.len() > 0);
        debug_assert!(instructions_b.len() > 0);

        let a_start = Uniform::new(0, instructions_a.len()).sample(current_generator);
        let b_start = Uniform::new(0, instructions_b.len()).sample(current_generator);

        let a_end = if a_start == instructions_a.len() - 1 {
            None
        } else {
            debug_assert!(instructions_a.len() > a_start);
            Some(Uniform::new(a_start + 1, instructions_a.len()).sample(current_generator))
        };

        let b_end = if b_start == instructions_b.len() - 1 {
            None
        } else {
            debug_assert!(instructions_b.len() > b_start);
            Some(Uniform::new(b_start + 1, instructions_b.len()).sample(current_generator))
        };

        let a_chunk = match a_end {
            None => &instructions_a[a_start..],
            Some(a_end_idx) => &instructions_a[a_start..a_end_idx],
        }
        .iter()
        .cloned()
        .collect_vec();

        let b_chunk = match b_end {
            None => &instructions_b[b_start..],
            Some(b_end_idx) => &instructions_b[b_start..b_end_idx],
        }
        .iter()
        .cloned()
        .collect_vec();

        if let Some(a_end_idx) = a_end {
            instructions_a.splice(a_start..a_end_idx, b_chunk)
        } else {
            instructions_a.splice(a_start.., b_chunk)
        }
        .collect_vec();

        if let Some(b_end_idx) = b_end {
            instructions_b.splice(b_start..b_end_idx, a_chunk)
        } else {
            instructions_b.splice(b_start.., a_chunk)
        }
        .collect_vec();

        debug_assert!(instructions_a.len() > 0, "instructions A after crossover");
        debug_assert!(instructions_b.len() > 0, "instructions B after crossover");

        [instructions_a, instructions_b]
    }
}

pub type Instructions = Vec<Instruction>;

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use crate::{
        core::{
            characteristics::{Breed, Generate},
            inputs::ValidInput,
            instruction::InstructionGeneratorParameters,
            program::{Program, ProgramGeneratorParameters},
        },
        utils::test::TestInput,
    };

    #[test]
    fn given_two_programs_when_two_point_crossover_multiple_times_then_instruction_set_never_grows()
    {
        let max_instructions = 100;
        let parameters = ProgramGeneratorParameters {
            max_instructions,
            instruction_generator_parameters: InstructionGeneratorParameters {
                n_extras: 1,
                external_factor: 10.,
                n_inputs: TestInput::N_INPUTS,
                n_actions: TestInput::N_ACTIONS,
            },
        };

        let program_a = Program::<ClassificationParameters<TestInput>>::generate(parameters);
        let program_b = Program::<ClassificationParameters<TestInput>>::generate(parameters);

        let mut parents = [program_a, program_b];

        for _ in 0..100 {
            let parent_a_instruction_len = parents[0].instructions.len();
            let parent_b_instruction_len = parents[1].instructions.len();

            let new_parents = Breed::two_point_crossover(&parents[0], &parents[1]);

            debug_assert!(new_parents[0].instructions.len() > 0);
            debug_assert!(new_parents[1].instructions.len() > 0);

            debug_assert!(
                new_parents[0].instructions.len()
                    <= parent_a_instruction_len + parent_b_instruction_len
            );
            debug_assert!(
                new_parents[1].instructions.len()
                    <= parent_a_instruction_len + parent_b_instruction_len
            );

            parents = new_parents;
        }
    }
}
