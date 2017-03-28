#[macro_use]
extern crate serde_derive;
extern crate toml;

type Player = String;

use std::collections::HashMap;
use std::io::Read;
use std::env::args;
use std::fs::File;

const BASE_MMR: f64 = 1500.0;
const K_FACTOR_BASE: f64 = 800.0;
const WIN_VALUE: f64 = 1.0;
const LOSE_VALUE: f64 = 0.0;

#[derive(Deserialize, Debug)]
struct Game {
    winners: Vec<Player>,
    losers: Vec<Player>,
}

#[derive(Deserialize, Debug)]
struct EloFile {
    players: Vec<Player>,
    games: Vec<Game>,
}

#[derive(Debug)]
struct PlayerStats {
    mmr: f64,
    games_played: usize,
    games_won: usize,
}
impl PlayerStats {
    fn new() -> PlayerStats {
        PlayerStats {
            mmr: BASE_MMR,
            games_played: 0,
            games_won: 0,
        }
    }
}

fn main() {
    let file_path = args().nth(1).expect("Provide File Path");
    let file = {
        let mut buf = String::new();
        File::open(file_path)
            .expect("File Not Openable")
            .read_to_string(&mut buf)
            .expect("File Not Readable");
        buf
    };
    let file: EloFile = toml::from_str(&file).expect("File Not TOML");

    let mut players: HashMap<String, PlayerStats> = HashMap::new();
    for player in file.players {
        players.insert(player, PlayerStats::new());
    }

    for game in file.games.iter() {
        for (win, player) in game.winners
            .iter()
            .map(|n| (true, n))
            .chain(game.losers.iter().map(|n| (false, n))) {

            players.get_mut(player).expect(&format!("Unknown Player: {}", player)).games_played +=
                1;
            players.get_mut(player).expect(&format!("Unknown Player: {}", player)).games_won +=
                if win { 1 } else { 0 };
            let old_mmr =
                players.get_mut(player).expect(&format!("Unknown Player: {}", player)).mmr.clone();

            let k = K_FACTOR_BASE /
                    players.get_mut(player)
                .expect(&format!("Unknown Player: {}", player))
                .games_played as f64;

            for enemy in if win {
                game.losers.iter()
            } else {
                game.winners.iter()
            } {
                let enemy_mmr =
                    players.get(enemy).expect(&format!("Unknown Player: {}", player)).clone().mmr;
                let e = 1.0 / (1.0 + 10.0f64.powf((enemy_mmr - old_mmr) / 400.0));
                let ref mut mmr =
                    players.get_mut(player).expect(&format!("Unknown Player: {}", player)).mmr;
                let w = if win { WIN_VALUE } else { LOSE_VALUE };
                *mmr = old_mmr + ((k * (w - e)) / game.losers.len() as f64);
            }
        }
    }
    let mut players = players.iter().collect::<Vec<_>>();
    players.sort_by_key(|player| player.1.mmr as usize);
    players.reverse();
    println!("{:#?}", players);
}
