use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::Serialize;
use near_sdk::{log, near_bindgen, BorshStorageKey, AccountId, Promise};

#[derive(Copy, Clone, BorshDeserialize, BorshSerialize, PartialEq, PartialOrd, Eq, Ord, Serialize)]
pub struct Vector2 {
    x: i8,
    y: i8,
    id: i8,
}

impl Vector2 {
    pub fn new(x_position: i8, y_position: i8, _id: i8) -> Vector2 {
        Vector2{x: x_position, y: y_position, id: _id}
    }
}
#[derive(Copy, Clone, BorshDeserialize, BorshSerialize, Serialize)]
pub enum UnitType {
    Orc,
    Mage,
    Skeleton
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Copy, Serialize)]
pub struct Unit {
    unit_type: UnitType,
    level: i8,
    stats: UnitStats,
    current_position: Vector2,
    unit_id: i8,
}

impl Unit {
    pub fn new(unit_level: i8, _unit_type: UnitType, id: i8) -> Self {
        Self {
            level: unit_level,
            stats: UnitStats::new(unit_level, _unit_type),
            unit_type: _unit_type,
            current_position: Vector2 { x: 0, y: 0, id: -1 },
            unit_id: id
        }
    }

    pub fn get_unit_type_by_id(id: i8) -> UnitType {
        match id {
            0 => UnitType::Mage,
            1 => UnitType::Orc,
            2 => UnitType::Skeleton,
            _ => panic!("Unknown unit type")
        }
    }
}

#[derive(Clone, BorshSerialize, BorshDeserialize, Copy, Serialize, Debug)]
pub struct UnitStats {
    hp: f32,
    damage: f32,
    max_hp: f32,
}

impl UnitStats {
    pub fn new(unit_level: i8, unit_type: UnitType) -> UnitStats {
        UnitStats {
            hp: UnitStats::get_unit_hp_by_level(unit_level, unit_type),
            damage: UnitStats::get_unit_damage_by_level(unit_level, unit_type),
            max_hp: UnitStats::get_unit_hp_by_level(unit_level, unit_type),
        }
    }
    
    /// returs max hp of unit based on it's level
    /// # Usage 
    /// * 'level' - level of the Unit
    /// * 'unit_type' - Orc, Skeleton or Mage
    fn get_unit_hp_by_level(level: i8, unit_type: UnitType) -> f32 {
            match unit_type {
                UnitType::Mage => 12.0 + 3.0*(level as f32-1.0),
                UnitType::Orc => match level {
                    1 => 40.0,
                    2 => 42.0,
                    3 => 48.0,
                    4 => 56.0,
                    5 => 64.0,
                    _ => 0.0
                }
                UnitType::Skeleton => match level {
                    1 => 25.0,
                    2 => 27.0,
                    3 => 30.0,
                    4 => 35.0,
                    5 => 40.0,
                    _ => 0.0
                }
            }
        }

    /// returs damage of unit based on it's level
    /// # Usage 
    /// * 'level' - level of the Unit
    /// * 'unit_type' - Orc, Skeleton or Mage
    fn get_unit_damage_by_level(level: i8, unit_type: UnitType) -> f32 {
        match unit_type {
            UnitType::Mage => 8.0 + 2.0*(level as f32-1.0),
            UnitType::Orc => match level {
                1 => 4.0,
                2 => 6.0,
                3 => 6.0,
                4 => 8.0,
                5 => 8.0,
                _ => 0.0
            }
            UnitType::Skeleton => 10.0 + 2.0*(level as f32-1.0)
        }
    }
}

///
/// Implemetation of cell on a board, must contain unit which is on a cell right now
#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, Serialize)]
pub struct Cell {
    pub unit: Option<Unit>,
    pub position: Vector2,
}

impl Cell {
    pub fn new(x:i8, y: i8, id:i8) -> Self {
        Cell{unit: None, position: Vector2::new(x,y, id)}
    }
}

