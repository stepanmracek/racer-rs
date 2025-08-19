use crate::controller::{Control, Controller};

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
    fn control(
        &mut self,
        velocity: f32,
        steering_angle: f32,
        wheels_on_track: &[bool; 4],
        sensor_readings: &[Option<f32>],
    ) -> super::Control {
        let mut vec = vec![velocity, steering_angle];
        vec.extend(wheels_on_track.iter().map(|&w| if w { 1.0 } else { 0.0 }));
        vec.extend(sensor_readings.iter().map(|r| r.unwrap_or(200.0))); // TODO: use SENSOR_REACH here
        // TODO: scale input vec values!!!

        let input_vec = ort::value::Tensor::from_array(([1, vec.len()], vec)).unwrap();
        let input = ort::inputs!["input" => input_vec];
        let session_output = self.session.run(input).unwrap();
        let output = session_output["output"].try_extract_array::<f32>().unwrap();

        Control {
            steer: output[[0, 0]],
            throttle: output[[0, 1]],
        }
    }
}
