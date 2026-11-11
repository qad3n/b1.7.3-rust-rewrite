use crate::resources::{Location, MINECRAFT};

struct File
{
    m_beta: &'static str,
    m_modern: &'static str,
}

// modern names are used where the image layout matches the beta file
const FILES: &[File] = &[
    File { m_beta: "achievement/bg.png", m_modern: "textures/gui/achievement/background.png" },
    File { m_beta: "achievement/icons.png", m_modern: "textures/gui/achievement/icons.png" },
    File { m_beta: "achievement/map.txt", m_modern: "texts/achievement_map.txt" },
    File { m_beta: "armor/chain_1.png", m_modern: "textures/models/armor/chainmail_layer_1.png" },
    File { m_beta: "armor/chain_2.png", m_modern: "textures/models/armor/chainmail_layer_2.png" },
    File { m_beta: "armor/cloth_1.png", m_modern: "textures/models/armor/leather_layer_1.png" },
    File { m_beta: "armor/cloth_2.png", m_modern: "textures/models/armor/leather_layer_2.png" },
    File { m_beta: "armor/diamond_1.png", m_modern: "textures/models/armor/diamond_layer_1.png" },
    File { m_beta: "armor/diamond_2.png", m_modern: "textures/models/armor/diamond_layer_2.png" },
    File { m_beta: "armor/gold_1.png", m_modern: "textures/models/armor/gold_layer_1.png" },
    File { m_beta: "armor/gold_2.png", m_modern: "textures/models/armor/gold_layer_2.png" },
    File { m_beta: "armor/iron_1.png", m_modern: "textures/models/armor/iron_layer_1.png" },
    File { m_beta: "armor/iron_2.png", m_modern: "textures/models/armor/iron_layer_2.png" },
    File { m_beta: "armor/power.png", m_modern: "textures/entity/creeper/creeper_armor.png" },
    File { m_beta: "art/kz.png", m_modern: "textures/painting/paintings_kristoffer_zetterstrand.png" },
    File { m_beta: "environment/clouds.png", m_modern: "textures/environment/clouds.png" },
    File { m_beta: "environment/rain.png", m_modern: "textures/environment/rain.png" },
    File { m_beta: "environment/snow.png", m_modern: "textures/environment/snow.png" },
    File { m_beta: "font/default.png", m_modern: "textures/font/ascii.png" },
    File { m_beta: "font.txt", m_modern: "texts/font.txt" },
    File { m_beta: "gui/background.png", m_modern: "textures/gui/options_background.png" },
    File { m_beta: "gui/container.png", m_modern: "textures/gui/container/generic_54.png" },
    File { m_beta: "gui/crafting.png", m_modern: "textures/gui/container/crafting_table.png" },
    File { m_beta: "gui/furnace.png", m_modern: "textures/gui/container/furnace.png" },
    File { m_beta: "gui/gui.png", m_modern: "textures/gui/widgets.png" },
    File { m_beta: "gui/icons.png", m_modern: "textures/gui/icons.png" },
    File { m_beta: "gui/inventory.png", m_modern: "textures/gui/container/inventory.png" },
    File { m_beta: "gui/items.png", m_modern: "textures/atlas/items.png" },
    File { m_beta: "gui/logo.png", m_modern: "textures/gui/logo.png" },
    File { m_beta: "gui/particles.png", m_modern: "textures/gui/particles.png" },
    File { m_beta: "gui/slot.png", m_modern: "textures/gui/container/stats_icons.png" },
    File { m_beta: "gui/trap.png", m_modern: "textures/gui/container/dispenser.png" },
    File { m_beta: "gui/unknown_pack.png", m_modern: "textures/misc/unknown_pack.png" },
    File { m_beta: "item/arrows.png", m_modern: "textures/entity/projectiles/arrow.png" },
    File { m_beta: "item/boat.png", m_modern: "textures/entity/boat.png" },
    File { m_beta: "item/cart.png", m_modern: "textures/entity/minecart.png" },
    File { m_beta: "item/door.png", m_modern: "textures/entity/door.png" },
    File { m_beta: "item/sign.png", m_modern: "textures/entity/sign.png" },
    File { m_beta: "lang/en_US.lang", m_modern: "lang/en_US.lang" },
    File { m_beta: "lang/stats_US.lang", m_modern: "lang/stats_US.lang" },
    File { m_beta: "misc/dial.png", m_modern: "textures/misc/dial.png" },
    File { m_beta: "misc/foliagecolor.png", m_modern: "textures/colormap/foliage.png" },
    File { m_beta: "misc/footprint.png", m_modern: "textures/particle/footprint.png" },
    File { m_beta: "misc/grasscolor.png", m_modern: "textures/colormap/grass.png" },
    File { m_beta: "misc/mapbg.png", m_modern: "textures/map/map_background.png" },
    File { m_beta: "misc/mapicons.png", m_modern: "textures/map/map_icons.png" },
    File { m_beta: "misc/pumpkinblur.png", m_modern: "textures/misc/pumpkinblur.png" },
    File { m_beta: "misc/shadow.png", m_modern: "textures/misc/shadow.png" },
    File { m_beta: "misc/vignette.png", m_modern: "textures/misc/vignette.png" },
    File { m_beta: "misc/water.png", m_modern: "textures/misc/underwater.png" },
    File { m_beta: "misc/watercolor.png", m_modern: "textures/colormap/water.png" },
    File { m_beta: "mob/char.png", m_modern: "textures/entity/steve.png" },
    File { m_beta: "mob/chicken.png", m_modern: "textures/entity/chicken.png" },
    File { m_beta: "mob/cow.png", m_modern: "textures/entity/cow/cow.png" },
    File { m_beta: "mob/creeper.png", m_modern: "textures/entity/creeper/creeper.png" },
    File { m_beta: "mob/ghast.png", m_modern: "textures/entity/ghast/ghast.png" },
    File { m_beta: "mob/ghast_fire.png", m_modern: "textures/entity/ghast/ghast_shooting.png" },
    File { m_beta: "mob/pig.png", m_modern: "textures/entity/pig/pig.png" },
    File { m_beta: "mob/pigman.png", m_modern: "textures/entity/pigman.png" },
    File { m_beta: "mob/pigzombie.png", m_modern: "textures/entity/zombie_pigman.png" },
    File { m_beta: "mob/saddle.png", m_modern: "textures/entity/pig/pig_saddle.png" },
    File { m_beta: "mob/sheep.png", m_modern: "textures/entity/sheep/sheep.png" },
    File { m_beta: "mob/sheep_fur.png", m_modern: "textures/entity/sheep/sheep_fur.png" },
    File { m_beta: "mob/silverfish.png", m_modern: "textures/entity/silverfish.png" },
    File { m_beta: "mob/skeleton.png", m_modern: "textures/entity/skeleton/skeleton.png" },
    File { m_beta: "mob/slime.png", m_modern: "textures/entity/slime/slime.png" },
    File { m_beta: "mob/spider.png", m_modern: "textures/entity/spider/spider.png" },
    File { m_beta: "mob/spider_eyes.png", m_modern: "textures/entity/spider_eyes.png" },
    File { m_beta: "mob/squid.png", m_modern: "textures/entity/squid.png" },
    File { m_beta: "mob/wolf.png", m_modern: "textures/entity/wolf/wolf.png" },
    File { m_beta: "mob/wolf_angry.png", m_modern: "textures/entity/wolf/wolf_angry.png" },
    File { m_beta: "mob/wolf_tame.png", m_modern: "textures/entity/wolf/wolf_tame.png" },
    File { m_beta: "mob/zombie.png", m_modern: "textures/entity/zombie/zombie.png" },
    File { m_beta: "particles.png", m_modern: "textures/particle/particles.png" },
    File { m_beta: "terrain.png", m_modern: "textures/atlas/blocks.png" },
    File { m_beta: "terrain/moon.png", m_modern: "textures/environment/moon.png" },
    File { m_beta: "terrain/sun.png", m_modern: "textures/environment/sun.png" },
    File { m_beta: "title/black.png", m_modern: "textures/gui/title/black.png" },
    File { m_beta: "title/mclogo.png", m_modern: "textures/gui/title/minecraft.png" },
    File { m_beta: "title/mojang.png", m_modern: "textures/gui/title/mojang.png" },
    File { m_beta: "title/splashes.txt", m_modern: "texts/splashes.txt" },
];

// a file the table does not know keeps its beta path under the minecraft namespace
pub(crate) fn location(path: &str) -> Location
{
    let path: &str = path.trim_start_matches('/');
    let modern: &str = FILES.iter().find(|file| file.m_beta == path).map_or(path, |file| file.m_modern);
    Location::minecraft(modern)
}

// the beta file that a legacy pack holds for a location
pub(crate) fn path(location: &Location) -> Option<&str>
{
    if location.m_namespace != MINECRAFT
    {
        return None;
    }

    let path: &str = location.m_path.as_str();
    match FILES.iter().find(|file| file.m_modern == path)
    {
        Some(file) => Some(file.m_beta),
        None if FILES.iter().any(|file| file.m_beta == path) => None,
        None => Some(path),
    }
}
