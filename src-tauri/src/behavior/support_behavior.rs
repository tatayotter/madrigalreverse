use std::time::{ Duration, Instant };

use slog::Logger;
use tauri::Window;

use super::Behavior;

use crate::{
    data::{ AliveState, Point },
    duo::{ self, HealerStatus },
    image_analyzer::ImageAnalyzer,
    ipc::{ BotConfig, FarmingConfig, FrontendInfo, SlotType, SupportConfig },
    movement::{ prelude::*, MovementAccessor },
    platform::{ eval_simple_click, send_slot_eval },
    play,
};
const HEAL_SKILL_CAST_TIME: u64 = 2000;
const BUFF_CAST_TIME: u64 = 2500;
const AOE_SKILL_CAST_TIME: u64 = 100;
/// Duo: how often the healer writes its status for the farmer.
const DUO_PUBLISH_INTERVAL_MS: u128 = 250;
/// Duo: how often the healer scans its screen for violets/giants.
const DUO_DANGER_SCAN_INTERVAL_MS: u128 = 1000;
/// Duo: how close (screen pixels) a violet/giant must be for the healer to warn.
const DUO_DANGER_RADIUS: i32 = 300;
/// Duo: the healer counts as "being hit" if its HP dropped within this window.
const DUO_UNDER_ATTACK_WINDOW_MS: u128 = 2000;
pub struct SupportBehavior<'a> {
    logger: &'a Logger,
    movement: &'a MovementAccessor,
    window: &'a Window,
    self_buff_usage_last_time: [[Option<Instant>; 10]; 9],
    slots_usage_last_time: [[Option<Instant>; 10]; 9],
    last_jump_time: Instant,
    avoid_obstacle_direction: String,

    is_waiting_for_revive: bool,

    last_far_from_target: Option<Instant>,
    last_target_distance: Option<i32>,

    wait_duration: Option<Duration>,
    wait_start: Instant,
    has_target: bool,
    self_buffing: bool,
    target_buffing: bool,
    buff_counter: u32,

    leader_in_range: bool,
    last_hp_seen: u32,
    last_hp_drop: Option<Instant>,
    last_duo_publish: Option<Instant>,
    last_danger_scan: Option<Instant>,
    danger_ms: u64,
}

impl<'a> Behavior<'a> for SupportBehavior<'a> {
    fn new(logger: &'a Logger, movement: &'a MovementAccessor, window: &'a Window) -> Self {
        Self {
            logger,
            movement,
            window,
            self_buff_usage_last_time: [[None; 10]; 9],
            slots_usage_last_time: [[None; 10]; 9],
            last_jump_time: Instant::now(),
            avoid_obstacle_direction: "D".to_owned(),
            is_waiting_for_revive: false,
            last_far_from_target: None,
            last_target_distance: None,
            wait_duration: None,
            wait_start: Instant::now(),
            has_target: false,
            self_buffing: false,
            target_buffing: false,
            buff_counter: 0,
            leader_in_range: false,
            last_hp_seen: 0,
            last_hp_drop: None,
            last_duo_publish: None,
            last_danger_scan: None,
            danger_ms: 0,
        }
    }

    fn start(&mut self, _config: &BotConfig) {}
    fn update(&mut self, _config: &BotConfig) {}
    fn stop(&mut self, _config: &BotConfig) {
        self.slots_usage_last_time = [[None; 10]; 9];
        self.self_buff_usage_last_time = [[None; 10]; 9];
        self.has_target = false;
        self.self_buffing = false;
        self.wait_duration = None;
        self.wait_start = Instant::now();
        slog::debug!(self.logger, "SupportBehavior stopped");
    }

    fn interupt(&mut self, config: &BotConfig) {
        self.stop(config);
    }

    fn run_iteration(
        &mut self,
        _frontend_info: &mut FrontendInfo,
        config: &BotConfig,
        image: &mut ImageAnalyzer
    ) {
        let config = config.support_config();
        self.update_slots_usage(config);

        self.has_target = image.client_stats.target_is_mover;
        self.duo_tick(config, image);

        self.use_party_skills(config);
        self.check_self_restorations(config, image);

        if self.has_target == false {
            self.leader_in_range = false;
            if config.is_in_party() {
                self.select_party_leader(config);
            }
            return;
        }

        self.follow_target();
        if self.rez_target(config, image) {
            if self.is_waiting_for_revive {
                if image.client_stats.target_hp.value > 0 {
                    self.is_waiting_for_revive = false;
                }
            } else {
                self.is_waiting_for_revive = true;
                self.slots_usage_last_time = [[None; 10]; 9];
                return;
            }
        }

        self.check_target_restorations(config, image);
        if image.client_stats.target_on_screen {
            let dist = self.is_target_in_range(config, image);
            self.leader_in_range = dist;
            if dist == false {
                return;
            }
        } else {
            self.leader_in_range = false;
        }

        if self.wait_cooldown() {
            return;
        }
        self.random_camera_movement();

        if config.is_in_party() {
            let self_buff = self.get_slot_for(
                config,
                None,
                SlotType::BuffSkill,
                false,
                Some(self.self_buff_usage_last_time)
            );

            self.send_buff(config, self_buff, true);
        }

        let target_buff = self.get_slot_for(config, None, SlotType::BuffSkill, false, None);

        if self.self_buffing == false && image.client_stats.target_is_alive {
            self.send_buff(config, target_buff, false);
        }
    }
}

