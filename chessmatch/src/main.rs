use std::{collections::HashMap, path::Path};
use chess::board::Board;
use serde::Deserialize;
use serde_yaml;

// So, how this is going to work:
// 1. Need a simple UCI serializer/deserializer. Not necessarily saying we need
// to use serde for this, but I can probably pull out some shared logic here 
// that gets used for this _and_ simbelmyne's UCI, right?
//
// 2. Need a simple loop that checks both engines for output. If it finds a 
// `bestmove` command, then play the move to our board, and send it over to 
// the other engine.
//
// 3. Need a simple UI (can mostly re-use the ratatui stuff for this) to
// render a Board, (should I also pull this out? I feel like i've used it in 
// so many places by now...). Namely, we render the board, and the accumulated
// info we get from the engines. Down the line, we'll only render this UI when
// passed a particular flag (chessmatch --graphical, or something similar)

#[derive(Debug, Clone, Deserialize)]
struct EngineConfig {
    name: String,
    command: String,
    depth: Option<usize>,
    time: Option<usize>,
    nodes: Option<usize>,
    options: HashMap<String, String>
}

#[derive(Debug, Clone, Deserialize)]
struct Config {
    engines: Vec<EngineConfig>
}

fn main() {
    let board = Board::new();

    let config_path = Path::new("./match.yaml");
    let config = std::fs::read_to_string(config_path).unwrap();
    let config: Config = serde_yaml::from_str(&config).unwrap();

    println!("{:?}", config);
}
