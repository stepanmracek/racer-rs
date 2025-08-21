use crate::{
    controller::Controller,
    environment::{Action, Observation},
};

pub struct OnnxController {
    session: ort::session::Session,
}

impl OnnxController {
    pub fn new(path: &str) -> Self {
        OnnxController {
            session: ort::session::Session::builder()
                .unwrap()
                .commit_from_file(path)
                .unwrap(),
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

        Action {
            steer: output[[0, 0]],
            throttle: output[[0, 1]],
        }
    }
}
