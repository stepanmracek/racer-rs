use racer_logic::{controller::Controller, environment::Action, observation::Observation};
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use std::collections::{HashMap, VecDeque};

pub enum ActionSelectionStrategy {
    Greedy,
    Stochastic,
}

pub struct OnnxController {
    session: ort::session::Session,
    output_to_action: HashMap<usize, (f32, f32)>,
    strategy: ActionSelectionStrategy,
}

pub struct OnnxWithHistoryController {
    session: ort::session::Session,
    output_to_action: HashMap<usize, (f32, f32)>,
    strategy: ActionSelectionStrategy,
    past_observations: VecDeque<Vec<f32>>,
    history_len: usize,
}

fn create_output_to_action() -> HashMap<usize, (f32, f32)> {
    HashMap::from([
        (0, (1.0, 1.0)),
        (1, (0.0, 1.0)),
        (2, (-1.0, 1.0)),
        (3, (1.0, 0.0)),
        (4, (0.0, 0.0)),
        (5, (-1.0, 0.0)),
        (6, (1.0, -1.0)),
        (7, (0.0, -1.0)),
        (8, (-1.0, -1.0)),
    ])
}

fn index_to_action(output_to_action: &HashMap<usize, (f32, f32)>, index: usize) -> Action {
    let action = output_to_action[&index];
    Action::new(action.0, action.1)
}

fn sample_action(output_to_action: &HashMap<usize, (f32, f32)>, probs: &[f32]) -> Action {
    let dist = WeightedIndex::new(probs).unwrap();
    let index = dist.sample(&mut rand::rng());
    index_to_action(output_to_action, index)
}

fn greedy_action(output_to_action: &HashMap<usize, (f32, f32)>, probs: &[f32]) -> Action {
    let index = probs
        .iter()
        .enumerate()
        .reduce(|acc, val| if val.1 > acc.1 { val } else { acc })
        .map(|(index, _val)| index)
        .unwrap_or_default();
    index_to_action(output_to_action, index)
}

impl OnnxController {
    pub fn new(path: &str, strategy: ActionSelectionStrategy) -> Self {
        OnnxController {
            session: ort::session::Session::builder()
                .unwrap()
                .commit_from_file(path)
                .unwrap(),
            output_to_action: create_output_to_action(),
            strategy,
        }
    }

    fn inference(&mut self, obs: Vec<f32>) -> Vec<f32> {
        let input_tensor = ort::value::Tensor::from_array(([1, obs.len()], obs)).unwrap();
        let input = ort::inputs!["input" => input_tensor];
        let session_output = self.session.run(input).unwrap();
        let probs = session_output["output"].try_extract_array::<f32>().unwrap();
        probs.into_iter().cloned().collect()
    }
}

impl Controller for OnnxController {
    fn control(&mut self, o: &Observation) -> Action {
        let obs_vec: Vec<f32> = o.clone().into();
        let probs = self.inference(obs_vec);

        match self.strategy {
            ActionSelectionStrategy::Greedy => greedy_action(&self.output_to_action, &probs),
            ActionSelectionStrategy::Stochastic => sample_action(&self.output_to_action, &probs),
        }
    }

    fn reset(&mut self) {}
}

impl OnnxWithHistoryController {
    pub fn new(path: &str, strategy: ActionSelectionStrategy, history_len: usize) -> Self {
        OnnxWithHistoryController {
            session: ort::session::Session::builder()
                .unwrap()
                .commit_from_file(path)
                .unwrap(),
            output_to_action: create_output_to_action(),
            strategy,
            past_observations: VecDeque::new(),
            history_len,
        }
    }

    fn inference(&mut self, obs: Vec<f32>) -> Vec<f32> {
        self.past_observations.push_back(obs);
        while self.past_observations.len() > self.history_len {
            self.past_observations.pop_front();
        }

        let observations_for_inference: Vec<Vec<f32>> =
            if self.past_observations.len() < self.history_len {
                let first_obs = self.past_observations.front().unwrap();
                let num_to_pad = self.history_len - self.past_observations.len();
                let mut padded_obs: Vec<Vec<f32>> =
                    std::iter::repeat_n(first_obs.clone(), num_to_pad).collect();
                padded_obs.extend(self.past_observations.iter().cloned());
                padded_obs
            } else {
                self.past_observations.iter().cloned().collect()
            };

        let history_size = self.history_len;
        let obs_len = self.past_observations.back().unwrap().len();
        let mut input_vec: Vec<f32> = Vec::with_capacity(history_size * obs_len);

        // Transpose from (history_size, obs_len) to (obs_len, history_size)
        for i in 0..obs_len {
            for j in 0..history_size {
                input_vec.push(observations_for_inference[j][i]);
            }
        }

        let input_tensor =
            ort::value::Tensor::from_array(([1, obs_len, history_size], input_vec)).unwrap();
        let input = ort::inputs!["input" => input_tensor];
        let session_output = self.session.run(input).unwrap();
        let probs = session_output["output"].try_extract_array::<f32>().unwrap();
        probs.into_iter().cloned().collect()
    }
}

impl Controller for OnnxWithHistoryController {
    fn control(&mut self, o: &Observation) -> Action {
        let obs_vec: Vec<f32> = o.clone().into();
        let probs = self.inference(obs_vec);

        match self.strategy {
            ActionSelectionStrategy::Greedy => greedy_action(&self.output_to_action, &probs),
            ActionSelectionStrategy::Stochastic => sample_action(&self.output_to_action, &probs),
        }
    }

    fn reset(&mut self) {
        self.past_observations.clear();
    }
}
