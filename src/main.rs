use std::{error, io::Write, marker::PhantomData, path::Path};

use csv::ReaderBuilder;
use lgp::{
    algorithm::{GeneticAlgorithm, HyperParameters, Population},
    fitness::{Fitness, FitnessScore},
    inputs::Inputs,
    iris::iris_data::{IrisInput, IRIS_DATASET_LINK},
    program::Program,
    registers::RegisterRepresentable,
};
use rand::prelude::SliceRandom;
use tempfile::NamedTempFile;

/// Lets describe the steps we're trying to execute.
///
/// First we initialize a population of programs.
///
/// Programs consist of instructions.
///
/// Instructions consist of four things, a source index, a target index, an operation and a mode.
/// Each instruction is executed. The resulting registers are the the "outputs" of the program.
///
///
/// Data can be retrieved from two places; the registers or the inputs (external data, likely from
/// the fs).
///
/// Ex (Input -> (N, M)):
///
/// 0.1, 0.2, 0.3, 0.4, 0.5
/// 0.2, 0.3, 0.4, 0.5, 0.6
/// ...
///
/// Ex (Registers -> (N,)):
///
/// 0.1, 0.2, 0.3, 0.4, 0.5
///
/// As demonstrated above, inputs can expand in another dimension (in the above case, its the #. of
/// rows) as long as they share a dimension (in the above case, its the #. of columns)
///
/// Smoke Test Algorithm:
///     1. Load input data
///     2. Generate programs (instructions, registers, etc..) -- Init Population
///     3. Eval Fitness
///     --
///     4. Drop x% (tournament selection)
///     5. Clone 1 - x % (Pick from the population uniformly)
///     --
///     6. Repeat from 3 until best == median == worst
///
///
/// Notes:
///     Inputs should be referenced. (RC?)
///
/// Fitness Algorithm:
///     For every input:
///         run all instructions
///         -
///         argmax(registers) == correct_val
///         reset registers
///     Fitness Score = # of correct outputs / total.
///
///
/// Linear Genetic Programming -> 1 Runnable -> N Programmable -> Executable  
///
/// Questions Remaining:
///
/// - [] How do verify the integerity of our indices?
/// - [] Uniform Distribution?
///
/// Registers = # of Total Classes + 1
struct BasicLGP<'a, InputType>
where
    InputType: RegisterRepresentable,
{
    population: Population<'a, InputType>,
    inputs: Inputs<InputType>,
    hyper_params: HyperParameters,
}

// TODO: Optimize code (reduce cloning).
impl<'a> GeneticAlgorithm<'a> for BasicLGP<'a, IrisInput> {
    type InputType = IrisInput;

    fn load_inputs(file_path: &'a Path) -> Inputs<Self::InputType> {
        let mut csv_reader = ReaderBuilder::new()
            .has_headers(false)
            .from_path(file_path)
            .unwrap();

        let raw_inputs: Vec<IrisInput> = csv_reader
            .deserialize()
            .map(|input| -> IrisInput { input.unwrap() })
            .collect();

        return raw_inputs;
    }

    fn new(hyper_params: lgp::algorithm::HyperParameters<'a>) -> Self {
        let inputs = Self::load_inputs(hyper_params.input_path);
        let population: Population<'a, Self::InputType> =
            Vec::with_capacity(hyper_params.population_size);

        return BasicLGP(population, inputs, hyper_params);
    }

    fn init_population(&mut self) -> Self {
        let BasicLGP {
            population,
            hyper_params,
            inputs,
        } = &self;
        for _ in 0..hyper_params.population_size {
            let program = Program::generate(inputs, hyper_params.instruction_size);
            population.push(program)
        }
        self
    }

    fn eval_population(&mut self) -> Self {
        for individual in self.population {
            let fitness = individual.eval_fitness();
            individual.fitness = Some(fitness);
        }

        self.population.sort_by_key(|p| p.fitness.unwrap());
        self
    }

    fn apply_natural_selection(&mut self) -> Self {
        assert!(retention_rate >= 0f32 && retention_rate <= 1f32);
        assert!(self.population.is_sorted());

        let pop_len = self.population.len();

        let HyperParameters { retention_rate, .. } = &self;

        let lowest_index = ((1f32 - retention_rate) * (pop_len as f32)).floor() as i32 as usize;

        for index in 0..lowest_index {
            self.population.remove(index)
        }

        self
    }

    fn breed(&mut self) -> Self {
        let Self { population, .. } = &mut self;
        let remaining_size = population.capacity() - population.len();

        let selected_individuals =
            population.choose_multiple(&mut rand::thread_rng(), remaining_size);

        for individual in selected_individuals {
            population.push(individual.clone())
        }

        self
    }
}

async fn get_iris_content() -> Result<ContentFilePair, Box<dyn error::Error>> {
    let tmp_file = NamedTempFile::new()?;
    let response = reqwest::get(IRIS_DATASET_LINK).await?;
    let content = response.text().await?;
    writeln!(&tmp_file, "{}", &content)?;

    Ok(ContentFilePair(content, tmp_file))
}

struct ContentFilePair(String, NamedTempFile);

// Lo, Mid, Hi
struct Benchmark<'a, InputType: RegisterRepresentable>(
    &'a Program<'a, InputType>,
    &'a Program<'a, InputType>,
    &'a Program<'a, InputType>,
);

