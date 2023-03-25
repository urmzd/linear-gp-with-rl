use crate::core::{characteristics::Reset, input_engine::EnvironmentalFactor, program::Program};
use core::fmt::Debug;
use std::cmp::Ordering;

use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Copy, PartialEq, Display, Serialize, Deserialize)]
pub enum FitnessScore {
    #[display(fmt = "valid: {}", _0)]
    Valid(f64),
    #[display(format = "out-of-bounds")]
    OutOfBounds,
    #[display(format = "not-evaluated")]
    NotEvaluated,
}

impl Reset for FitnessScore {
    fn reset(&mut self) {
        *self = FitnessScore::NotEvaluated
    }
}

impl PartialOrd for FitnessScore {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Valid(a), Self::Valid(b)) => a.partial_cmp(b),
            (Self::Valid(_), _) => Some(Ordering::Greater),
            (_, Self::Valid(_)) => Some(Ordering::Less),
            _ => Some(Ordering::Equal),
        }
    }
}

impl FitnessScore {
    pub fn is_not_evaluated(&self) -> bool {
        match self {
            Self::NotEvaluated => true,
            _ => false,
        }
    }

    pub fn is_invalid(&self) -> bool {
        match self {
            FitnessScore::Valid(_) | FitnessScore::NotEvaluated => false,
            _ => true,
        }
    }

    pub fn unwrap_or(&self, value: f64) -> f64 {
        match self {
            FitnessScore::Valid(fitness_score) => *fitness_score,
            _ => value,
        }
    }

    pub fn unwrap(&self) -> f64 {
        match self {
            FitnessScore::Valid(fitness_score) => *fitness_score,
            _ => unreachable!(),
        }
    }
}

pub trait Fitness<E, P> {
    // Takes a set of parameters, runs the program and updates the item to contain the new fitness.
    fn eval_fitness(item: &mut Program, environment: &mut E, additonal_parameters: &mut P);
}

pub struct FitnessEngine;
