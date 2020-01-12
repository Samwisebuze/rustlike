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

struct State {
    ecs: World
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
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
        .build();
    }

    rltk::main_loop(context, gs);
}
