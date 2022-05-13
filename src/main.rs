fn main() {
    let mut stack = vec![boardgame_scheduler::State::new()];
    let mut index = 0;

    let mut max_get_players_played_count = 0;
    let start = std::time::Instant::now();
    let mut n = 1000;
    let mut counter = 0;

    loop {
        let mut min_depth = index + 1;
        let mut max_depth = index + 1;
        let mut loop_start = std::time::Instant::now();
        for _ in 0..n {
            while stack.len() < index + 2 {
                stack.push(boardgame_scheduler::State::new());
            }

            let (_, data) = stack.split_at_mut(index);
            let (state, data) = data.split_first_mut().unwrap();
            let state2 = data.first_mut().unwrap();
            if let Ok(res) = state.step(state2) {
                let played = state.get_players_played_count();
                if played > max_get_players_played_count {
                    max_get_players_played_count = played;
                }
                let _ = res.unwrap();
                index += 1;
            } else {
                index = index.checked_sub(1).unwrap();
            }

            min_depth = min_depth.min(index + 1);
            max_depth = max_depth.max(index + 1);
        }

        counter += n;
        let last = stack[min_depth - 1];
        let last = stack[index];
        let mut pretty = String::new();
        last.format_schedule(&mut pretty);
        println!("\n{:?}\n{}\n", last, pretty);
        println!(
            "Rate: {}, Stack size: {}, min_depth: {}, max_depth: {}, max_get_players_played_count: {}",
            counter as f32 / start.elapsed().as_secs_f32().max(0.1),
            stack.len(),
            min_depth,
			max_depth,
            max_get_players_played_count,
        );
        n = (((n as f64) / loop_start.elapsed().as_secs_f64().max(0.1)) as u64).max(1000) / 2;
    }
}