impl SupportBehavior<'_> {
    /// Duo link: look for nearby violets/giants and share our status with the farmer.
    fn duo_tick(&mut self, config: &SupportConfig, image: &ImageAnalyzer) {
        if !config.duo_link() {
            return;
        }
        let hp = image.client_stats.hp.value;
        if hp < self.last_hp_seen {
            self.last_hp_drop = Some(Instant::now());
        }
        self.last_hp_seen = hp;
        if self.last_danger_scan.map_or(true, |t| t.elapsed().as_millis() >= DUO_DANGER_SCAN_INTERVAL_MS) {
            self.last_danger_scan = Some(Instant::now());
            // Default name colors; the healer has no mob color settings of its own.
            if image.find_danger_near(&FarmingConfig::default(), DUO_DANGER_RADIUS).is_some() {
                self.danger_ms = duo::now_ms();
                slog::info!(self.logger, "Healer sees a violet/giant nearby, telling the farmer");
            }
        }
        if self.last_duo_publish.map_or(true, |t| t.elapsed().as_millis() >= DUO_PUBLISH_INTERVAL_MS) {
            self.last_duo_publish = Some(Instant::now());
            duo::publish_healer(&HealerStatus {
                updated_ms: duo::now_ms(),
                alive: image.client_stats.is_alive == AliveState::Alive,
                hp: image.client_stats.hp.value,
                mp: image.client_stats.mp.value,
                has_leader: self.has_target,
                leader_in_range: self.leader_in_range,
                danger_ms: self.danger_ms,
                under_attack: self
                    .last_hp_drop
                    .map_or(false, |t| t.elapsed().as_millis() < DUO_UNDER_ATTACK_WINDOW_MS),
            });
        }
    }

    fn random_camera_movement(&mut self) {
        //add movement every minute to try to avoid bot detection
        if self.last_jump_time.elapsed().as_millis() > 10000 {
            use crate::movement::prelude::*;
            play!(self.movement => [
                // Rotate in random direction for a random duration
                Rotate(rot::Right, dur::Fixed(50)),
                // Wait a bit to wait for monsters to enter view
                Wait(dur::Fixed(50)),
            ]);

            self.last_jump_time = Instant::now();
        }
    }
    fn get_target_distance(&mut self, image: &mut ImageAnalyzer) -> Option<i32> {
        if let Some(target_distance) = image.client_stats.target_distance {
            return Some(target_distance);
        } else {
            return Some(9999);
        }
    }
    fn move_circle_pattern(&mut self) {
        use crate::movement::prelude::*;
        play!(self.movement => [
            HoldKeys(vec!["W", "Space", &self.avoid_obstacle_direction]),
            Wait(dur::Fixed(100)),
            ReleaseKey(&self.avoid_obstacle_direction),
            Wait(dur::Fixed(500)),
            ReleaseKeys(vec!["Space", "W"]),
            PressKey("Z"),
        ]);

        self.avoid_obstacle_direction = {
            if self.avoid_obstacle_direction == "D" { "A".to_owned() } else { "D".to_owned() }
        };
    }
    fn is_target_in_range(&mut self, config: &SupportConfig, image: &mut ImageAnalyzer) -> bool {
        let distance = self.get_target_distance(image);
        if let Some(distance) = distance {
            if distance == 9999 {
                self.move_circle_pattern();
                return false;
            }

            if distance > (config.get_max_main_distance() as i32) {
                if let Some(last_target_distance) = self.last_target_distance {
                    if distance > (config.get_max_main_distance() as i32) * 2 {
                        self.move_circle_pattern();
                    } else {
                        if let Some(last_far_from_target) = self.last_far_from_target {
                            if
                                last_far_from_target.elapsed().as_millis() > 3000 &&
                                last_target_distance < distance
                            {
                                self.last_far_from_target = Some(Instant::now());
                                self.move_circle_pattern();
                            }
                        } else {
                            self.last_far_from_target = Some(Instant::now());
                        }
                    }
                }
                self.last_target_distance = Some(distance);
                self.follow_target();

                return false;
            } else {
                self.last_far_from_target = None; //Some(Instant::now());
                return true;
            }
        }
        return false;
    }
    fn send_buff(
        &mut self,
        config: &SupportConfig,
        buff: Option<(usize, usize)>,
        is_self_buff: bool
    ) {
        if buff.is_some() {
            if is_self_buff {
                if self.self_buffing == false {
                    self.self_buffing = true;
                    slog::debug!(self.logger, "Starting self buffing");
                    self.buff_counter = 0;
                }
                self.lose_target();
            } else {
                if self.target_buffing == false {
                    self.target_buffing = true;
                    slog::debug!(self.logger, "Starting target buffing");
                    self.buff_counter = 0;
                }
            }
            let slot = buff.unwrap();
            self.send_slot(slot, is_self_buff);
            self.buff_counter += 1;
            self.wait(Duration::from_millis(BUFF_CAST_TIME));
        } else if is_self_buff {
            if self.self_buffing {
                self.self_buffing = false;
                slog::debug!(self.logger, "Ending self buffing {}", self.buff_counter);
                self.select_party_leader(config);
            }
        } else if is_self_buff == false {
            if self.target_buffing {
                self.target_buffing = false;
                slog::debug!(self.logger, "Ending target buffing {}", self.buff_counter);
            }
        }
    }
    fn wait_cooldown(&mut self) -> bool {
        if self.wait_duration.is_some() {
            if self.wait_start.elapsed() < self.wait_duration.unwrap() {
                return true;
            } else {
                self.wait_duration = None;
            }
        }
        return false;
    }
    fn wait(&mut self, duration: Duration) {
        self.wait_duration = {
            if self.wait_duration.is_some() {
                Some(self.wait_duration.unwrap() + duration)
            } else {
                self.wait_start = Instant::now();
                Some(duration)
            }
        };
    }

    fn rez_target(&mut self, config: &SupportConfig, image: &mut ImageAnalyzer) -> bool {
        if image.client_stats.target_is_mover && image.client_stats.target_is_alive == false {
            self.get_slot_for(config, None, SlotType::RezSkill, true, None);
            return true;
        } else {
            return false;
        }
    }
    fn lose_target(&mut self) {
        if self.has_target {
            play!(self.movement => [
               PressKey("Escape"),
               Wait(dur::Random(200..250)),
           ]);
        }
    }

    fn select_party_leader(&mut self, _config: &SupportConfig) {
        //attempt to get party leader
        play!(self.movement => [
            // Open party menu
            PressKey("P"),
        ]);
        std::thread::sleep(Duration::from_millis(150));
        let point = Point::new(213, 440); //moving to the "position of the party window
        eval_simple_click(self.window, point);
        play!(self.movement => [
        PressKey("Z"),
        Wait(dur::Fixed(10)),
        PressKey("P"),
        ]);
        std::thread::sleep(Duration::from_millis(500));
    }

    fn follow_target(&mut self) {
        if self.has_target {
            play!(self.movement => [
                PressKey("Z"),
            ]);
        }
    }

    /// Update slots cooldown timers
    fn update_slots_usage(&mut self, config: &SupportConfig) {
        for (slotbar_index, slot_bars) in self.slots_usage_last_time.into_iter().enumerate() {
            for (slot_index, last_time) in slot_bars.into_iter().enumerate() {
                let cooldown = u128::from(config
                    .get_slot_cooldown(slotbar_index, slot_index)
                    .unwrap_or(100));
                if let Some(last_time) = last_time {
                    let slot_last_time = last_time.elapsed().as_millis();
                    if slot_last_time > cooldown {
                        self.slots_usage_last_time[slotbar_index][slot_index] = None;
                    }
                }
            }
        }
        for (slotbar_index, slot_bars) in self.self_buff_usage_last_time.into_iter().enumerate() {
            for (slot_index, last_time) in slot_bars.into_iter().enumerate() {
                let cooldown = u128::from(config
                    .get_slot_cooldown(slotbar_index, slot_index)
                    .unwrap_or(100));
                if let Some(last_time) = last_time {
                    let slot_last_time = last_time.elapsed().as_millis();
                    if slot_last_time > cooldown {
                        self.self_buff_usage_last_time[slotbar_index][slot_index] = None;
                    }
                }
            }
        }
    }

    fn get_slot_for(
        &mut self,
        config: &SupportConfig,
        threshold: Option<u32>,
        slot_type: SlotType,
        send: bool,
        last_slots_usage: Option<[[Option<Instant>; 10]; 9]>
    ) -> Option<(usize, usize)> {
        let is_self_buff = {
            if let Some(_) = last_slots_usage { true } else { false }
        };
        let slot_usage = {
            if let Some(last_slots_usage) = last_slots_usage {
                last_slots_usage
            } else {
                self.slots_usage_last_time
            }
        };
        if let Some(slot_index) = config.get_usable_slot_index(slot_type, threshold, slot_usage) {
            if send {
                self.send_slot(slot_index, is_self_buff);
            }
            return Some(slot_index);
        }
        None
    }

    fn send_slot(&mut self, slot_index: (usize, usize), is_self_buff: bool) {
        // Send keystroke for first slot mapped to pill
        send_slot_eval(self.window, slot_index.0, slot_index.1);
        // Update usage last time
        if is_self_buff {
            self.self_buff_usage_last_time[slot_index.0][slot_index.1] = Some(Instant::now());
        } else {
            self.slots_usage_last_time[slot_index.0][slot_index.1] = Some(Instant::now());
        }
    }

    fn use_party_skills(&mut self, config: &SupportConfig) {
        let party_skills = config.get_all_usable_slot_for_type(
            SlotType::PartySkill,
            self.slots_usage_last_time
        );
        for slot_index in party_skills {
            self.send_slot(slot_index, false);
        }
    }

    fn check_self_restorations(&mut self, config: &SupportConfig, image: &mut ImageAnalyzer) {
        let health_stat = Some(image.client_stats.hp.value);
        // Use a HealSkill if configured when health is under 85
        let pill = self.get_slot_for(config, health_stat, SlotType::Pill, true, None);
        if pill.is_none() {
            let heal = self.get_slot_for(config, health_stat, SlotType::HealSkill, false, None);
            if heal.is_none() {
                let aoe_heal = self.get_slot_for(
                    config,
                    health_stat,
                    SlotType::AOEHealSkill,
                    true,
                    None
                );
                if aoe_heal.is_none() {
                    self.get_slot_for(config, health_stat, SlotType::Food, true, None);
                } else {
                    std::thread::sleep(Duration::from_millis(AOE_SKILL_CAST_TIME));
                    self.get_slot_for(config, health_stat, SlotType::AOEHealSkill, true, None);
                    std::thread::sleep(Duration::from_millis(AOE_SKILL_CAST_TIME));
                    self.get_slot_for(config, health_stat, SlotType::AOEHealSkill, true, None);
                }
            } else {
                if config.is_in_party() {
                    self.lose_target();
                    self.send_slot(heal.unwrap(), true);
                }
            }

            // Check MP
            let mp_stat = Some(image.client_stats.mp.value);
            self.get_slot_for(config, mp_stat, SlotType::MpRestorer, true, None);

            // Check FP

            let fp_stat = Some(image.client_stats.fp.value);
            self.get_slot_for(config, fp_stat, SlotType::FpRestorer, true, None);
        }
    }

    fn check_target_restorations(&mut self, config: &SupportConfig, image: &mut ImageAnalyzer) {
        let target_health_stat = Some(image.client_stats.target_hp.value);
        // Use a HealSkill if configured when health is under 85

        let heal = self.get_slot_for(config, target_health_stat, SlotType::HealSkill, true, None);
        if heal.is_none() {
            let aoe_heal = self.get_slot_for(
                config,
                target_health_stat,
                SlotType::AOEHealSkill,
                true,
                None
            );
            if aoe_heal.is_some() {
                self.get_slot_for(config, target_health_stat, SlotType::AOEHealSkill, true, None);
                std::thread::sleep(Duration::from_millis(100));
                self.get_slot_for(config, target_health_stat, SlotType::AOEHealSkill, true, None);
                std::thread::sleep(Duration::from_millis(100));
            }
        } else {
        }
    }
}