///
/// Implementation of a desk in a game, contrains a map of all cells on a board.
/// 
#[derive(BorshDeserialize, BorshSerialize,BorshStorageKey, Clone, Serialize)]
pub struct Desk {
    cells: Vec<Cell>,
    x_range: i8,
    y_range: i8,
}

impl Desk {
    pub fn new(x:i8, y:i8) -> Self {
        Desk {
        cells: Self::generate_desk(x, y),
        x_range: x,
        y_range: y
    }
    }


    fn generate_desk(x:i8, y:i8) -> Vec<Cell> {
        let mut cells = vec![];
        let mut index =0_i8;
        for i in 0..x {
            for j in 0..y {
                cells.push(Cell::new(i, j, index));
                index+=1;
            }
        }
        cells
    }

    pub fn get_cell_at(&self, x:i8, y:i8) -> Cell {
        return *self.cells.iter().find(|c| (c.position.x == x && c.position.y == y)).unwrap();
    }

    pub fn insert_unit_at_cell(&mut self, x:i8, y:i8, unit: Option<Unit>) {
        let index = self.cells.iter().position(|&c| (c.position.x == x && c.position.y == y)).unwrap();
        let mut cell = self.get_cell_at(x, y);
        cell.unit = unit;
        self.cells.push(cell);
        self.cells.swap_remove(index);
    }

    pub fn get_unit_by_id(&self, unit_id: i8) -> Unit {
        let cell = self.cells.iter().find(|c|(c.unit.is_some() && c.unit.unwrap().unit_id == unit_id));
        assert!(cell.is_some(), "No unit with same unit_id has found in this desk");
        cell.unwrap().unit.unwrap()
    }

    pub fn get_cells_to_deal_damage(&self, position: Vector2, unit_type: UnitType) -> Vec<Option<&Cell>> {
        assert!(position.x <= self.x_range && position.x >= 0, "Incorrect position");
        assert!(position.y <= self.y_range && position.y >= 0, "Incorrect position");
        match unit_type {
            UnitType::Orc => {
                self.get_orc_damage_cells(position)
            },
            UnitType::Mage => self.get_mage_damage_cells(position),
            UnitType::Skeleton => self.get_skeleton_damage_cells(position),
        }
    } 

    fn get_orc_damage_cells(&self, position: Vector2) -> Vec<Option<&Cell>> {
        let cells = vec![self.cells.iter().find(|&c| c.position.x == position.x+1 && c.position.y == position.y),
        self.cells.iter().find(|&c| c.position.y == position.y+1 && c.position.x == position.x),
        self.cells.iter().find(|&c| c.position.x == position.x-1 && c.position.y == position.y),
        self.cells.iter().find(|&c| c.position.y == position.y-1 && c.position.x == position.x),
        self.cells.iter().find(|&c| c.position.x == position.x+1 && c.position.y == position.y+1),
        self.cells.iter().find(|&c| c.position.x == position.x-1 && c.position.y == position.y-1),
        self.cells.iter().find(|&c| c.position.x == position.x-1 && c.position.y == position.y+1),
        self.cells.iter().find(|&c| c.position.x == position.x+1 && c.position.y == position.y-1)
        ];
        cells
    }

    fn get_mage_damage_cells(&self, position: Vector2) -> Vec<Option<&Cell>> {
        let cells = vec![self.cells.iter().find(|&c| c.position.x == position.x +1 && c.position.y == position.y),
        self.cells.iter().find(|&c| c.position.x == position.x +2 && c.position.y == position.y),
        self.cells.iter().find(|&c| c.position.x == position.x -1 && c.position.y == position.y),
        self.cells.iter().find(|&c| c.position.x == position.x -2 && c.position.y == position.y),
        self.cells.iter().find(|&c| c.position.y == position.y +1 && c.position.x == position.x),
        self.cells.iter().find(|&c| c.position.y == position.y +2 && c.position.x == position.x),
        self.cells.iter().find(|&c| c.position.y == position.y -1 && c.position.x == position.x),
        self.cells.iter().find(|&c| c.position.y == position.y -2 && c.position.x == position.x)];
        cells
    }

