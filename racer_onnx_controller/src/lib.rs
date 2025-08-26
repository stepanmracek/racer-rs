use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use std::collections::HashMap;

use racer_logic::{
    controller::Controller,
    environment::{Action, Observation},
};

pub enum ActionSelectionStrategy {
    Greedy,
    Stochastic,
}

pub struct OnnxController {
    session: ort::session::Session,
    output_to_action: HashMap<usize, (f32, f32)>,
    strategy: ActionSelectionStrategy,
}

impl OnnxController {
    pub fn new(path: &str, strategy: ActionSelectionStrategy) -> Self {
        OnnxController {
            session: ort::session::Session::builder()
                .unwrap()
                .commit_from_file(path)
                .unwrap(),
            output_to_action: HashMap::from([
                (0, (1.0, 1.0)),
                (1, (0.0, 1.0)),
                (2, (-1.0, 1.0)),
                (3, (1.0, 0.0)),
                (4, (0.0, 0.0)),
                (5, (-1.0, 0.0)),
                (6, (1.0, -1.0)),
                (7, (0.0, -1.0)),
                (8, (-1.0, -1.0)),
            ]),
            strategy,
        }
    }
}

impl OnnxController {
    fn inference(&mut self, obs: Vec<f32>) -> Vec<f32> {
        let input_tensor = ort::value::Tensor::from_array(([1, obs.len()], obs)).unwrap();
        let input = ort::inputs!["input" => input_tensor];
        let session_output = self.session.run(input).unwrap();
        let probs = session_output["output"].try_extract_array::<f32>().unwrap();
        probs.into_iter().cloned().collect()
    }

    fn index_to_action(&self, index: usize) -> Action {
        let action = self.output_to_action[&index];
        Action {
            steer: action.0,
            throttle: action.1,
        }
    }

    fn sample_action(&self, probs: &[f32]) -> Action {
        let dist = WeightedIndex::new(probs).unwrap();
        let index = dist.sample(&mut rand::rng());
        self.index_to_action(index)
    }

    fn greedy_action(&self, probs: &[f32]) -> Action {
        let index = probs
            .iter()
            .enumerate()
            .reduce(|acc, val| if val.1 > acc.1 { val } else { acc })
            .map(|(index, _val)| index)
            .unwrap_or_default();
        self.index_to_action(index)
    }
}

impl Controller for OnnxController {
    fn control(&mut self, o: &Observation) -> Action {
        let obs_vec: Vec<f32> = o.clone().into();
        let probs = self.inference(obs_vec);

        match self.strategy {
            ActionSelectionStrategy::Greedy => self.greedy_action(&probs),
            ActionSelectionStrategy::Stochastic => self.sample_action(&probs),
        }
    }
}
