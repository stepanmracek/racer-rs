use pyo3::prelude::*;

#[pyclass(unsendable)]
struct Environment {
    env: racer_logic::environment::Environment,
}

#[pymethods]
impl Environment {
    #[new]
    #[pyo3(signature = (seed=0, off_track_prob=0.0))]
    pub fn new(seed: Option<u64>, off_track_prob: f32) -> Self {
        Self {
            env: racer_logic::environment::Environment::new(seed, off_track_prob),
        }
    }

    pub fn step(&mut self, steer: f32, throttle: f32) -> (Vec<f32>, f32, bool) {
        let action = racer_logic::environment::Action { steer, throttle };
        let outcome = self.env.step(&action, true);

        let observation: Vec<f32> = self.env.observation.clone().into();
        (observation, outcome.reward, outcome.finished)
    }

    fn observation(&self) -> Vec<f32> {
        self.env.observation.clone().into()
    }

    #[pyo3(signature = (seed=0, off_track_prob=0.0))]
    pub fn reset(&mut self, seed: Option<u64>, off_track_prob: f32) -> Vec<f32> {
        self.env = racer_logic::environment::Environment::new(seed, off_track_prob);
        self.observation()
    }
}

#[pymodule]
fn racer_gym(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Environment>()?;
    Ok(())
}
