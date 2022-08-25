use bincode::Options;
use std::io::Read;
use std::io::Write;
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

fn bstep() -> Result<(), Box<dyn std::error::Error>> {
    let max_cache = 100_000;
    let cache_diff_limit = 10;

    let state = boardgame_scheduler::State::new();
    let available_count = state.get_available_count() as usize;

    let bincode_ops = bincode::DefaultOptions::new().with_fixint_encoding();
    let serialized_size = bincode_ops.serialized_size(&state)?;

    let mut files = Vec::new();
    for i in 0..available_count + 1 {
        let file = std::fs::File::create(format!("tmp{}.txt", i))?;
        let buf = std::io::BufWriter::new(file);
        files.push((vec![], buf));
    }
    files[available_count].0.push(state);
    let start = std::time::Instant::now();
    let mut n = 100000;
    let mut counter = 0;
    let mut i = 0;
    let mut loop_start = std::time::Instant::now();

    while let Some((mut stack, mut write_buf)) = files.pop() {
        let current_count = files.len();
        println!("Current_count: {}", current_count);
        write_buf.flush()?;
        let mut file_buf =
            if let Ok(file) = std::fs::File::open(format!("tmp{}.txt", current_count)) {
                let buf_read = std::io::BufReader::new(file);
                Some(buf_read)
            } else {
                None
            };

        let mut buf = vec![0; serialized_size as usize];

        let mut max_players_placed = 0;
        let mut best = None;

        'inner: loop {
            let mut state = if let Some(state) = stack.pop() {
                state
            } else if file_buf
                .as_mut()
                .and_then(|file| file.read_exact(&mut buf).ok())
                .is_some()
            {
                match bincode_ops.deserialize(&buf) {
                    Ok(state) => state,
                    Err(err) => {
                        println!("Error decoding: {:?}", err);
                        break 'inner;
                    }
                }
            } else {
                let _ = std::fs::remove_file(format!("tmp{}.txt", current_count));
                break 'inner;
            };
            counter += 1;
            let players_placed = state.get_players_played_count();
            if players_placed > max_players_placed {
                max_players_placed = players_placed;
                best = Some(state);
            }

            let mut callback = |state: &boardgame_scheduler::State| {
                //if state.find_hidden_singles().is_err() {
                //    return;
                //}
                let available_count = state.get_available_count() as usize;
                assert!(available_count <= current_count);
                if available_count == current_count {
                    stack.push(*state);
                } else {
                    let target = &mut files[available_count];
                    if (available_count + cache_diff_limit < current_count)
                        || target.0.len() > max_cache
                    {
                        bincode_ops.serialize_into(&mut target.1, state).unwrap();
                    } else {
                        target.0.push(*state);
                    }
                }
                //let score = 24 * 6 * 6 - score;
            };
            state.bstep(&mut callback);

            //println!("Allocatins: {}", allocations);

            i += 1;
            if i > n {
                println!(
                    "Rate: {}, current available_count: {}",
                    counter as f32 / start.elapsed().as_secs_f32().max(0.1),
                    current_count
                );

                n = (((n as f64) / loop_start.elapsed().as_secs_f64().max(0.1)) as u64).max(1000)
                    / 2;
                loop_start = std::time::Instant::now();
                i = 0;
            }
        }
        if let Some(best) = best {
            println!("Max players placed: {}", max_players_placed);
            println!("State: {:}", best);
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    bstep()
}
