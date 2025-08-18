pub fn format_time(time: f64) -> String {
    let time = (time * 100.0) as u64;
    let hundrets = time % 100;
    let seconds = (time / 100) % 60;
    let minutes = time / 6000;
    format!("{minutes:02}:{seconds:02}:{hundrets:02}")
}
