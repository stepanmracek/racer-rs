use ndarray::Array2;
use ndarray_rand::RandomExt;
use ndarray_rand::rand_distr::{Normal, Uniform};
use racer_logic::{
    controller::Controller,
    environment::{Action, Observation},
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Layer {
    pub w: Array2<f32>,
    pub b: Array2<f32>,
}

impl Layer {
    pub fn new(w: Array2<f32>, b: Array2<f32>) -> Self {
        Self { w, b }
    }

    fn random(in_dim: usize, out_dim: usize) -> Self {
        let std_dev = (2.0 / (in_dim + out_dim) as f32).sqrt();
        let w = Array2::random((out_dim, in_dim), Normal::new(0.0f32, std_dev).unwrap());
        let b = Array2::random((out_dim, 1), Uniform::new(-1.0, 1.0).unwrap());
        Self { w, b }
    }

    fn forward(&self, input: &Array2<f32>) -> Array2<f32> {
        let z = self.w.dot(input) + &self.b;
        z.mapv(f32::tanh)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct NdarrayController {
    pub layers: Vec<Layer>,
}

impl NdarrayController {
    pub fn new(layers: Vec<Layer>) -> Self {
        Self { layers }
    }

    pub fn random() -> Self {
        NdarrayController {
            layers: vec![Layer::random(26, 64), Layer::random(64, 2)],
        }
    }

    fn forward(&self, input: Array2<f32>) -> Array2<f32> {
        self.layers
            .iter()
            .fold(input, |acc, layer| layer.forward(&acc))
    }

    pub fn save(&self, path: &str) {
        let json = serde_json::to_string(self).unwrap();
        std::fs::write(path, json).unwrap();
    }

    pub fn load(path: &str) -> Self {
        let json = std::fs::read_to_string(path).unwrap();
        serde_json::from_str(&json).unwrap()
    }
}

fn vec_to_array2(vec: Vec<f32>) -> Array2<f32> {
    Array2::from_shape_vec((vec.len(), 1), vec).unwrap()
}

impl Controller for NdarrayController {
    fn control(&mut self, o: &Observation) -> Action {
        let input = vec_to_array2(o.clone().into());
        let output = self.forward(input);

        Action {
            steer: output[[0, 0]],
            throttle: output[[1, 0]],
        }
    }

    fn reset(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward() {
        let ctrl = NdarrayController::random();
        let input: Vec<f32> = (0..26).map(|_| 1.0).collect();
        let input = vec_to_array2(input);
        let output = ctrl.forward(input);
        assert_eq!(output.shape(), [2, 1]);
    }

    #[test]
    fn serde() {
        let ctrl = NdarrayController::random();
        ctrl.save("/tmp/test_ndarray_ctrl.json");

        let loaded = NdarrayController::load("/tmp/test_ndarray_ctrl.json");
        assert_eq!(ctrl, loaded);

        std::fs::remove_file("/tmp/test_ndarray_ctrl.json").unwrap();
    }
}
