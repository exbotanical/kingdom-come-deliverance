use rltk::{self};
use specs::prelude::*;
use specs::World;

use crate::components::Position;
use crate::components::Renderable;
use crate::map::draw_map;
use crate::map::Map;
use crate::player::player_input;
use crate::systems::damage;
use crate::systems::DamageSystem;
use crate::systems::MapIndexingSystem;
use crate::systems::MeleeCombatSystem;
use crate::systems::MonsterAISystem;
use crate::systems::VisibilitySystem;

#[derive(PartialEq, Clone, Copy)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
}

pub struct State {
    pub ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);

        let mut mob = MonsterAISystem {};
        mob.run_now(&self.ecs);

        let mut map_idx = MapIndexingSystem {};
        map_idx.run_now(&self.ecs);

        let mut melee = MeleeCombatSystem {};
        melee.run_now(&self.ecs);

        let mut damage = DamageSystem {};
        damage.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

impl rltk::GameState for State {
    fn tick(&mut self, ctx: &mut rltk::Rltk) {
        ctx.cls();

        let mut run_state = *self.ecs.fetch::<RunState>();

        match run_state {
            RunState::PreRun => {
                self.run_systems();
                run_state = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                run_state = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                run_state = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems();
                run_state = RunState::AwaitingInput;
            }
        }

        {
            let mut run_writer = self.ecs.write_resource::<RunState>();
            *run_writer = run_state;
        }

        damage::reap(&mut self.ecs);

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();
        let map = self.ecs.fetch::<Map>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
            }
        }
    }
}
