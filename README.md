# Hop Chain Contracts

![banner](https://user-images.githubusercontent.com/73827631/203133565-82908d63-e33b-4692-8c94-d67eb7115a74.png)

# Summary
 - [Contract methods](#contract-methods)
 - [Game Rules](#game-rules)
 - [Units description](#units-description)
 - [Deployment instruction](#deployment-instruction)
 
Hop chain is a revolutionary on-chain tactic game which was insipired by [Chinese Checkers](https://en.wikipedia.org/wiki/Chinese_checkers).
Contracts were written on NEAR using Rust programming language.
## Contract methods
We recommend to read [Game rules](#game-rules) first.

Contract fields: 
  - battles - vector of all battles.
  - battles_of_player - map of battles where key is the player account and value is the ID of the battle.
  - opened_battles - vector of all battles you can join.
  - last_player_battle_id - map which is used to store last battle player participated in.
```rust
pub struct Contract {
    battles: Vector<Battle>,
    battles_of_player: LookupMap<AccountId, i8>,
    opened_battles: Vector<Battle>,
    last_player_battle_id: LookupMap<AccountId, i8>,
}
```
- get_last_battle_of_player - basically just returns id of the last battle player participated in
```rust
 pub fn get_last_battle_of_player (&self, player_account: AccountId) -> Option<i8> {
        let id = self.last_player_battle_id.get(&player_account);
        id
    }
```
- create_battle - payable method which creates a new "room" in contract where any player can join. This method is payable and requires deposit.
```rust
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
```
- make_move - allows you to move unit from one cell to another. Accepts:
  - battle_id - an ID of the battle you want to change values in.
  - unit_id - unit that you want to move.
  - moves - a vector of moves. Must contain all of the cells your unit is going to visit during this turn. ex: [[1,2],[3,2],[5,2]].
```rust
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
```
- find_battle - just returns bid and id of any opened battle if there is any
```rust
pub fn find_battle(&self) -> (u128,i8){
        if (self.opened_battles.is_empty()) {
            panic!("No opened battles found");
        }
        let battle = self.opened_battles.get(0).unwrap();
        (battle.bid, battle.battle_id)
    }
```
- place_units - allows you to place units on the game field. 
  - battle_id - an ID of the battle you want to change values in.
  - units - a vector which contains unit type and it's position where unit type is the first element of the array element and x&y coordinates will be the second and the third elements. For example if we want to place skeleton on cell [1,2] we will pass [2,1,2] ([unit type, x position, y position])
```rust
    pub fn place_units(&mut self, battle_id: i8, units: Vec<(i8,i8,i8)>) {
        let (mut battle, battle_index) = self.get_battle(battle_id);
        assert_eq!(battle.battle_id.clone(), battle_id.clone());
        battle.place_units(units, env::signer_account_id());
        self.battles.push(&battle);
        self.battles.swap_remove(battle_index.try_into().unwrap());
    }
```
- join_battle - method which lets you join one of the opened battles
  - battle_id - an ID of the battle you want to join
```rust
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
```
# Game Rules
There are 3 types of units in game:
- Orc
- Skeleton
- Mage

Each unit has it's own attack pattern and unique stats. Point of the game is to eliminate all of the enemy's units.

Unit can jump 1 cell on X or Y coordinates, or, it can make a series of moves jumping over allies or enemies. Every hop multiplies base damage of the unit to 1.5x-2x-2.5x-3x-... . After last hop, it deals splash damage based on attack pattern (shown bellow for each unit type). We were trying to build fair balance so we each unit has it's own tactic. 

For example Orc has lots of health, but has low damage and can damage all nearby cells.

On the other hand Skeleton has smaller health amount, but deals lots of damage to 4 directions.

Finally Mage has very low amount of hp and average damage, but can shoot 1 cell further than other units.


## Units description
#### Orc
<img src = "https://user-images.githubusercontent.com/73827631/203140844-641af63a-c5e3-4463-9bbe-c54618a96a49.png" width="200" height="200"/>

###### Stats
|level|hp|damage|
|-|-|-|
|1|40|4|
|2|42|6|
|3|48|6|
|4|56|8|
|5|64|8|
###### Attack pattern
| | | |
|-|-|-|
|:axe:|:axe:|:axe:|
|:axe:|:japanese_ogre:|:axe:|
|:axe:|:axe:|:axe:|

#### Skeleton
<img src = "https://user-images.githubusercontent.com/73827631/203140785-dd7af68a-4454-4ffe-9a8a-415f203001e1.png" width="200" height="200"/>

###### Stats
|level|hp|damage|
|-|-|-|
|1|25|10|
|2|27|12|
|3|30|14|
|4|35|16|
|5|40|18|


###### Attack pattern
| | | |
|-|-|-|
| |:dagger:| |
|:dagger:|:skull:|:dagger:|
| |:dagger:| |


#### Mage

<img src = "https://user-images.githubusercontent.com/73827631/203141086-f0e3ed5c-78cf-46be-b6f2-d413a2afef2b.png" width="200" height="200"/>

###### Stats
|level|hp|damage|
|-|-|-|
|1|12|8|
|2|15|10|
|3|18|12|
|4|22|14|
|5|25|16|

###### Attack Pattern

| | | | | |
|-|-|-|-|-|
| | |:magic_wand:| | |
| | |:magic_wand:| | |
|:magic_wand:|:magic_wand:|:mage:|:magic_wand:|:magic_wand:|
| | |:magic_wand:| | |
| | |:magic_wand:| | |
# Deployment instruction
  - Download Github repo
  - run ```yarn install``` or ```npm install```
  - run commands bellow to start deployment process.
  
**Run these command in CLI**
```shell
yarn build
near deploy --accountId *your account ID* --wasmFile *path to wasm file*/hop_chain_contracts.wasm
```
