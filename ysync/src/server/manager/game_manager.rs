use std::collections::VecDeque;

use bevy_utils::HashMap;

use crate::Game;

#[derive(Debug)]
pub struct GameManager {
    games: Vec<Game>,
    active_games: Vec<u16>,
    free_ids: VecDeque<u16>,
}

impl GameManager {
    pub fn new() -> GameManager {
        GameManager {
            games: Vec::new(),
            active_games: Vec::new(),
            free_ids: VecDeque::new(),
        }
    }
    pub fn add_game(&mut self, game: &mut Game) -> bool {
        if let Some(existing_game) = self.games.iter().find(|g| g.host_id == game.host_id) {
            if let Some(_) = self.active_games.iter().find(|a| **a == existing_game.game_id) {
                return false;
            }
        }
        let mut new_id: bool = false;
        let id = match self.free_ids.pop_front() {
            Some(free_id) => free_id,
            None => {
                new_id = true;
                self.games.len() as u16
            }
        };
        game.game_id = id;
        match new_id {
            true => self.games.push(game.clone()),
            false => self.games[id as usize] = game.clone(),
        }
        self.active_games.push(id);
        true
    }
    pub fn remove_game(&mut self, host_id: u16) -> Option<u16> {
        let game_id = self.games.iter().find(|c| c.host_id == host_id)?.game_id;
        self.active_games.retain(|a| *a != game_id);
        self.free_ids.push_back(game_id);
        Some(game_id)
    }
    pub fn add_client_to_game(&mut self, client_id: u16, game_id: u16) {
        self.games.iter_mut().find(|g| g.game_id == game_id).unwrap().clients.push(client_id);
    }
    pub fn remove_client_from_game(&mut self, client_id: u16) -> u16 {
        self.games.iter_mut().find(|g| g.clients.contains(&client_id)).map(|g| {
            g.clients.retain(|c| *c != client_id);
            g.game_id
        }).unwrap()
    }
    pub fn get_game(&self, game_id: u16) -> Game {
        self.games[game_id as usize].clone()
    }
    pub fn get_games(&self) -> HashMap<u16, Game> {
        self.active_games.iter().map(|id| (*id, self.games[*id as usize].clone())).collect()
    }
}
