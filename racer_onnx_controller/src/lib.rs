use std::collections::HashMap;

use racer_logic::{
    controller::Controller,
    environment::{Action, Observation},
};

pub struct OnnxController {
    session: ort::session::Session,
    output_to_action: HashMap<usize, (f32, f32)>,
}

impl OnnxController {
    pub fn new(path: &str) -> Self {
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
        }
    }
}

impl Controller for OnnxController {
    fn control(&mut self, o: &Observation) -> Action {
        let obs_vec: Vec<f32> = o.clone().into();
        let input_tensor = ort::value::Tensor::from_array(([1, obs_vec.len()], obs_vec)).unwrap();
        let input = ort::inputs!["input" => input_tensor];
        let session_output = self.session.run(input).unwrap();
        let output = session_output["output"].try_extract_array::<f32>().unwrap();

        let mut max_index = 0;
        let mut max_val = 0.0;
        for (index, &val) in output.into_iter().enumerate() {
            if val > max_val {
                max_val = val;
                max_index = index;
            }
        }

        let action = self.output_to_action[&max_index];

        Action {
            steer: action.0,
            throttle: action.1,
        }
    }
}
