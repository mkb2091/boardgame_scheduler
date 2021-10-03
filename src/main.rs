fn main() {
    let mut stack = vec![boardgame_scheduler::State::new()];
    let mut max_get_players_played_count = 0;
    let start = std::time::Instant::now();
    let mut n = 1000;
	let mut counter = 0;
    loop {
        let mut min_depth = stack.len();
        let mut max_depth = stack.len();
		let mut loop_start = std::time::Instant::now();
		
        for _ in 0..n {
            let mut state = stack.pop().unwrap();
            if let Ok(res) = state.step() {
                let played = state.get_players_played_count();
                if played > max_get_players_played_count {
                    max_get_players_played_count = played;
                }
                stack.push(state);
                stack.push(res.unwrap());
            }
            min_depth = min_depth.min(stack.len());
            max_depth = max_depth.max(stack.len());
        }
        counter += n;
		let last = stack[min_depth - 1];
		let last = stack.last().unwrap();
		let mut pretty = String::new();
		last.format_schedule(&mut pretty);
		println!("\n{:?}\n{}\n", 
            last,
			pretty);
        println!(
            "Rate: {}, Stack size: {}, min_depth: {}, max_depth: {}, max_get_players_played_count: {}",
            counter as f32 / start.elapsed().as_secs_f32().max(0.1),
            stack.len(),
            min_depth,
			max_depth,
            max_get_players_played_count,
        );
		n = (((n as f64) / loop_start.elapsed().as_secs_f64().max(0.1)) as u64).max(1000)/2;
    }
}
