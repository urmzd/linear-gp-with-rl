use core::fmt;
use std::path::PathBuf;

use csv::ReaderBuilder;
use more_asserts::assert_le;
use noisy_float::prelude::{r64, Float};
use rand::prelude::{IteratorRandom, SliceRandom};
use serde::de::DeserializeOwned;

use crate::{
    core::characteristics::{Breed, Fitness, Generate},
    utils::random::generator,
};

use super::{
    characteristics::Mutate,
    inputs::{Inputs, ValidInput},
    population::Population,
};

#[derive(Debug)]
pub struct HyperParameters<OrganismType>
where
    OrganismType: Fitness + Mutate + Generate,
{
    pub population_size: usize,
    pub gap: f64,
    pub n_mutations: usize,
    pub n_crossovers: usize,
    pub max_generations: usize,
    pub fitness_parameters: OrganismType::FitnessParameters,
    pub program_parameters: OrganismType::GeneratorParameters,
}

pub trait Loader
where
    Self::InputType: ValidInput + DeserializeOwned,
{
    type InputType;

    fn load_inputs(file_path: impl Into<PathBuf>) -> Inputs<Self::InputType> {
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(false)
            .from_path(file_path.into())
            .unwrap();

        let inputs: Result<Inputs<Self::InputType>, _> = csv_reader
            .deserialize()
            .into_iter()
            .map(|input| input)
            .collect();

        inputs.unwrap()
    }
}

pub trait GeneticAlgorithm
where
    Self::O: Fitness + Generate + PartialOrd + Sized + Clone + Mutate + Breed + fmt::Debug,
{
    type O;

    /// Prevent errors from being thrown when "multple" initializations occur.
    /// TODO: Remove monkey patch.
    fn init_env() -> () {
        pretty_env_logger::try_init().unwrap_or(());
    }

    fn init_population(hyper_params: &HyperParameters<Self::O>) -> Population<Self::O> {
        let mut population = Population::with_capacity(hyper_params.population_size);

        for _ in 0..hyper_params.population_size {
            let program = Self::O::generate(&hyper_params.program_parameters);
            population.push(program)
        }

        population
    }

    fn rank(
        population: &mut Population<Self::O>,
        fitness_parameters: &mut <Self::O as Fitness>::FitnessParameters,
    ) {
        for individual in population.iter_mut() {
            if individual.get_fitness().is_none() {
                individual.eval_fitness(fitness_parameters);
                assert_ne!(individual.get_fitness(), None);
            }
        }
        population.sort();
    }

    fn apply_selection(population: &mut Population<Self::O>, gap: f64) {
        assert!(gap >= 0. && gap <= 1.);
        assert_le!(population.last(), population.first());

        let pop_len = population.len();

        let cutoff_index = ((r64(1.0) - gap) * (pop_len as f64)).floor().const_raw() as usize;

        for _ in 0..cutoff_index {
            population.pop();
        }
    }

    fn breed(
        population: &mut Population<Self::O>,
        mut n_mutations: usize,
        mut n_crossovers: usize,
        mutation_parameters: &<Self::O as Generate>::GeneratorParameters,
    ) {
        let pop_cap = population.capacity();
        let pop_len = population.len();

        let mut remaining_pool_spots: usize = pop_cap - pop_len;

        assert_le!(n_mutations + n_crossovers, remaining_pool_spots);

        let mut children = vec![];

        // Crossover + Mutation
        // TODO: Add a way to priortize mutations or crossovers.
        while (n_crossovers + n_mutations) > 0 {
            let selected_a = population.iter().choose(&mut generator());
            let selected_b = population.iter().choose(&mut generator());

            if let (Some(parent_a), Some(parent_b)) = (selected_a, selected_b) {
                if n_crossovers > 0 {
                    let crossover_child = parent_a
                        .two_point_crossover(parent_b)
                        .choose(&mut generator())
                        .unwrap()
                        .to_owned();

                    remaining_pool_spots -= 1;
                    n_crossovers -= 1;
                    children.push(crossover_child)
                }

                if n_mutations > 0 {
                    let parents = [parent_a, parent_b];
                    let selected_parent = parents.choose(&mut generator());

                    let mutation_child = selected_parent
                        .map(|parent| parent.mutate(mutation_parameters))
                        .unwrap();

                    remaining_pool_spots -= 1;
                    n_mutations -= 1;

                    children.push(mutation_child)
                }
            };
        }

        // Fill reset with clones
        for individual in population
            .iter()
            .cloned()
            .choose_multiple(&mut generator(), remaining_pool_spots)
        {
            population.push(individual)
        }

        population.extend(children)
    }

    fn execute<'b>(
        hyper_params: &mut HyperParameters<Self::O>,
        mut hooks: EventHooks<'b, Self::O>,
    ) -> Result<Population<Self::O>, Box<dyn std::error::Error>> {
        Self::init_env();

        let EventHooks {
            after_init,
            after_rank,
            after_selection,
            after_breed,
            ..
        } = &mut hooks;

        let mut population = Self::init_population(hyper_params);

        if let Some(hook) = after_init {
            (hook)(&mut population)?;
        }

        for _generation in 0..hyper_params.max_generations {
            Self::rank(&mut population, &mut hyper_params.fitness_parameters);
            if let Some(hook) = after_rank {
                (hook)(&mut population)?;
            }

            Self::apply_selection(&mut population, hyper_params.gap);
            if let Some(hook) = after_selection {
                (hook)(&mut population)?;
            }

            Self::breed(
                &mut population,
                hyper_params.n_mutations,
                hyper_params.n_crossovers,
                &hyper_params.program_parameters,
            );
            if let Some(hook) = after_breed {
                (hook)(&mut population)?;
            }
        }

        Ok(population)
    }
}

