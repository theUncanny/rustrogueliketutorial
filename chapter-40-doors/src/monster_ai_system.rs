extern crate specs;
use specs::prelude::*;
use super::{Viewshed, Monster, Map, Position, WantsToMelee, RunState, 
    Confusion, particle_system::ParticleBuilder, EntityMoved};
extern crate rltk;
use rltk::{Point};

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadExpect<'a, Point>,
                        ReadExpect<'a, Entity>,
                        ReadExpect<'a, RunState>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>, 
                        ReadStorage<'a, Monster>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, WantsToMelee>,
                        WriteStorage<'a, Confusion>,
                        WriteExpect<'a, ParticleBuilder>,
                        WriteStorage<'a, EntityMoved>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, player_pos, player_entity, runstate, entities, mut viewshed, 
            monster, mut position, mut wants_to_melee, mut confused, mut particle_builder,
            mut entity_moved) = data;

        if *runstate != RunState::MonsterTurn { return; }

        for (entity, mut viewshed,_monster,mut pos) in (&entities, &mut viewshed, &monster, &mut position).join() {
            let mut can_act = true;

            let is_confused = confused.get_mut(entity);
            if let Some(i_am_confused) = is_confused {
                i_am_confused.turns -= 1;
                if i_am_confused.turns < 1 {
                    confused.remove(entity);
                }
                can_act = false;

                particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::MAGENTA), 
                    rltk::RGB::named(rltk::BLACK), rltk::to_cp437('?'), 200.0);
            }

            if can_act {
                let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
                if distance < 1.5 {
                    wants_to_melee.insert(entity, WantsToMelee{ target: *player_entity }).expect("Unable to insert attack");
                }
                else if viewshed.visible_tiles.contains(&*player_pos) {
                    // Path to the player
                    let path = rltk::a_star_search(
                        map.xy_idx(pos.x, pos.y) as i32, 
                        map.xy_idx(player_pos.x, player_pos.y) as i32, 
                        &mut *map
                    );
                    if path.success && path.steps.len()>1 {
                        let mut idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = false;
                        pos.x = path.steps[1] % map.width;
                        pos.y = path.steps[1] / map.width;
                        entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
                        idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = true;
                        viewshed.dirty = true;
                    }
                }
            }
        }
    }
}