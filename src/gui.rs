extern crate rltk;
use rltk::{ RGB, Rltk, Console };
extern crate specs;
use specs::prelude::*;
use super::{CombatStats, Player, gamelog::GameLog};


pub fn draw_ui(ecs: &World, ctx : &mut Rltk) {
    ctx.draw_box(0, 43, 79, 6, RGB::named(rltk::WHITE), RGB::named(rltk::BLACK));

    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!(" HP: {} / {} ", stats.hp, stats.max_hp);
        ctx.print_color(12, 43, RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK), &health);

        ctx.draw_bar_horizontal(28, 43, 51, stats.hp, stats.max_hp, RGB::named(rltk::RED), RGB::named(rltk::BLACK));
    }

    let log = ecs.fetch::<GameLog>();

    let mut y = 44;
    for txt in log.entries.iter() {
        if y < 49 { ctx.print(2, y, &txt.to_string()); }
        y += 1;
    }

    
}