    fn get_skeleton_damage_cells(&self, position: Vector2) -> Vec<Option<&Cell>> {
        let cells = vec![self.cells.iter().find(|&c| c.position.x == position.x + 1 && c.position.y == position.y),
        self.cells.iter().find(|&c| c.position.x == position.x - 1 && c.position.y == position.y),
        self.cells.iter().find(|&c| c.position.y == position.y + 1 && c.position.x == position.x),
        self.cells.iter().find(|&c| c.position.y == position.y - 1 && c.position.x == position.x)];
        cells
    }

    pub fn place_unit(&mut self, position: Vector2, unit: Unit) -> bool {
        assert!(self.get_cell_at(position.x, position.y).unit.is_none(), "Cell is full!");
        self.insert_unit_at_cell(position.x, position.y, Some(unit));
        true
    }

    fn deal_damage_at_cell(&mut self, x: i8, y:i8, damage: f32) -> i8 {
        let index = self.cells.iter().position(|&c| (c.position.x == x && c.position.y == y)).unwrap();
        let mut cell = self.get_cell_at(x, y);
        let mut unit_to_delete_id = -1;
        if let Some(u) = &mut cell.unit {
            u.stats.hp -= damage;
            if u.stats.hp <= 0.0 {
                unit_to_delete_id = u.unit_id;
                cell.unit = None
                
            }
        }
        self.cells.push(cell);
        self.cells.swap_remove(index);
        unit_to_delete_id
    }
}

impl From<&Desk> for Desk {
    fn from(d: &Desk) -> Self {
        Self {
            cells: d.cells.clone(),
            x_range: d.x_range,
            y_range: d.y_range
        }
    }
}
#[derive(BorshDeserialize, BorshSerialize, Serialize)]
pub struct Battle {
    battle_id: i8,
    first_player: Option<AccountId>,
    second_player: Option<AccountId>,
    desk: Desk,
    bid: u128,
    first_player_units: Vec<i8>,
    second_player_units: Vec<i8>,
    current_state: i8,
    last_unit_id: i8,
    current_move_owner: i8,
    winner: Option<AccountId>
}

/// we cant derive Copy for non-primitive types so this is the reason why we implement From trait for our custom struct
impl From<&Battle> for Battle{
    fn from(b: &Battle) -> Self {
        Self { battle_id: b.battle_id, 
            first_player: b.first_player.clone(), 
            second_player: b.second_player.clone(),
            desk: Desk::from(&b.desk),
            first_player_units: b.first_player_units.clone(),
            second_player_units: b.second_player_units.clone(),
            bid: b.bid,
            current_state: b.current_state,
            last_unit_id: b.last_unit_id,
            current_move_owner: b.current_move_owner,
            winner: b.winner.clone()
        }
    }
}

impl Battle {

    pub fn new(_bid: u128, creator: AccountId, id: i8, x: i8, y: i8) -> Self {
        Battle { 
            battle_id: id, 
            first_player: Some(creator), 
            second_player: None,
            desk: Desk::new(x, y),
            first_player_units: vec![],
            second_player_units: vec![],
            bid: _bid,
            current_state: -1,
            last_unit_id: 0,
            current_move_owner: 0,
            winner: None
        }
    }

    pub fn join_battle(&mut self, player_id: AccountId) {
        assert!(self.second_player.is_none(), "Battle is full");
        self.second_player = Some(player_id);
        self.current_state = 0;
    }

