use crate::constants::*;
use crate::error::DieselError;
use crate::models::AttackType;
use crate::simulation::blocks::BuildingsManager;
use crate::simulation::defense::DefenseManager;
use crate::simulation::error::*;
use crate::util::function;
use anyhow::Result;
use diesel::prelude::*;
use diesel::PgConnection;
use std::collections::{HashMap, HashSet};

use super::attacker::Attacker;

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct Emp {
    pub path_id: usize,
    pub x_coord: i32,
    pub y_coord: i32,
    pub radius: i32,
    pub damage: i32,
    pub attacker_id: i32,
}

pub struct Emps(pub HashMap<i32, HashSet<Emp>>);

impl Emps {
    // Returns a hashmap of Emp with time as key
    pub fn new(conn: &mut PgConnection, attackers: &HashMap<i32, Attacker>) -> Result<Self> {
        use crate::schema::attack_type;
        let mut emps = HashMap::new();
        let emp_types: HashMap<i32, AttackType> = attack_type::table
            .load::<AttackType>(conn)
            .map_err(|err| DieselError {
                table: "attack_type",
                function: function!(),
                error: err,
            })?
            .into_iter()
            .map(|emp| (emp.id, emp))
            .collect();

        for (attacker_id, attacker) in attackers.iter() {
            let attacker_path = &attacker.path;

            for path in attacker_path {
                if !path.is_emp {
                    continue;
                }
                if let (Some(emp_type), Some(emp_time)) = (path.emp_type, path.emp_time) {
                    let emp_type = emp_types.get(&emp_type).ok_or(KeyError {
                        key: emp_type,
                        hashmap: "emp_types".to_string(),
                    })?;
                    let emp = Emp {
                        path_id: path.id,
                        x_coord: path.x_coord,
                        y_coord: path.y_coord,
                        radius: emp_type.attack_radius,
                        damage: emp_type.attack_damage,
                        attacker_id: *attacker_id,
                    };

                    emps.entry(emp_time).or_insert_with(HashSet::new);
                    emps.get_mut(&emp_time).unwrap().insert(emp);
                } else {
                    return Err(EmpDetailsError { path_id: path.id }.into());
                }
            }
        }
        Ok(Emps(emps))
    }

    pub fn simulate(
        &self,
        minute: i32,
        buildings_manager: &mut BuildingsManager,
        defense_manager: &mut DefenseManager,
        attackers: &mut HashMap<i32, Attacker>,
    ) -> Result<()> {
        let Emps(emps) = self;
        if !emps.contains_key(&minute) {
            return Ok(());
        }
        for emp in emps.get(&minute).ok_or(KeyError {
            key: minute,
            hashmap: "emps".to_string(),
        })? {
            // Get the attacker of the corresponding EMP
            let attacker = attackers.get(&emp.attacker_id).ok_or(KeyError {
                key: emp.attacker_id,
                hashmap: "emp_types".to_string(),
            })?;
            if !attacker.is_planted(emp.path_id)? {
                continue;
            }
            let radius = emp.radius;

            let mut affected_buildings: HashSet<i32> = HashSet::new();

            for x in emp.x_coord - radius..=emp.x_coord + radius {
                for y in emp.y_coord - radius..=emp.y_coord + radius {
                    if !(0..MAP_SIZE as i32).contains(&x) || !(0..MAP_SIZE as i32).contains(&y) {
                        continue;
                    }
                    let distance = (x - emp.x_coord).pow(2) + (y - emp.y_coord).pow(2);
                    if distance > radius.pow(2) {
                        continue;
                    }

                    defense_manager.defenders.get_damage(x, y);

                    // attackers is in the imapact of EMP
                    for (_, attacker) in attackers.iter_mut() {
                        let attacker_path = attacker.path_in_current_frame.clone();
                        for (position, attacker_path_stats) in
                            attacker_path.iter().rev().enumerate()
                        {
                            if x == attacker_path_stats.attacker_path.x_coord
                                && y == attacker_path_stats.attacker_path.y_coord
                            {
                                attacker.get_damage(
                                    emp.damage,
                                    attacker.path_in_current_frame.len() - 1 - position,
                                );
                                break;
                            }
                        }
                    }

                    let building_id = buildings_manager.buildings_grid[x as usize][y as usize];
                    if building_id != 0 {
                        affected_buildings.insert(building_id);
                    } else {
                        /* robots on road
                        robots_manager.damage_and_reassign_robots(
                            emp.damage,
                            x,
                            y,
                            buildings_manager,
                        )?; */
                    }

                    // Robots whose shortest path was in impact of an emp
                    /*
                    CODE
                     */
                }
            }

            /* for building_id in &affected_buildings {
                /* let building =
                    buildings_manager
                        .buildings
                        .get_mut(building_id)
                        .ok_or(KeyError {
                            key: *building_id,
                            hashmap: "buildings".to_string(),
                        })?; */
                /*let Building {
                    absolute_entrance_x: x,
                    absolute_entrance_y: y,
                    ..
                } = building;*/
                // robots in affected building
               /*
               code for damage to robots inside building
                */

                /* let building =
                    buildings_manager
                        .buildings
                        .get_mut(building_id)
                        .ok_or(KeyError {
                            key: *building_id,
                            hashmap: "buildings".to_string(),
                        })?; */

                /*
                reduce robots in building after damage

                building.population -= destroyed_robots;
                 */
                // robots going to affected building
            } */
        }
        Ok(())
    }
}
