fn step() {
    let mut freelist = Vec::<Box<boardgame_scheduler::State>>::new();
    let mut stack = vec![Box::new(boardgame_scheduler::State::new())];

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
            let mut state2 = freelist
                .pop()
                .unwrap_or_else(|| Box::new(boardgame_scheduler::State::new()));

            if let Ok(res) = state.step(&mut state2) {
                let played = state.get_players_played_count();
                if played > max_get_players_played_count {
                    max_get_players_played_count = played;
                }

                let _ = res.unwrap();
                stack.push(state);
                stack.push(state2);
            } else {
                freelist.push(state);
                freelist.push(state2);
            }

            min_depth = min_depth.min(stack.len());
            max_depth = max_depth.max(stack.len());
        }

        counter += n;
        let last = &stack[min_depth - 1];
        let last = &stack.last().unwrap();
        println!("\n{:?}\n{}\n", last, last);
        println!(
            "Rate: {}, Stack size: {}, allocations: {}, min_depth: {}, max_depth: {}, max_get_players_played_count: {}",
            counter as f32 / start.elapsed().as_secs_f32().max(0.1),
            stack.len(),
            stack.len() + freelist.len(),
            min_depth,
			max_depth,
            max_get_players_played_count,
        );
        n = (((n as f64) / loop_start.elapsed().as_secs_f64().max(0.1)) as u64).max(1000) / 2;
    }
}

fn bstep() {
    let mut freelist = Vec::<Box<boardgame_scheduler::State>>::new();
    let mut stack = vec![vec![Box::new(boardgame_scheduler::State::new())]];

    let start = std::time::Instant::now();
    let mut n = 1000;
    let mut counter = 0;
    let mut i = 0;
    let mut loop_start = std::time::Instant::now();
    let mut allocations = 1;
    while let Some(mut states) = stack.pop() {
        let mut state = states.pop();
        if !states.is_empty() {
            stack.push(states);
        }

        if let Some(mut state) = state {
            let mut allocator = || {
                freelist.pop().unwrap_or_else(|| {
                    allocations += 1;
                    Box::new(boardgame_scheduler::State::new())
                })
            };
            let mut callback = |state: Box<boardgame_scheduler::State>| {
                let available_count = state.get_available_count() as usize;
                let played_count = state.get_players_played_count() as usize;
                let score = available_count - played_count * 2;
                while stack.len() <= score {
                    stack.push(Vec::new());
                }

                stack[score].push(state);
                //stack.sort_unstable_by_key(|s| s.get_available_count());
            };
            state.bstep(&mut allocator, &mut callback);
            //println!("Allocatins: {}", allocations);
            n = 10000;

            i += 1;
            if i > n {
                counter += i;
                println!("\n{:?}\n{}\n", state, state);

                println!(
                    "Rate: {}, allocations: {}, current available_count: {}",
                    counter as f32 / start.elapsed().as_secs_f32().max(0.1),
                    allocations,
                    state.get_available_count()
                );

                n = (((n as f64) / loop_start.elapsed().as_secs_f64().max(0.1)) as u64).max(1000)
                    / 2;
                loop_start = std::time::Instant::now();
                i = 0;
            }

            freelist.push(state);
        }
    }
}

fn main() {
    bstep()
}