    pub fn make_move(&mut self, unit_id: i8, moves: Vec<(i8,i8)>, caller: AccountId) -> Option<AccountId> {
        assert!(self.current_state == 2, "Current state must be 2");
        if Some(caller.clone()) == self.first_player {
            assert!(self.first_player_units.contains(&unit_id), "Illegal move! This unit is not yours!");
            assert!(self.current_move_owner == 0, "Current move owner is second player");
        }
        else if Some(caller) == self.second_player {
            assert!(self.second_player_units.contains(&unit_id), "Illegal move! This unit is not yours");
            assert!(self.current_move_owner == 1, "Current move owner is first player");
        }
        let length = moves.len();
        let (start_x, start_y) = moves.first().unwrap();
        let unit = self.desk.get_unit_by_id(unit_id);
        let initial_damage = unit.stats.damage as f32;
        assert!(self.desk.get_cell_at(*start_x, *start_y).unit.is_some(), "There is no unit on start cell!");
        assert!(unit_id == (self.desk.get_cell_at(*start_x, *start_y)).unit.unwrap().unit_id);
        let mut damage = initial_damage;
        for i in 0..length-1 {
            let (x1,y1) = moves.get(i).unwrap();
            let (x2,y2) = moves.get(i+1).unwrap();
            let pos_1 = Vector2 { x: *x1, y: *y1, id: 100 };
            let pos_2 = Vector2 { x: *x2, y: *y2, id: 100 };
            if self.is_valid_move(pos_1, pos_2) {
                if self.is_unit_between(pos_1, pos_2){
                    damage = initial_damage * (1_f32+0.5*(i as i8+1) as f32);
                }
            }
            else {
                panic!("Move was Illegal! {} {} - {} {}", pos_1.x, pos_1.y, pos_2.x, pos_2.y);
            }
        }
        let (end_x, end_y) = moves.last().unwrap();
        self.desk.insert_unit_at_cell(*end_x, *end_y, Some(unit));
        self.desk.insert_unit_at_cell(*start_x, *start_y, None);
        self.deal_damage(damage, Vector2 { x: *end_x, y: *end_y, id: -10 }, unit.unit_type);
        if self.current_move_owner == 0 {
            self.current_move_owner = 1;
        }
        else if self.current_move_owner == 1 {
            self.current_move_owner = 0;
        }
        if self.first_player_units.is_empty() {
            self.winner = Some(self.second_player.clone().unwrap());
            self.current_state = 3;
        }
        else if self.second_player_units.is_empty() {
            self.winner = Some(self.first_player.clone().unwrap());
            self.current_state = 3;
        }
        self.winner.clone()
    }

    fn deal_damage(&mut self, damage: f32, cell_position: Vector2, unit_type: UnitType) {
        let binding = self.desk.clone();
        let cells = binding.get_cells_to_deal_damage(cell_position, unit_type);
        cells.iter().for_each(|x| {
            if x.is_some() && x.unwrap().unit.is_some() {
                let unit_to_delete = self.desk.deal_damage_at_cell(x.unwrap().position.x, x.unwrap().position.y, damage);
                if unit_to_delete != -1 {
                    if self.first_player_units.contains(&unit_to_delete) {
                        let index = self.first_player_units.iter().position(|&x| x == unit_to_delete);
                        if index.is_some() {
                            self.first_player_units.remove(index.unwrap());
                        }
                    }
                    else if self.second_player_units.contains(&unit_to_delete) {
                        let index = self.second_player_units.iter().position(|&x| x == unit_to_delete);
                        if index.is_some() {
                            self.second_player_units.remove(index.unwrap());
                        }
                    }

                }
            }
        });
        //initial_desk_cells.iter().filter(|x| x.position.x == cells.iter().any(f))
    } 

    fn is_valid_move(&self, pos_1: Vector2, pos_2: Vector2) -> bool {
        match self.get_move_vector(pos_1, pos_2) {
            0 => {
                if self.is_valid_y_move(pos_1.y, pos_2.y, pos_1.x) {
                    return true
                }
                
            },
            1 => {
                if self.is_valid_x_move(pos_1.x, pos_2.x, pos_1.y) {
                    return true
                }
            },
            _ => panic!("Illegal move!")
        }
        false
    }