impl<'a, InputType> Benchmark<'a, InputType>
where
    InputType: RegisterRepresentable,
{
    fn get_benchmark_individuals(population: &'a Population<'a, InputType>) -> Self {
        let worst = population.first().unwrap();
        let middle_index = math::round::floor(population.len() as f64 / 2 as f64, 1) as usize;
        let median = population.get(middle_index).unwrap();
        let best = population.last().unwrap();

        return Benchmark(worst, median, best);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn error::Error>> {
    /*
     *    let ContentFilePair(_, tmp_file) = get_iris_content().await?;
     *
     *    let hyper_params = HyperParameters {
     *        input_path: tmp_file.path(),
     *        population_size: 100,
     *        instruction_size: 100,
     *        retention_rate: 0.5,
     *    };
     *
     *    let inputs = <BasicLGP as GeneticAlgorithm>::load_inputs(hyper_params.input_path);
     *    let mut pop = <BasicLGP as GeneticAlgorithm>::init_population(
     *        hyper_params.population_size,
     *        hyper_params.instruction_size,
     *        &inputs,
     *    );
     *
     *    pop.sort_by_cached_key(|p| p.eval_fitness());
     *
     *    let Benchmark(worst, median, best) = Benchmark::get_benchmark_individuals(&pop);
     *
     *    while worst != median && median != best {
     *        pop = <BasicLGP as GeneticAlgorithm>::retrieve_selection(&pop, hyper_params.retention_rate);
     *        pop = <BasicLGP as GeneticAlgorithm>::breed(pop);
     *    }
     */

    Ok(())
}

/*
 *#[cfg(test)]
 *mod tests {
 *    use std::error;
 *
 *    use rand::Rng;
 *
 *    use super::*;
 *
 *    #[tokio::test]
 *    async fn given_population_when_breeding_occurs_then_population_capacity_is_met(
 *    ) -> Result<(), Box<dyn error::Error>> {
 *        let ContentFilePair(_, tmp_file) = get_iris_content().await?;
 *        let inputs = <BasicLGP as GeneticAlgorithm>::load_inputs(tmp_file.path());
 *
 *        const SIZE: usize = 100;
 *        const MAX_INSTRUCTIONS: usize = 100;
 *
 *        let mut population =
 *            <BasicLGP as GeneticAlgorithm>::init_population(SIZE, MAX_INSTRUCTIONS, &inputs);
 *
 *        // Drop half approximately.
 *        population.retain(|_| rand::thread_rng().gen_bool(0.5));
 *
 *        let dropped_pop_len = population.len();
 *
 *        assert!(dropped_pop_len < SIZE);
 *
 *        let new_pop = <BasicLGP as GeneticAlgorithm>::breed(population);
 *
 *        println!("{}", new_pop.len());
 *
 *        assert!(new_pop.len() == SIZE);
 *
 *        Ok(())
 *    }
 *
 *    #[tokio::test]
 *    async fn given_population_and_retention_rate_when_selection_occurs_then_population_is_cut_by_dropout(
 *    ) -> Result<(), Box<dyn error::Error>> {
 *        let ContentFilePair(_, tmp_file) = get_iris_content().await?;
 *        let inputs = <BasicLGP as GeneticAlgorithm>::load_inputs(tmp_file.path());
 *
 *        const SIZE: usize = 100;
 *        const MAX_INSTRUCTIONS: usize = 100;
 *        const RETENTION_RATE: f32 = 0.5;
 *
 *        let population =
 *            <BasicLGP as GeneticAlgorithm>::init_population(SIZE, MAX_INSTRUCTIONS, &inputs);
 *
 *        let selected_population =
 *            <BasicLGP as GeneticAlgorithm>::retrieve_selection(population, RETENTION_RATE);
 *
 *        assert!(
 *            selected_population.len()
 *                == ((SIZE as f32 * (1f32 - RETENTION_RATE)).floor() as i32 as usize)
 *        );
 *
 *        Ok(())
 *    }
 *
 *    #[tokio::test]
 *    async fn given_inputs_and_hyperparams_when_population_is_initialized_then_population_generated_with_hyperparams_and_inputs(
 *    ) -> Result<(), Box<dyn error::Error>> {
 *        let ContentFilePair(_, tmp_file) = get_iris_content().await?;
 *        let inputs = <BasicLGP as GeneticAlgorithm>::load_inputs(tmp_file.path());
 *        const SIZE: usize = 100;
 *        const MAX_INSTRUCTIONS: usize = 100;
 *        let population =
 *            <BasicLGP<IrisInput> as GeneticAlgorithm>::init_population(SIZE, MAX_INSTRUCTIONS, &inputs);
 *
 *        assert!(population.len() == SIZE);
 *
 *        for individual in population {
 *            assert!(individual.instructions.len() <= SIZE)
 *        }
 *
 *        Ok(())
 *    }
 *
 *    #[tokio::test]
 *    async fn given_iris_dataset_when_csv_is_read_then_rows_are_deserialized_as_structs(
 *    ) -> Result<(), Box<dyn error::Error>> {
 *        let ContentFilePair(content, _) = get_iris_content().await?;
 *        assert_ne!(content.len(), 0);
 *
 *        let content_bytes = content.as_bytes();
 *
 *        let mut reader = csv::ReaderBuilder::new()
 *            .has_headers(false)
 *            .from_reader(content_bytes);
 *
 *        let data = reader.deserialize();
 *        let mut count = 0;
 *
 *        for result in data {
 *            let _record: IrisInput = result?;
 *            count += 1;
 *        }
 *
 *        assert_ne!(count, 0);
 *
 *        Ok(())
 *    }
 *
 *    #[tokio::test]
 *    async fn given_iris_dataset_when_csv_path_is_provided_then_collection_of_iris_structs_are_returned(
 *    ) -> Result<(), Box<dyn error::Error>> {
 *        let ContentFilePair(_, tmpfile) = get_iris_content().await?;
 *        let inputs = <BasicLGP<IrisInput> as GeneticAlgorithm>::load_inputs(tmpfile.path());
 *        assert_ne!(inputs.len(), 0);
 *        Ok(())
 *    }
 *}
 */