pub type GpHook<'a, O> =
    &'a mut dyn FnMut(&mut Population<O>) -> Result<(), Box<dyn std::error::Error>>;
pub struct EventHooks<'a, O>
where
    O: PartialOrd + Clone,
{
    pub after_init: Option<GpHook<'a, O>>,
    pub after_evaluate: Option<GpHook<'a, O>>,
    pub after_rank: Option<GpHook<'a, O>>,
    pub after_selection: Option<GpHook<'a, O>>,
    pub after_breed: Option<GpHook<'a, O>>,
}

impl<'a, O> EventHooks<'a, O>
where
    O: PartialOrd + Clone,
{
    pub fn with_after_init(self, f: GpHook<'a, O>) -> Self {
        Self {
            after_init: Some(f),
            ..self
        }
    }

    pub fn with_after_selection(self, f: GpHook<'a, O>) -> Self {
        Self {
            after_selection: Some(f),
            ..self
        }
    }

    pub fn with_after_rank(self, f: GpHook<'a, O>) -> Self {
        Self {
            after_rank: Some(f),
            ..self
        }
    }

    pub fn with_after_breed(self, f: GpHook<'a, O>) -> Self {
        Self {
            after_breed: Some(f),
            ..self
        }
    }
}

impl<'a, O> fmt::Debug for EventHooks<'a, O>
where
    O: PartialOrd + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventHooks")
            .field("after_init", &"after_init")
            .field("after_evaluate", &"after_evaluate")
            .field("after_selection", &"after_selection")
            .field("after_rank", &"after_rank")
            .field("after_breed", &"after_breed")
            .finish()
    }
}

impl<'a, O> Default for EventHooks<'a, O>
where
    O: PartialOrd + Clone,
{
    fn default() -> Self {
        Self {
            after_init: None,
            after_evaluate: None,
            after_rank: None,
            after_selection: None,
            after_breed: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::{
        core::{instruction::InstructionGeneratorParameters, program::ProgramGeneratorParameters},
        extensions::classification::ClassificationParameters,
        utils::{
            random::generator,
            test::{TestInput, TestLgp},
        },
    };
    use rand::{distributions::Standard, Rng};

    use super::{EventHooks, GeneticAlgorithm, HyperParameters};

    #[test]
    fn given_lgp_instance_with_event_hooks_when_execute_then_closures_are_executed(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let inputs = [0; 5].map(|_| generator().sample(Standard)).to_vec();
        let received = Rc::new(RefCell::new(Vec::new()));
        let mut hyper_params = HyperParameters {
            population_size: 10,
            gap: 0.5,
            n_mutations: 5,
            n_crossovers: 0,
            max_generations: 1,
            fitness_parameters: ClassificationParameters::new(inputs),
            program_parameters: ProgramGeneratorParameters::new(
                10,
                InstructionGeneratorParameters::from::<TestInput>(1),
            ),
        };

        TestLgp::execute(
            &mut hyper_params,
            EventHooks::default()
                .with_after_init(&mut |_p| {
                    received.borrow_mut().push(1);
                    Ok(())
                })
                .with_after_rank(&mut |_p| {
                    received.borrow_mut().push(2);
                    Ok(())
                })
                .with_after_selection(&mut |_p| {
                    received.borrow_mut().push(3);
                    Ok(())
                })
                .with_after_breed(&mut |_p| {
                    received.borrow_mut().push(4);
                    Ok(())
                }),
        )?;

        pretty_assertions::assert_eq!(received.borrow().as_slice(), &[1, 2, 3, 4]);

        Ok(())
    }
}
