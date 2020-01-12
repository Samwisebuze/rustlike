rltk::add_wasm_support!();
use rltk::{Console, GameState, Rltk, RGB, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};
#[macro_use]
extern crate specs_derive;

// The derive macro simplifies component creation, 
// by generating the boilerplate needed for it.
#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: u8,
    fg: RGB,
    bg: RGB,
}

// The alternative to specs-derive is to implement
// the Component class every single time.
// impl Component for Position {
//     type Storage = VecStorage<Self>;
// }

#[derive(Component)]
struct LeftMover {}

// An empty struct used to attach the logic in
// the LeftWalker System below
struct LeftWalker {}

// Implement a specs System for the Left Mover Functionality
// 'a is a lifetime specifier, so the system will 
// exist just long enough to run.
impl<'a> System<'a> for LeftWalker {
    type SystemData = (ReadStorage<'a, LeftMover>, 
                        WriteStorage<'a, Position>);
    // Implement the run trait from System
    fn run(&mut self, (lefty, mut pos) : Self::SystemData) {
        for (_lefty, pos) in (&lefty, &mut pos).join() {
            pos.x -= 1;
            if pos.x < 0 { pos.x = 79; }
        }
    }
}

struct State {
    ecs: World,

}

impl State {
    fn run_systems(&mut self) {
        // Create a new mutable LeftWalker System
        let mut lw = LeftWalker{};
        // Run the Left walker system on the World
        lw.run_now(&self.ecs);
        // Tells the world to run any queued changes
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        self.run_systems();
        
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}

fn main() {
    let context = Rltk::init_simple8x8(80, 50, "Hello Rust World!", "resources");
    let mut gs = State {
        ecs: World::new()
    };
    // Register Components with the GameState
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<LeftMover>();

    // Create a new Entity
    gs.ecs
    .create_entity()
    .with(Position { x: 40, y: 25 })
    .with(Renderable {
        glyph: rltk::to_cp437('@'),
        fg: RGB::named(rltk::YELLOW),
        bg: RGB::named(rltk::BLACK),
    })
    .build();

    // Create New Entities
    for i in 0..=10 {
        gs.ecs
        .create_entity()
        .with(Position { x: i * 7, y: 20 })
        .with(Renderable {
            glyph: rltk::to_cp437('☺'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
        })
        .with(LeftMover{})
        .build();
    }

    rltk::main_loop(context, gs);
}
