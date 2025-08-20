use crate::{
    controller::Controller,
    environment::{Action, Observation, SENSOR_REACH},
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
        let mut vec = vec![o.velocity, o.steering_angle];
        vec.extend(o.wheels_on_track.iter().map(|&w| if w { 1.0 } else { 0.0 }));
        vec.extend(
            o.sensors
                .distances
                .iter()
                .map(|r| r.unwrap_or(SENSOR_REACH)),
        );

        let input_vec = ort::value::Tensor::from_array(([1, vec.len()], vec)).unwrap();
        let input = ort::inputs!["input" => input_vec];
        let session_output = self.session.run(input).unwrap();
        let output = session_output["output"].try_extract_array::<f32>().unwrap();

        Action {
            steer: output[[0, 0]],
            throttle: output[[0, 1]],
        }
    }
}
