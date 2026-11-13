import IconFood from '../assets/slots/slot_food.png'
import IconPill from '../assets/slots/slot_pill.png'
import IconHeal from '../assets/slots/slot_heal.png'
import IconAoeHeal from '../assets/slots/slot_aoe_heal.png'
import IconMp from '../assets/slots/slot_mp.png'
import IconFp from '../assets/slots/slot_fp.png'
import IconPet from '../assets/slots/slot_pet.png'
import IconPickup from '../assets/slots/slot_pickup.png'
import IconAttack from '../assets/slots/slot_attack.png'
import IconAoeAttack from '../assets/slots/slot_aoe_attack.png'
import IconBuff from '../assets/slots/slot_buff.png'
import IconRez from '../assets/slots/slot_rez.png'
import IconBoard from '../assets/slots/slot_board.png'
import IconParty from '../assets/slots/slot_party.png'

export type FixedArray<TItem, TLength extends number> = [TItem, ...TItem[]] & { length: TLength }
export const slotTypes = ["Unused", "Food", "Pill", "HealSkill","AOEHealSkill", "MpRestorer",
                                                    "FpRestorer", "PickupPet", "PickupMotion", "AttackSkill",
                                                    "AOEAttackSkill", "BuffSkill", "RezSkill", "Flying", "PartySkill"] as const;
export const thresholdSlotTypes = ["Food", "Pill","HealSkill","MpRestorer", "FpRestorer", 'AOEHealSkill'];
export const cooldownSlotTypes = ["Food", "Pill", "HealSkill", "AOEHealSkill", "AttackSkill","AOEAttackSkill", "BuffSkill", "MpRestorer", "FpRestorer", "PickupPet","PartySkill"];
// "Flying" (Board) is allowed for farming: the duo escape gets on the board when the healer dies
export const farmingSlotsBlacklist = ["RezSkill","AOEHealSkill"]
//if we are leveling someone, the support role might want to pick up
export const supportSlotsBlacklist = ["AttackSkill","AOEAttackSkill"]


export type SlotType = typeof slotTypes[number];

export const createSlotBars = () => (
    [...new Array(9)].map(_ => ({slots:[...new Array(10)].map(_ => ({ slot_type: 'Unused', slot_enabled: false } as SlotModel))})) as SlotBars
)

export const SLOT_SIZE_PX = 40;

export const translateType = (type: SlotType) => {
    switch (type) {
        case 'Unused': return ''
        case 'Food': return IconFood
        case 'Pill': return IconPill
        case 'HealSkill': return IconHeal
        case 'AOEHealSkill': return IconAoeHeal
        case 'MpRestorer': return IconMp
        case 'FpRestorer': return IconFp
        case 'PickupPet': return IconPet
        case 'PickupMotion': return IconPickup
        case 'AttackSkill': return IconAttack
        case 'AOEAttackSkill': return IconAoeAttack
        case 'BuffSkill': return IconBuff
        case 'RezSkill': return IconRez
        case 'Flying': return IconBoard
        case 'PartySkill': return IconParty
    }
}

export const translateDesc = (type: SlotType, defaultUnused: string = '') => {
    switch (type) {
        case 'Unused': return [defaultUnused, defaultUnused]
        case 'Food': return ['Food','Food']
        case 'Pill': return ['Pill','Pill']
        case 'HealSkill': return ["Heal",'Heal skill']
        case 'AOEHealSkill': return ['AOE H','AOE Heal skill']
        case 'MpRestorer': return ['MP', 'MP restorer']
        case 'FpRestorer': return ['FP', 'FP restorer']
        case 'PickupPet': return ['Pet', 'Pickup pet']
        case 'PickupMotion': return ['Pickup', 'Pickup motion']
        case 'AttackSkill': return ['Attack', 'Attack skill']
        case 'AOEAttackSkill': return ['AOE', 'AOE Attack skill']
        case 'BuffSkill': return ['Buff', 'Buff skill']
        case 'RezSkill': return ['Rez', 'Resurrection skill']
        case 'Flying': return ['Board', 'Board']
        case 'PartySkill': return ['Party', 'Party skill']
    }
}
export type SlotModel = {
    slot_type: SlotType,
    slot_cooldown?: number,
    slot_threshold?: number,
    slot_enabled: boolean,
}
export type SlotBarHolder = {
    slots: SlotBarModel
}

type SlotBarModel = FixedArray<SlotModel, 10>
export type SlotBars = FixedArray<SlotBarHolder, 9>

export type ModeModel = "Farming" | "Support" | "AutoShout"

export type FarmingConfigModel = Partial<{
    [key: string]: any;
    on_demand_pet: boolean,
    use_attack_skills: boolean,
    stay_in_area: boolean,
    slot_bars: SlotBars,
    circle_pattern_rotation_duration: number,

    passive_mobs_colors: number[];
    passive_tolerence: number;
    aggressive_mobs_colors: number[];
    aggressive_tolerence: number;

    is_stop_fighting: boolean;
    prevent_already_attacked: boolean;
    prioritize_aggro: boolean;

    obstacle_avoidance_cooldown: number,
    obstacle_avoidance_max_try: number,

    min_mobs_name_width: number,
    max_mobs_name_width: number,

    min_hp_attack: number,
    on_death_disconnect: boolean,
    interval_between_buffs: number,
    mobs_timeout: number,
    aoe_farming: number,

    on_afk_disconnect: boolean,
    afk_timeout: number,
    afk_ready_to_disconnect: boolean,

    area_radius: number,
    area_check_interval: number,
    revive_return: boolean,
    revive_rest: number,
    revive_max_distance: number,

    giant_flee: boolean,
    flee_radius: number,
    giant_mobs_colors: number[],
    giant_tolerence: number,


    duo_wait_for_healer: boolean,
    duo_healer_min_hp: number,
    duo_healer_min_mp: number,
    duo_protect_healer: boolean,
    duo_escape_on_healer_death: boolean,

    name_reading: boolean,
    monster_whitelist: string[],
    monster_blacklist: string[],
}>

export type SupportConfigModel = Partial<{
    [key: string]: any;
    slot_bars: SlotBars,
    obstacle_avoidance_cooldown: number,
    interval_between_buffs: number,
    is_in_party: boolean,
    on_death_disconnect: boolean,

    on_afk_disconnect: boolean,
    afk_timeout: number,
    afk_ready_to_disconnect: boolean,
    max_main_distance: number,
    duo_link: boolean,
}>

export type ShoutConfigModel = Partial<{
    [key: string]: any;
    shout_interval: number,
    shout_messages: string[],

    on_afk_disconnect: boolean,
    afk_timeout: number,
    afk_ready_to_disconnect: boolean,
}>

export type BotConfigModel = {
    change_id: number,
    is_running: boolean,
    mode?: ModeModel,
    farming_config: FarmingConfigModel,
    support_config: SupportConfigModel,
    shout_config: ShoutConfigModel,
}

export type AnyConfig = FarmingConfigModel | SupportConfigModel | ShoutConfigModel
