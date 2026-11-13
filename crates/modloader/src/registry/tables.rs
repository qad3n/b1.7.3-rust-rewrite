use super::{Registry, SyncedRegistry};
use game::block::Block;
use game::crafting::{CraftingManager, FurnaceRecipes, IRecipe};
use game::entity::EntityList;
use game::item::{Item, ItemStack};
use game::tileentity::TileEntityKind;
use jvm::array_list::JArrayList;
use mod_protocol::Identifier;
use mod_protocol::abi::RegistryKind;
use mod_protocol::spec::{BlockSpec, EntitySpec, ItemSpec, RecipeSpec, TileEntitySpec};
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

pub struct BlockEntry
{
    pub m_spec: Option<BlockSpec>,
    // set for the hidden extra ids of a block with more than 16 states, the block and which page of states
    pub m_page: Option<(Identifier, usize)>,
}

pub struct ItemEntry
{
    pub m_spec: Option<ItemSpec>,
    // set for the item form of a block, which shares the block numeric id
    pub m_block: Option<Identifier>,
}

pub struct EntityEntry
{
    pub m_saveKey: String,
    pub m_spec: Option<EntitySpec>,
}

pub struct TileEntityEntry
{
    pub m_saveId: String,
    pub m_spec: Option<TileEntitySpec>,
}

pub struct RecipeEntry
{
    pub m_spec: Option<RecipeSpec>,
}

pub struct FuelEntry
{
    pub m_ticks: i32,
}

pub struct PacketEntry
{
    pub m_clientbound: bool,
    pub m_serverbound: bool,
}

pub struct DimensionEntry;

pub struct BiomeEntry;

// later recipes for the same output take a numbered suffix in recipe list order
fn unique(taken: &mut HashMap<String, i32>, name: String) -> String
{
    let count: &mut i32 = taken.entry(name.clone()).or_insert(0);
    *count += 1;
    match *count
    {
        1 => name,
        n => format!("{name}_{n}"),
    }
}

// registries whose numeric ids reach saves or the wire, blocks before items because block items share ids
pub const SYNCED_ORDER: [&str; 6] = ["block", "item", "entity", "tile_entity", "dimension", "biome"];

// each registry mirrors a game lookup table, the game arrays stay the hot path caches filled at freeze
pub struct Registries
{
    pub m_blocks: Registry<BlockEntry>,
    pub m_items: Registry<ItemEntry>,
    pub m_entities: Registry<EntityEntry>,
    pub m_tileEntities: Registry<TileEntityEntry>,
    pub m_recipes: Registry<RecipeEntry>,
    pub m_smelting: Registry<RecipeEntry>,
    pub m_fuel: Registry<FuelEntry>,
    pub m_packets: Registry<PacketEntry>,
    pub m_dimensions: Registry<DimensionEntry>,
    pub m_biomes: Registry<BiomeEntry>,
    pub m_named: BTreeMap<RegistryKind, Registry<()>>,
}

impl Registries
{
    pub fn seeded() -> Self
    {
        let mut registries: Self = Self
        {
            m_blocks: Registry::new("block"),
            m_items: Registry::new("item"),
            m_entities: Registry::new("entity"),
            m_tileEntities: Registry::new("tile_entity"),
            m_recipes: Registry::new("recipe"),
            m_smelting: Registry::new("smelting"),
            m_fuel: Registry::new("fuel"),
            m_packets: Registry::new("packet"),
            m_dimensions: Registry::new("dimension"),
            m_biomes: Registry::new("biome"),
            m_named: super::kinds::named(),
        };
        registries.seedBlocksAndItems();
        registries.seedEntities();
        registries.seedRecipes();
        registries.seedPackets();
        registries.seedNamed();

        for (numeric, name) in game::world::WorldProvider::NAMES
        {
            registries.m_dimensions.seed(Identifier::minecraft(name), numeric, DimensionEntry);
        }

        for (numeric, (name, _)) in game::worldgen::BiomeGenBase::NAMED.iter().enumerate()
        {
            registries.m_biomes.seed(Identifier::minecraft(name), numeric as i32, BiomeEntry);
        }

        registries
    }

    fn seedBlocksAndItems(&mut self)
    {
        // the game names its own blocks and items, a row whose object the game did not build is left out
        for (id, name) in Block::NAMES.iter().copied().filter(|(id, _)| Block::byId(*id).is_some())
        {
            self.m_blocks.seed(Identifier::minecraft(name), id, BlockEntry { m_spec: None, m_page: None });
        }

        for (numeric, name) in Item::NAMES.iter().copied().filter(|(numeric, _)| Item::byId(*numeric).is_some())
        {
            self.m_items.seed(Identifier::minecraft(name), numeric, ItemEntry { m_spec: None, m_block: None });
        }

        // a block item whose name a real item already uses, such as the bed block, takes a _block suffix
        for (id, name) in Block::NAMES.iter().copied()
        {
            if Item::byId(id).is_none()
            {
                continue;
            }

            let plain: Identifier = Identifier::minecraft(name);
            let itemId: Identifier = if self.m_items.get(&plain).is_some()
            {
                Identifier::minecraft(&format!("{name}_block"))
            }
            else
            {
                plain.clone()
            };
            self.m_items.seed(itemId, id, ItemEntry { m_spec: None, m_block: Some(plain) });
        }
    }

