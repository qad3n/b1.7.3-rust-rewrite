use mod_protocol::abi::RegistryKind;

pub(crate) struct VanillaTag
{
    pub(crate) m_registry: RegistryKind,
    pub(crate) m_name: &'static str,
    pub(crate) m_members: &'static [&'static str],
}

const fn block(name: &'static str, members: &'static [&'static str]) -> VanillaTag
{
    VanillaTag { m_registry: RegistryKind::Block, m_name: name, m_members: members }
}

const fn biome(name: &'static str, members: &'static [&'static str]) -> VanillaTag
{
    VanillaTag { m_registry: RegistryKind::Biome, m_name: name, m_members: members }
}

const fn fluid(name: &'static str, members: &'static [&'static str]) -> VanillaTag
{
    VanillaTag { m_registry: RegistryKind::Fluid, m_name: name, m_members: members }
}

const fn damage(name: &'static str, members: &'static [&'static str]) -> VanillaTag
{
    VanillaTag { m_registry: RegistryKind::DamageType, m_name: name, m_members: members }
}

const fn entity(name: &'static str, members: &'static [&'static str]) -> VanillaTag
{
    VanillaTag { m_registry: RegistryKind::Entity, m_name: name, m_members: members }
}

const fn item(name: &'static str, members: &'static [&'static str]) -> VanillaTag
{
    VanillaTag { m_registry: RegistryKind::Item, m_name: name, m_members: members }
}

const ORES: &[&str] =
    &["minecraft:coal_ore", "minecraft:iron_ore", "minecraft:gold_ore", "minecraft:diamond_ore", "minecraft:lapis_ore"];
const REDSTONE_ORES: &[&str] = &["minecraft:redstone_ore", "minecraft:lit_redstone_ore"];
const STORAGE: &[&str] =
    &["minecraft:iron_block", "minecraft:gold_block", "minecraft:diamond_block", "minecraft:lapis_block"];
const PICKAXES: &[&str] = &[
    "minecraft:wooden_pickaxe",
    "minecraft:stone_pickaxe",
    "minecraft:iron_pickaxe",
    "minecraft:golden_pickaxe",
    "minecraft:diamond_pickaxe",
];
const AXES: &[&str] = &[
    "minecraft:wooden_axe",
    "minecraft:stone_axe",
    "minecraft:iron_axe",
    "minecraft:golden_axe",
    "minecraft:diamond_axe",
];
const SHOVELS: &[&str] = &[
    "minecraft:wooden_shovel",
    "minecraft:stone_shovel",
    "minecraft:iron_shovel",
    "minecraft:golden_shovel",
    "minecraft:diamond_shovel",
];
const HOES: &[&str] = &[
    "minecraft:wooden_hoe",
    "minecraft:stone_hoe",
    "minecraft:iron_hoe",
    "minecraft:golden_hoe",
    "minecraft:diamond_hoe",
];
const SWORDS: &[&str] = &[
    "minecraft:wooden_sword",
    "minecraft:stone_sword",
    "minecraft:iron_sword",
    "minecraft:golden_sword",
    "minecraft:diamond_sword",
];

// modern minecraft and c tags over beta content so mods name groups instead of listing vanilla blocks
pub(crate) const TAGS: &[VanillaTag] = &[
    // damage types grouped as later versions group them, beta armor still reduces every one
    damage("minecraft:is_fire", &["minecraft:in_fire", "minecraft:on_fire", "minecraft:lava", "minecraft:fireball"]),
    damage("minecraft:is_explosion", &["minecraft:explosion"]),
    damage("minecraft:is_projectile", &["minecraft:arrow", "minecraft:thrown", "minecraft:fireball"]),
    damage("minecraft:is_fall", &["minecraft:fall"]),
    damage("minecraft:is_drowning", &["minecraft:drown"]),
    damage("minecraft:bypasses_armor", &[]),
    // behaviour toward players so mods tell hostile from neutral from passive without naming mobs
    // spider stays hostile alone since it hunts on sight in the dark where it spawns and only daylight calms it
    entity(
        "c:hostile",
        &[
            "minecraft:monster",
            "minecraft:creeper",
            "minecraft:skeleton",
            "minecraft:spider",
            "minecraft:giant",
            "minecraft:zombie",
            "minecraft:slime",
            "minecraft:ghast",
        ],
    ),
    entity("c:neutral", &["minecraft:pig_zombie", "minecraft:wolf"]),
    entity("c:passive", &["minecraft:pig", "minecraft:sheep", "minecraft:cow", "minecraft:chicken", "minecraft:squid"]),
    // the giant is no undead in modern minecraft nor in the 1.7 mods that read this tag
    entity("minecraft:undead", &["minecraft:skeleton", "minecraft:zombie", "minecraft:pig_zombie"]),
    entity("minecraft:arthropod", &["minecraft:spider"]),
    entity("minecraft:skeletons", &["minecraft:skeleton"]),
    entity("minecraft:zombies", &["minecraft:zombie", "minecraft:pig_zombie"]),
    // fluid tags name fluids, so mods' oils and fuels join shared groups machines take
    fluid("minecraft:water", &["minecraft:water"]),
    fluid("minecraft:lava", &["minecraft:lava"]),
    fluid("c:oil", &[]),
    fluid("c:fuel", &[]),
    biome(
        "minecraft:is_overworld",
        &[
            "minecraft:rainforest",
            "minecraft:swampland",
            "minecraft:seasonal_forest",
            "minecraft:forest",
            "minecraft:savanna",
            "minecraft:shrubland",
            "minecraft:taiga",
            "minecraft:desert",
            "minecraft:plains",
            "minecraft:ice_desert",
            "minecraft:tundra",
        ],
    ),
    biome("minecraft:is_nether", &["minecraft:hell"]),
    biome("minecraft:is_forest", &["minecraft:forest", "minecraft:seasonal_forest", "minecraft:rainforest"]),
    biome("minecraft:is_taiga", &["minecraft:taiga"]),
    biome("minecraft:is_jungle", &["minecraft:rainforest"]),
    biome("minecraft:is_savanna", &["minecraft:savanna"]),
    biome("c:is_swamp", &["minecraft:swampland"]),
    biome("c:is_desert", &["minecraft:desert", "minecraft:ice_desert"]),
    biome("c:is_plains", &["minecraft:plains", "minecraft:shrubland"]),
    biome("c:is_snowy", &["minecraft:taiga", "minecraft:ice_desert", "minecraft:tundra"]),
    block("minecraft:dirt", &["minecraft:dirt", "minecraft:grass"]),
    block("minecraft:sand", &["minecraft:sand"]),
    block("minecraft:wool", &["minecraft:wool"]),
    block("minecraft:planks", &["minecraft:planks"]),
    block("minecraft:saplings", &["minecraft:sapling"]),
    block("minecraft:small_flowers", &["minecraft:yellow_flower", "minecraft:red_flower"]),
    block("minecraft:flowers", &["minecraft:yellow_flower", "minecraft:red_flower"]),
    block("minecraft:base_stone_overworld", &["minecraft:stone"]),
    block("minecraft:base_stone_nether", &["minecraft:netherrack"]),
    block("minecraft:ice", &["minecraft:ice"]),
    block("minecraft:snow", &["minecraft:snow", "minecraft:snow_layer"]),
    block("minecraft:fire", &["minecraft:fire"]),
    block("minecraft:water", &["minecraft:water", "minecraft:flowing_water"]),
    block("minecraft:lava", &["minecraft:lava", "minecraft:flowing_lava"]),
    block("minecraft:rails", &["minecraft:rail", "minecraft:golden_rail", "minecraft:detector_rail"]),
    block("minecraft:doors", &["minecraft:wooden_door", "minecraft:iron_door"]),
    block("minecraft:wooden_doors", &["minecraft:wooden_door"]),
    block("minecraft:stairs", &["minecraft:oak_stairs", "minecraft:stone_stairs"]),
    block("minecraft:wooden_stairs", &["minecraft:oak_stairs"]),
    block("minecraft:slabs", &["minecraft:stone_slab"]),
    block("minecraft:pressure_plates", &["minecraft:stone_pressure_plate", "minecraft:wooden_pressure_plate"]),
    block("minecraft:buttons", &["minecraft:stone_button"]),
    block("minecraft:beds", &["minecraft:bed"]),
    block("minecraft:crops", &["minecraft:wheat"]),
    block("minecraft:portals", &["minecraft:portal"]),
    block("minecraft:signs", &["minecraft:standing_sign", "minecraft:wall_sign"]),
    block("minecraft:standing_signs", &["minecraft:standing_sign"]),
    block("minecraft:wall_signs", &["minecraft:wall_sign"]),
    block(
        "minecraft:replaceable",
        &[
            "minecraft:tallgrass",
            "minecraft:deadbush",
            "minecraft:snow_layer",
            "minecraft:fire",
            "minecraft:water",
            "minecraft:flowing_water",
            "minecraft:lava",
            "minecraft:flowing_lava",
        ],
    ),
    block("c:ores", ORES),
    block("c:ores", REDSTONE_ORES),
    block("c:ores/coal", &["minecraft:coal_ore"]),
    block("c:ores/iron", &["minecraft:iron_ore"]),
    block("c:ores/gold", &["minecraft:gold_ore"]),
    block("c:ores/diamond", &["minecraft:diamond_ore"]),
    block("c:ores/lapis", &["minecraft:lapis_ore"]),
    block("c:ores/redstone", REDSTONE_ORES),
    block("c:storage_blocks", STORAGE),
    block("c:storage_blocks/iron", &["minecraft:iron_block"]),
    block("c:storage_blocks/gold", &["minecraft:gold_block"]),
    block("c:storage_blocks/diamond", &["minecraft:diamond_block"]),
    block("c:storage_blocks/lapis", &["minecraft:lapis_block"]),
    block("c:glass_blocks", &["minecraft:glass"]),
    block("c:chests", &["minecraft:chest"]),
    block("c:cobblestones", &["minecraft:cobblestone", "minecraft:mossy_cobblestone"]),
    block("c:sandstone_blocks", &["minecraft:sandstone"]),
    block("c:obsidians", &["minecraft:obsidian"]),
    block("c:gravels", &["minecraft:gravel"]),
    block("c:pumpkins", &["minecraft:pumpkin", "minecraft:lit_pumpkin"]),
    block("c:player_workstations/crafting_tables", &["minecraft:crafting_table"]),
    block("c:player_workstations/furnaces", &["minecraft:furnace", "minecraft:lit_furnace"]),
    item("minecraft:planks", &["minecraft:planks"]),
    item("minecraft:wool", &["minecraft:wool"]),
    item("minecraft:saplings", &["minecraft:sapling"]),
    item("minecraft:logs", &["minecraft:log"]),
    item("minecraft:leaves", &["minecraft:leaves"]),
    item("minecraft:sand", &["minecraft:sand"]),
    item("minecraft:small_flowers", &["minecraft:yellow_flower", "minecraft:red_flower"]),
    item("minecraft:rails", &["minecraft:rail", "minecraft:golden_rail", "minecraft:detector_rail"]),
    item("minecraft:doors", &["minecraft:wooden_door", "minecraft:iron_door"]),
    item("minecraft:stairs", &["minecraft:oak_stairs", "minecraft:stone_stairs"]),
    item("minecraft:slabs", &["minecraft:stone_slab"]),
    item("minecraft:coals", &["minecraft:coal"]),
    item("minecraft:pickaxes", PICKAXES),
    item("minecraft:axes", AXES),
    item("minecraft:shovels", SHOVELS),
    item("minecraft:hoes", HOES),
    item("minecraft:swords", SWORDS),
    item(
        "minecraft:head_armor",
        &[
            "minecraft:leather_helmet",
            "minecraft:chainmail_helmet",
            "minecraft:iron_helmet",
            "minecraft:golden_helmet",
            "minecraft:diamond_helmet",
        ],
    ),
    item(
        "minecraft:chest_armor",
        &[
            "minecraft:leather_chestplate",
            "minecraft:chainmail_chestplate",
            "minecraft:iron_chestplate",
            "minecraft:golden_chestplate",
            "minecraft:diamond_chestplate",
        ],
    ),
    item(
        "minecraft:leg_armor",
        &[
            "minecraft:leather_leggings",
            "minecraft:chainmail_leggings",
            "minecraft:iron_leggings",
            "minecraft:golden_leggings",
            "minecraft:diamond_leggings",
        ],
    ),
    item(
        "minecraft:foot_armor",
        &[
            "minecraft:leather_boots",
            "minecraft:chainmail_boots",
            "minecraft:iron_boots",
            "minecraft:golden_boots",
            "minecraft:diamond_boots",
        ],
    ),
    item("c:ores", ORES),
    item("c:ores", REDSTONE_ORES),
    item("c:ores/coal", &["minecraft:coal_ore"]),
    item("c:ores/iron", &["minecraft:iron_ore"]),
    item("c:ores/gold", &["minecraft:gold_ore"]),
    item("c:ores/diamond", &["minecraft:diamond_ore"]),
    item("c:ores/lapis", &["minecraft:lapis_ore"]),
    item("c:ores/redstone", &["minecraft:redstone_ore"]),
    item("c:storage_blocks", STORAGE),
    item("c:storage_blocks/iron", &["minecraft:iron_block"]),
    item("c:storage_blocks/gold", &["minecraft:gold_block"]),
    item("c:storage_blocks/diamond", &["minecraft:diamond_block"]),
    item("c:storage_blocks/lapis", &["minecraft:lapis_block"]),
    item("c:gravels", &["minecraft:gravel"]),
    item("c:music_discs", &["minecraft:record_13", "minecraft:record_cat"]),
    item("minecraft:ice", &["minecraft:ice"]),
    item("c:ingots", &["minecraft:iron_ingot", "minecraft:gold_ingot"]),
    item("c:ingots/iron", &["minecraft:iron_ingot"]),
    item("c:ingots/gold", &["minecraft:gold_ingot"]),
    item("c:gems", &["minecraft:diamond"]),
    item("c:gems/diamond", &["minecraft:diamond"]),
    item("c:dusts/redstone", &["minecraft:redstone"]),
    item("c:dusts/glowstone", &["minecraft:glowstone_dust"]),
    item("c:rods/wooden", &["minecraft:stick"]),
    item("c:strings", &["minecraft:string"]),
    item("c:seeds/wheat", &["minecraft:wheat_seeds"]),
    item("c:crops/wheat", &["minecraft:wheat"]),
    item("c:bones", &["minecraft:bone"]),
    item("c:gunpowders", &["minecraft:gunpowder"]),
    item("c:slime_balls", &["minecraft:slime_ball"]),
    item("c:leathers", &["minecraft:leather"]),
    item("c:feathers", &["minecraft:feather"]),
    item("c:eggs", &["minecraft:egg"]),
    item("c:flints", &["minecraft:flint"]),
    item("c:bricks", &["minecraft:brick"]),
    item("c:glass_blocks", &["minecraft:glass"]),
    item("c:chests", &["minecraft:chest"]),
    item("c:stones", &["minecraft:stone"]),
    item("c:player_workstations/crafting_tables", &["minecraft:crafting_table"]),
    item("c:player_workstations/furnaces", &["minecraft:furnace"]),
    item("c:cobblestones", &["minecraft:cobblestone", "minecraft:mossy_cobblestone"]),
    item("c:obsidians", &["minecraft:obsidian"]),
    item("c:sandstone_blocks", &["minecraft:sandstone"]),
];
