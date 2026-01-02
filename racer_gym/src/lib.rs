use pyo3::prelude::*;

#[pyclass(unsendable)]
struct Environment {
    env: racer_logic::environment::Environment,
}

#[pyclass]
#[derive(Clone)]
enum Goal {
    ReachFinish,
    BackToTrack,
}

impl Goal {
    fn to_racer_goal(&self) -> Box<dyn racer_logic::environment::Goal> {
        match self {
            Goal::ReachFinish => Box::new(racer_logic::environment::ReachFinish::default()),
            Goal::BackToTrack => Box::new(racer_logic::environment::BackToTrack::default()),
        }
    }
}

#[pymethods]
impl Environment {
    #[new]
    #[pyo3(signature = (seed=0, goal=Goal::ReachFinish))]
    pub fn new(seed: Option<u64>, goal: Goal) -> Self {
        Self {
            env: racer_logic::environment::Environment::new(seed, 1, goal.to_racer_goal()),
        }
    }

    pub fn step(&mut self, steer: f32, throttle: f32) -> (Vec<f32>, f32, bool) {
        let action = racer_logic::environment::Action { steer, throttle };
        let outcome = self.env.step(&[action], true);

        let observation: Vec<f32> = self.env.observations[0].clone().into();
        (observation, outcome[0].reward, outcome[0].finished)
    }

    fn observation(&self) -> Vec<f32> {
        self.env.observations[0].clone().into()
    }

    #[pyo3(signature = (seed=0, goal=Goal::ReachFinish))]
    pub fn reset(&mut self, seed: Option<u64>, goal: Goal) -> Vec<f32> {
        self.env = racer_logic::environment::Environment::new(seed, 1, goal.to_racer_goal());
        self.observation()
    }
}

#[pymodule]
fn racer_gym(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Goal>()?;
    m.add_class::<Environment>()?;
    Ok(())
}