    fn is_unit_between(&self, pos_1: Vector2, pos_2: Vector2) -> bool {
        let min_x = std::cmp::min(pos_1.x,pos_2.x);
        let min_y = std::cmp::min(pos_1.y, pos_2.y);
        match self.get_move_vector(pos_1, pos_2) {
            0 => {
                if self.desk.get_cell_at(pos_1.x, min_y+1).unit.is_some() {
                    return true
                }
                false
            },
            1 => {
                
                if self.desk.get_cell_at(min_x + 1, pos_1.y).unit.is_some() {
                    return true
                }
                false
            },
            _ => panic!("Illegal move!")
        }
    }

    fn get_move_vector(&self, pos_1: Vector2, pos_2: Vector2) -> i8 {
        if pos_1.x == pos_2.x {
            return 0
        }
        else if pos_1.y == pos_2.y {
            return 1
        }
        -1
    }

    fn is_valid_x_move(&self, x_1: i8, x_2: i8, y: i8) -> bool {
        let max = std::cmp::max(x_1, x_2);
        let min = std::cmp::min(x_1, x_2);
        if max - min <= 0 {
            return false
        }
        else if max - min == 2 && self.desk.get_cell_at(min+1, y).unit.is_some() {
            return true
        }
        if max - min == 1 {
            return true;
        }
        false
    }

    fn is_valid_y_move(&self, y_1: i8, y_2:i8, x: i8) -> bool {
        let max = std::cmp::max(y_1,y_2);
        let min = std::cmp::min(y_1, y_2);
        if max - min <= 0 {
            return false
        }
        else if max-min == 2 && self.desk.get_cell_at(x, min+1).unit.is_some() {
            return true
        }
        if max - min == 1 {
            return true;
        }
        false
    }

    pub fn place_units(&mut self, units: Vec<(i8,i8,i8)>, caller: AccountId) {
        assert!(self.current_state == 0, "Incorrect state!");
        assert!(units.len() <= 6, "Max units is 6");
        if Some(caller.clone()) == self.first_player {
            assert!(self.first_player_units.len() + units.len() <=6, "Max units is 6!");
        }
        else if Some(caller.clone()) == self.second_player {
            assert!(self.second_player_units.len() + units.len() <= 6, "Max units is 6!");
        }

        assert!(Some(caller.clone()) == self.first_player || Some(caller.clone()) == self.second_player, "This is not your room!");
        for unit in units {
        
        let (unit_type, x, y) = unit;
        let new_unit = Unit::new(1, Unit::get_unit_type_by_id(unit_type), self.last_unit_id);
        if Some(caller.clone()) == self.first_player {
            assert!(x <= 2, "you cant place unit on this cell");
            self.first_player_units.push(self.last_unit_id);
        }
        else {
            assert!(x < self.desk.x_range && x>= 4, "you cant place unit on this cell");
            self.second_player_units.push(self.last_unit_id);
        }
        self.last_unit_id +=1;
        let pos = Vector2::new(x,y,0);
        self.desk.place_unit(pos, new_unit);
        if !(self.first_player_units.is_empty()) && !(self.second_player_units.is_empty()) {
            self.current_state = 2;
        }
        }
    }
}


use near_sdk::collections::{Vector, LookupMap};
use near_sdk::{env, Balance, assert_one_yocto};


const MIN_BID: Balance = 10_000_000_000_000_000_000_000;
const ONE_YOCTO: Balance =1;
const DEFAULT_FIELD: Vector2 = Vector2{x:7, y:4, id:0};
#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Battles,
    BattleOfPlayer,
    LastBattleOfPlayer,
    OpenedBattles
}

