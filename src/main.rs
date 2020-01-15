extern crate rltk;
use rltk::{Console, GameState, Point, Rltk};
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
mod gamelog;
mod gui;
mod inventory_system;
mod spawner;
use inventory_system::{ItemCollectionSystem, ItemDropSystem, ItemUseSystem};

rltk::add_wasm_support!();

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting { range: i32, item: Entity },
}

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
        let mut pickup = ItemCollectionSystem {};
        let mut potions = ItemUseSystem {};
        let mut drop_items = ItemDropSystem {};
        vis.run_now(&self.ecs);
        mob.run_now(&self.ecs);
        map_idx.run_now(&self.ecs);
        melee.run_now(&self.ecs);
        dmg.run_now(&self.ecs);
        pickup.run_now(&self.ecs);
        drop_items.run_now(&self.ecs);
        potions.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        let mut newrunstate;
        {
            // Borrow-Checker Scope
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let result = gui::show_inventory(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let is_ranged = self.ecs.read_storage::<Ranged>();
                        let is_item_ranged = is_ranged.get(item_entity);

                        if let Some(is_item_ranged) = is_item_ranged {
                            newrunstate = RunState::ShowTargeting {
                                range: is_item_ranged.range,
                                item: item_entity,
                            };
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent
                                .insert(
                                    *self.ecs.fetch::<Entity>(),
                                    WantsToUseItem { item: item_entity, target: None },
                                )
                                .expect("Unable to insert intent");
                            newrunstate = RunState::PlayerTurn;
                        }
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = gui::drop_item_menu(self, ctx);
                match result.0 {
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Cancel => {
                        newrunstate = RunState::AwaitingInput;
                    }
                    gui::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToDropItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowTargeting { range, item } => {
                let result = gui::ranged_target(self, ctx, range);
                match result.0 {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent.insert(*self.ecs.fetch::<Entity>(), WantsToUseItem{ item, target: result.1 }).expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
        }

        {
            // Borrow_Checker Scope
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

            let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
            data.sort_by(|&a, &b| b.1.render_order.cmp(&a.1.render_order));
            for (pos, render) in data.iter() {
                let idx = map.xy_idx(pos.x, pos.y);
                if map.visible_tiles[idx] {
                    ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph)
                }
            }
        }

        gui::draw_ui(&self.ecs, ctx);
    }
}

fn main() {
    let mut context = Rltk::init_simple8x8(80, 50, "Rustlike", "resources");
    context.with_post_scanlines(true);
    let mut gs = State { ecs: World::new() };

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
    gs.ecs.register::<Item>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<WantsToDropItem>();


    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    // Generate Map
    let map: Map = Map::new_map_rooms_and_corridors();
    // Get Player's Spawn point
    let (player_x, player_y) = map.rooms[0].center();
    // Initialize Player Entity
    let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);
    // Spawn Stuff in Rooms
    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room);
    }

    gs.ecs.insert(map);
    // Register Player's Point with the world
    gs.ecs.insert(Point::new(player_x, player_y));
    gs.ecs.insert(player_entity);
    gs.ecs.insert(RunState::PreRun);
    gs.ecs.insert(gamelog::GameLog {
        entries: vec!["Welcome to Rustlike".to_string()],
    });

    rltk::main_loop(context, gs);
}
