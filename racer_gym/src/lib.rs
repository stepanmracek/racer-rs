use pyo3::prelude::*;
use racer_logic::environment::{Action, Environment};

#[pyfunction]
fn env_step(seed: Option<u64>) -> PyResult<f32> {
    let mut env = Environment::new(seed);
    let outcome = env.step(
        &Action {
            steer: 0.0,
            throttle: 1.0,
        },
        true,
    );
    Ok(outcome.reward)
}

#[pymodule]
fn racer_gym(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(env_step, m)?)?;
    Ok(())
}
