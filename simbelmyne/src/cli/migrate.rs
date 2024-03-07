use crate::evaluate::tuner::EvalWeights;
use crate::evaluate::new_tuner::EvalWeights as NewEvalWeights;
use tuner::Tune;

pub fn run_migrate() {
    let weights = EvalWeights::default().weights();
    let new_weights = NewEvalWeights::from(weights);
    
    println!("{new_weights}");
}
