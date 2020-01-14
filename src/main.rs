extern crate rltk;
use rltk::{Console, GameState, Point, RandomNumberGenerator, Rltk, RGB};
extern crate specs;
use specs::prelude::*;
#[macro_use]
extern crate specs_derive;
mod components;
pub use components::*;
mod map;
pub use map::*;
mod player;
use player::*;
mod rect;
pub use rect::Rect;
mod visibility_system;
use visibility_system::VisibilitySystem;
mod monster_ai_system;
use monster_ai_system::MonsterAI;
mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;
mod damage_system;
use damage_system::DamageSystem;
mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;

rltk::add_wasm_support!();

#[derive(PartialEq, Copy, Clone)]
pub enum RunState { AwaitingInput, PreRun, PlayerTurn, MonsterTurn }    

pub struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        let mut mob = MonsterAI {};
        let mut map_idx = MapIndexingSystem {};
        let mut dmg = DamageSystem {};
        let mut melee = MeleeCombatSystem {};
        vis.run_now(&self.ecs);
        mob.run_now(&self.ecs);
        map_idx.run_now(&self.ecs);
        melee.run_now(&self.ecs);
        dmg.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        let mut newrunstate;
        { // Borrow-Checker Scope
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }
        
        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                newrunstate = RunState::AwaitingInput;
            }
        }

        { // Borrow_Checker Scope
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        // *BONG* BRING OUT YER DEAD *BONG*
        damage_system::delete_the_dead(&mut self.ecs);
        // Render Loop
        {
            draw_map(&self.ecs, ctx);

            let positions = self.ecs.read_storage::<Position>();
            let renderables = self.ecs.read_storage::<Renderable>();
            let map = self.ecs.fetch::<Map>();

            for (pos, render) in (&positions, &renderables).join() {
                let idx = map.xy_idx(pos.x, pos.y);
                if map.visible_tiles[idx] { 
                    ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph) 
                }
            }
        }
    }
}

fn main() {
    let context = Rltk::init_simple8x8(80, 50, "Rustlike", "resources");
    let mut gs = State {
        ecs: World::new(),
    };
    gs.ecs.insert(RunState::PreRun);
    // Register Components to World
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();

    // Generate Map
    let map: Map = Map::new_map_rooms_and_corridors();
    // Get Player's Spawn point
    let (player_x, player_y) = map.rooms[0].center();

    let mut rng = RandomNumberGenerator::new();
    for (i, room) in map.rooms.iter().skip(1).enumerate() {
        let (x, y) = room.center();

        let glyph: u8;
        let name: String;
        let roll = rng.roll_dice(1, 2);
        match roll {
            1 => {
                glyph = rltk::to_cp437('g');
                name = "Goblin".to_string();
            }
            _ => {
                glyph = rltk::to_cp437('o');
                name = "Orc".to_string();
            }
        }
        gs.ecs
            .create_entity()
            .with(Position { x, y })
            .with(Renderable {
                glyph: glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .with(Monster {})
            .with(Name {
                name: format!("{} #{}", &name, i),
            })
            .with(CombatStats {
                max_hp: 16,
                hp: 16,
                defense: 1,
                power: 4,
            })
            .with(BlocksTile {})
            .build();
    }

    gs.ecs.insert(map);
    // Initialize Player Entity
    let player_entity = player::init_player(player_x, player_y, &mut gs.ecs);
    gs.ecs.insert(player_entity);
    // Register Player's Point with the world
    gs.ecs.insert(Point::new(player_x, player_y));

    rltk::main_loop(context, gs);
}