// Define the contract structure
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    battles: Vector<Battle>,
    battles_of_player: LookupMap<AccountId, i8>,
    opened_battles: Vector<Battle>,
    last_player_battle_id: LookupMap<AccountId, i8>,
}

// Define the default, which automatically initializes the contract
impl Default for Contract{
    fn default() -> Self{
        Self{
            battles: Vector::new(StorageKey::Battles),
            battles_of_player: LookupMap::new(StorageKey::BattleOfPlayer),
            last_player_battle_id: LookupMap::new(StorageKey::LastBattleOfPlayer),
            opened_battles: Vector::new(StorageKey::OpenedBattles)
        }
    }
}

#[near_bindgen]
// Implement the contract structure
impl Contract {
   
    #[payable]
    pub fn join_battle(&mut self, battle_id: i8) -> bool {
        let (mut battle, battle_index) = self.get_battle(battle_id);
        assert_eq!(battle.battle_id.clone(), battle_id.clone());
        assert!(env::attached_deposit() >= battle.bid);
        battle.join_battle(env::signer_account_id());
        let op_battle_index = self.opened_battles.iter().position(|x| x.battle_id == battle.battle_id);
        self.opened_battles.swap_remove(op_battle_index.unwrap().try_into().unwrap());
        self.battles.push(&battle);
        self.battles.swap_remove(battle_index.try_into().unwrap());
        true
    }


    pub fn place_units(&mut self, battle_id: i8, units: Vec<(i8,i8,i8)>) {
        let (mut battle, battle_index) = self.get_battle(battle_id);
        assert_eq!(battle.battle_id.clone(), battle_id.clone());
        battle.place_units(units, env::signer_account_id());
        self.battles.push(&battle);
        self.battles.swap_remove(battle_index.try_into().unwrap());
    }

    #[private]
    fn get_battle(&self, battle_id: i8) -> (Battle, usize) {
        let battles_vec = self.battles.to_vec();
        let battle_index = battles_vec.iter().position(|b| b.battle_id == battle_id).unwrap();
        let battle = self.battles.get(battle_index.try_into().unwrap()).unwrap();
        (battle, battle_index)
    }
    
    pub fn find_battle(&self) -> (u128,i8){
        if self.opened_battles.is_empty() {
            panic!("No opened battles found");
        }
        let battle = self.opened_battles.get(0).unwrap();
        (battle.bid, battle.battle_id)
    }

    pub fn make_move(&mut self, battle_id: i8, unit_id: i8, moves: Vec<(i8,i8)>) -> bool {
        let (mut battle, index) = self.get_battle(battle_id);
        let winner = battle.make_move(unit_id, moves, env::signer_account_id());
        self.battles.push(&battle);
        self.battles.swap_remove(index.try_into().unwrap());
        if winner.is_some() {
            Promise::new(winner.unwrap()).transfer(battle.bid * 2);
        }
        true
    }

    #[payable]
    pub fn create_battle(&mut self) -> i8 {
        assert!(near_sdk::env::attached_deposit() >= MIN_BID, "Attached deposit is less than minimum bid");
        let battle_id = self.battles.len() as i8;
        let battle = Battle::new(
            near_sdk::env::attached_deposit(),
            env::signer_account_id(),
            battle_id,
            DEFAULT_FIELD.x,
            DEFAULT_FIELD.y);
        self.battles.push(&battle);
        self.last_player_battle_id.insert(&env::signer_account_id(), &battle_id.clone());
        self.opened_battles.push(&battle);
        battle_id
    }

    pub fn get_battle_info(&self, battle_id: i8) -> Battle {
        self.get_battle(battle_id).0
    }

    pub fn get_last_battle_of_player (&self, player_account: AccountId) -> Option<i8> {
        
        self.last_player_battle_id.get(&player_account)
    }

    pub fn leave_battle (&mut self) {
        self.last_player_battle_id.remove(&env::signer_account_id());
    }
}