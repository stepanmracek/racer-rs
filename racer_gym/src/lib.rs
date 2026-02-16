use pyo3::prelude::*;

#[pyclass(unsendable)]
struct Environment {
    env: racer_logic::environment::Environment,
}

#[pyclass]
#[derive(Clone, Copy)]
pub struct Action {
    #[pyo3(get, set)]
    pub steer: f32,
    #[pyo3(get, set)]
    pub throttle: f32,
}

impl Action {
    fn to_racer_action(self) -> racer_logic::environment::Action {
        racer_logic::environment::Action::new(self.steer, self.throttle)
    }
}

#[pymethods]
impl Action {
    #[new]
    fn new(steer: f32, throttle: f32) -> Self {
        Self { steer, throttle }
    }
}

#[pyclass]
#[derive(Clone)]
enum Goal {
    ReachFinish,
    BackToTrack,
}

impl Goal {
    fn to_racer_goal(&self) -> Box<dyn racer_logic::goal::Goal> {
        match self {
            Goal::ReachFinish => Box::new(racer_logic::goal::ReachFinish::default()),
            Goal::BackToTrack => Box::new(racer_logic::goal::BackToTrack::default()),
        }
    }
}

#[pymethods]
impl Environment {
    #[new]
    #[pyo3(signature = (seed=0, car_count=1, off_track_prob=0.0,  goal=Goal::ReachFinish, track_width=42.0))]
    pub fn new(
        seed: Option<u64>,
        car_count: usize,
        off_track_prob: f32,
        goal: Goal,
        track_width: f32,
    ) -> Self {
        Self {
            env: racer_logic::environment::EnvironmentBuilder::default()
                .with_seed(seed)
                .with_off_track_prob(off_track_prob)
                .with_goal(goal.to_racer_goal())
                .with_track_width(track_width)
                .build(car_count)
                .unwrap(),
        }
    }

    pub fn step(&mut self, actions: Vec<Action>) -> (Vec<Vec<f32>>, f32, bool) {
        let actions: Vec<racer_logic::environment::Action> = actions
            .iter()
            .map(|action| action.to_racer_action())
            .collect();
        let outcome = &self.env.step(&actions, true)[0];

        (self.observations(), outcome.reward, outcome.terminated)
    }

    fn observations(&self) -> Vec<Vec<f32>> {
        self.env
            .observations
            .iter()
            .map(|o| Vec::from(o.clone()))
            .collect()
    }

    #[pyo3(signature = (seed=0, car_count=1, off_track_prob=0.0, goal=Goal::ReachFinish, track_width=42.0))]
    pub fn reset(
        &mut self,
        seed: Option<u64>,
        car_count: usize,
        off_track_prob: f32,
        goal: Goal,
        track_width: f32,
    ) -> Vec<Vec<f32>> {
        self.env = racer_logic::environment::EnvironmentBuilder::default()
            .with_seed(seed)
            .with_off_track_prob(off_track_prob)
            .with_goal(goal.to_racer_goal())
            .with_track_width(track_width)
            .build(car_count)
            .unwrap();
        self.observations()
    }
}

#[pymodule]
fn racer_gym(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Action>()?;
    m.add_class::<Goal>()?;
    m.add_class::<Environment>()?;
    Ok(())
}
