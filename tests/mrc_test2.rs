use std::{cmp, rc::Rc, cell::RefCell};

use ptstd::ptr::mrc::Mrc;
use rand::Rng;

type OnReceivedDamage = Box<dyn Fn(u32)>;

struct Monster {
    health: u32,
    received_damage: Vec<OnReceivedDamage>,
}

impl Monster {
    fn take_damage(&mut self, amount: u32) {
        let damage_received = cmp::min(self.health, amount);
        self.health -= damage_received;
        for callback in &mut self.received_damage {
            callback(damage_received);
        }
    }

    fn add_listener(&mut self, listener: OnReceivedDamage) {
        self.received_damage.push(listener);
    }
}

impl Default for Monster {
    fn default() -> Self {
        Monster { health: 100, received_damage: Vec::new() }
    }
}

#[derive(Default)]
struct DamageCounter {
    damage_inflicted: u32,
}

impl DamageCounter {
    fn reached_target_damage(&self) -> bool {
        self.damage_inflicted > 100
    }

    fn on_damage_received(&mut self, damage: u32) {
        self.damage_inflicted += damage;
    }
}

#[test]
fn test_example() {
    let mut rng = rand::thread_rng();
    let counter = Rc::new(RefCell::new(DamageCounter::default()));
    let mut monsters: Vec<_> = (0..5).map(|_| Monster::default()).collect();

    for monster in &mut monsters {
        let counter = Rc::clone(&counter);
        monster.add_listener(Box::new( 
            move |damage| {
                counter.borrow_mut().on_damage_received(damage)
            }
        ));
    }

    while !counter.borrow().reached_target_damage() {
        let index = rng.gen_range(0..monsters.len());
        let target = &mut monsters[index];

        let damage = rng.gen_range(0..50);
        target.take_damage(damage);

        println!("Monster {} received {} damage", index, damage);
    }
}

#[test]
fn test_example2() {
    let mut rng = rand::thread_rng();
    let counter = Mrc::new(DamageCounter::default());
    let mut monsters: Vec<_> = (0..5).map(|_| Monster::default()).collect();

    for monster in &mut monsters {
        let counter = counter.clone();
        monster.add_listener(Box::new( 
            move |damage| {
                unsafe { counter.to_mut().on_damage_received(damage) }
            }
        ));
    }

    while !counter.reached_target_damage() {
        let index = rng.gen_range(0..monsters.len());
        let target = &mut monsters[index];

        let damage = rng.gen_range(0..50);
        target.take_damage(damage);

        println!("Monster {} received {} damage", index, damage);
    }
}