    fn seedEntities(&mut self)
    {
        for (key, numeric) in EntityList::vanillaRows()
        {
            let id: Identifier = Identifier::minecraft(&snakeCase(key));
            self.m_entities.seed(id, numeric, EntityEntry { m_saveKey: key.to_owned(), m_spec: None });
        }

        for (index, saveId) in TileEntityKind::vanillaIds().enumerate()
        {
            let id: Identifier = Identifier::minecraft(&snakeCase(saveId));
            self.m_tileEntities.seed(id, index as i32, TileEntityEntry { m_saveId: saveId.to_owned(), m_spec: None });
        }
    }

    fn seedRecipes(&mut self)
    {
        let list: Rc<JArrayList<Rc<dyn IRecipe>>> = CraftingManager::getInstance().getRecipeList();
        let mut taken: HashMap<String, i32> = HashMap::new();

        for index in 0..list.size()
        {
            let output: Rc<ItemStack> = list.get(index).getRecipeOutput();
            let name: String = format!("crafting/{}", self.outputName(output.m_itemID.get(), output.getItemDamage()));
            let id: Identifier = Identifier::minecraft(&unique(&mut taken, name));
            crate::content::recipes::nameRecipe(&list.get(index), id.clone());
            self.m_recipes.seed(id, index, RecipeEntry { m_spec: None });
        }

        let mut inputs: Vec<i32> = FurnaceRecipes::smelting().getSmeltingList().borrow().keys();
        inputs.sort_unstable();

        for input in inputs
        {
            let name: String = format!("smelting/{}", self.outputName(input, 0));
            self.m_smelting.seed(Identifier::minecraft(&name), input, RecipeEntry { m_spec: None });
        }

        for (name, ticks) in [("stick", 100), ("coal", 1600), ("lava_bucket", 20000), ("sapling", 100)]
        {
            let id: Identifier = Identifier::minecraft(name);
            if let Some(numeric) = self.m_items.numeric(&id)
            {
                self.m_fuel.seed(id, numeric, FuelEntry { m_ticks: ticks });
            }
        }
    }

    // the item path, with the damage value when a recipe makes a variant such as a dye colour
    fn outputName(&self, item: i32, damage: i32) -> String
    {
        let path: String =
            self.m_items.byNumeric(item).map_or_else(|| item.to_string(), |row| row.m_id.path().to_owned());

        match damage
        {
            0 => path,
            _ => format!("{path}_{damage}"),
        }
    }

    // game and loader rows of the named registries, client-only rows such as particles come from the client
    fn seedNamed(&mut self)
    {
        let rows: [(RegistryKind, Vec<String>); 7] = [
            (RegistryKind::Attribute, game::attribute::names()),
            (RegistryKind::GameRule, game::gamerule::GameRule::all().into_iter().map(|row| row.name()).collect()),
            (RegistryKind::Fluid, game::fluid::names()),
            (RegistryKind::DamageType, game::damage::names()),
            (RegistryKind::Machine, crate::content::machines::vanillaNames()),
            (RegistryKind::Structure, crate::structures::vanillaNames()),
            (RegistryKind::LootCondition, crate::content::loot::vanillaNames()),
        ];

        for (kind, names) in rows
        {
            for name in names
            {
                match Identifier::parse(&name)
                {
                    Ok(id) => self.seedName(kind, id),
                    Err(error) => crate::log::warn(crate::log::LOADER, &format!("vanilla row {name}: {error}")),
                }
            }
        }
    }

    fn seedPackets(&mut self)
    {
        for (numeric, clientbound, serverbound) in game::network::packet::packetRows()
        {
            let entry: PacketEntry = PacketEntry { m_clientbound: clientbound, m_serverbound: serverbound };
            self.m_packets.seed(Identifier::minecraft(&format!("packet/{numeric}")), numeric, entry);
        }
    }

    pub fn synced(&self) -> [&dyn SyncedRegistry; 6]
    {
        [&self.m_blocks, &self.m_items, &self.m_entities, &self.m_tileEntities, &self.m_dimensions, &self.m_biomes]
    }

    pub fn syncedMut(&mut self) -> [&mut dyn SyncedRegistry; 6]
    {
        [
            &mut self.m_blocks,
            &mut self.m_items,
            &mut self.m_entities,
            &mut self.m_tileEntities,
            &mut self.m_dimensions,
            &mut self.m_biomes,
        ]
    }

    pub fn blockId(&self, id: &Identifier) -> Option<i32> { self.m_blocks.numeric(id) }

    pub fn itemId(&self, id: &Identifier) -> Option<i32> { self.m_items.numeric(id) }
}

// java class keys such as PigZombie become pig_zombie
fn snakeCase(key: &str) -> String
{
    let mut out: String = String::with_capacity(key.len() + 4);

    for (index, character) in key.chars().enumerate()
    {
        if character.is_ascii_uppercase()
        {
            if index > 0
            {
                out.push('_');
            }

            out.push(character.to_ascii_lowercase());
        }
        else
        {
            out.push(character);
        }
    }

    out
}
