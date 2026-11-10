use super::{BlockData, ResolvedDrop, templates};
use crate::atlas::Atlas;
use crate::registry::{self, Row};
use game::block::{Block, BlockClass, Material, StepSound};
use game::math::AxisAlignedBB;
use game::world::{IBlockAccess, World};
use jvm::array_list::JArrayList;
use mod_protocol::Identifier;
use mod_protocol::spec::{BlockSpec, Cuboid, DropSpec, FaceTextures};
use std::rc::Rc;

pub const FACES: usize = 6;
// render type the client draws as the cuboids of a block model
pub const MODEL_RENDER_TYPE: i32 = 1000;
const MODEL_UNITS: f64 = 16.0;
const DEFAULT_SPRITE: i32 = 1;

fn data(this: &Block) -> Rc<BlockData>
{
    super::blockData(this.m_blockID).expect("a data driven block class only runs for registered mod blocks")
}

fn faceTexture(this: &Rc<Block>, face: i32, metadata: i32) -> i32
{
    let data: Rc<BlockData> = data(this);
    let side: usize = face.clamp(0, 5) as usize;
    if let Some(states) = &data.m_stateFaces
    {
        let state: usize = data.m_page * crate::states::pages::STATES_PER_ID + (metadata as usize & 15);
        return states.get(state).map_or(data.m_faces[side], |faces| faces[side]);
    }
    match data.m_bed
    {
        Some(bed) => bedTexture(&bed, face, metadata),
        None => data.m_faces[side],
    }
}

// BlockBed.getBlockTextureFromSideAndMetadata with the bed's own sprites instead of offsets on the terrain sheet
fn bedTexture(bed: &[i32; 7], face: i32, metadata: i32) -> i32
{
    let [topHead, topFoot, sideHead, sideFoot, endHead, endFoot, bottom] = *bed;
    if face == 0
    {
        return bottom;
    }
    let direction: usize = game::block::block_bed::getDirectionFromMetadata(metadata) as usize;
    let along: i32 = game::block::model_bed::BED_DIRECTION[direction][face.clamp(0, 5) as usize];
    let head: bool = game::block::block_bed::isBlockFootOfBed(metadata);
    match (head, along)
    {
        (true, 2) => endHead,
        (true, 4 | 5) => sideHead,
        (true, _) => topHead,
        (false, 3) => endFoot,
        (false, 4 | 5) => sideFoot,
        (false, _) => topFoot,
    }
}

// the side offsets of faces 2 to 5, as Facing.offsetsXForSide and offsetsZForSide
const SIDE_OFFSETS: [[i32; 2]; 6] = [[0, 0], [0, 0], [0, -1], [0, 1], [-1, 0], [1, 0]];

fn connects(spec: &BlockSpec, world: &dyn IBlockAccess, own: i32, x: i32, y: i32, z: i32) -> bool
{
    let neighbor: i32 = world.getBlockId(x, y, z);
    if neighbor == own || (neighbor != 0 && Block::opaqueCubeLookup(neighbor))
    {
        return true;
    }
    let listed = spec.model.as_ref().map(|model| model.connects_to.as_slice()).unwrap_or_default();
    listed.iter().any(|id| crate::registry::with(|registries| registries.blockId(id)) == Some(neighbor))
}

// the cuboids of a model for a metadata alone, as a block in flight shows with no neighbors to connect to
pub fn metaCuboids(blockId: i32, meta: i32) -> Vec<Cuboid>
{
    let Some(data) = super::blockData(blockId)
    else
    {
        return Vec::new();
    };
    let Some(model) = data.m_spec.model.as_ref()
    else
    {
        return Vec::new();
    };
    model
        .cuboids
        .iter()
        .filter(|cuboid| cuboid.connect.is_none() && (cuboid.metas.is_empty() || cuboid.metas.contains(&meta)))
        .cloned()
        .collect()
}

// the cuboids of a model that show at a spot, by its metadata and its neighbors
pub fn activeCuboids(blockId: i32, world: &dyn IBlockAccess, x: i32, y: i32, z: i32) -> Vec<Cuboid>
{
    let Some(data) = super::blockData(blockId)
    else
    {
        return Vec::new();
    };
    let Some(model) = data.m_spec.model.as_ref()
    else
    {
        return Vec::new();
    };
    let meta: i32 = world.getBlockMetadata(x, y, z);
    model
        .cuboids
        .iter()
        .filter(|cuboid| cuboid.metas.is_empty() || cuboid.metas.contains(&meta))
        .filter(|cuboid| {
            cuboid.connect.is_none_or(|face| {
                let [dx, dz] = SIDE_OFFSETS[face.clamp(0, 5) as usize];
                connects(&data.m_spec, world, blockId, x + dx, y, z + dz)
            })
        })
        .cloned()
        .collect()
}

fn color(this: &Rc<Block>, _metadata: i32) -> i32 { data(this).m_spec.color.map_or(0xFFFFFF, |color| color as i32) }

fn colorAt(this: &Rc<Block>, _world: &dyn IBlockAccess, _x: i32, _y: i32, _z: i32) -> i32 { color(this, 0) }

fn renderType(this: &Rc<Block>) -> i32
{
    data(this).m_spec.model.as_ref().and_then(|model| model.render_type).unwrap_or(0)
}

fn notOpaque(_this: &Rc<Block>) -> bool { false }

fn modelRenderType(_this: &Rc<Block>) -> i32 { MODEL_RENDER_TYPE }

fn cuboidBoxes(this: &Block, world: &World, x: i32, y: i32, z: i32) -> Vec<Rc<AxisAlignedBB>>
{
    boxesAt(&activeCuboids(this.m_blockID, world, x, y, z), x, y, z)
}

fn boxesAt(cuboids: &[Cuboid], x: i32, y: i32, z: i32) -> Vec<Rc<AxisAlignedBB>>
{
    cuboids
        .iter()
        .map(|cuboid| {
            AxisAlignedBB::getBoundingBoxFromPool(
                x as f64 + cuboid.from[0] as f64 / MODEL_UNITS,
                y as f64 + cuboid.from[1] as f64 / MODEL_UNITS,
                z as f64 + cuboid.from[2] as f64 / MODEL_UNITS,
                x as f64 + cuboid.to[0] as f64 / MODEL_UNITS,
                y as f64 + cuboid.to[1] as f64 / MODEL_UNITS,
                z as f64 + cuboid.to[2] as f64 / MODEL_UNITS,
            )
        })
        .collect()
}

// collision comes from the model cuboids as data, never a wasm call, entities test it every tick
fn modelCollisions(
    this: &Rc<Block>,
    world: &Rc<World>,
    x: i32,
    y: i32,
    z: i32,
    shape: &AxisAlignedBB,
    collisions: &JArrayList<Rc<AxisAlignedBB>>,
)
{
    for bounds in cuboidBoxes(this, world, x, y, z)
    {
        if shape.intersectsWith(&bounds)
        {
            collisions.add(bounds);
        }
    }
}

fn specCollisions(
    this: &Rc<Block>,
    _world: &Rc<World>,
    x: i32,
    y: i32,
    z: i32,
    shape: &AxisAlignedBB,
    collisions: &JArrayList<Rc<AxisAlignedBB>>,
)
{
    let data: Rc<BlockData> = data(this);
    for bounds in boxesAt(data.m_spec.collision.as_deref().unwrap_or_default(), x, y, z)
    {
        if shape.intersectsWith(&bounds)
        {
            collisions.add(bounds);
        }
    }
}

fn modelSelection(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32) -> Rc<AxisAlignedBB>
{
    let boxes: Vec<Rc<AxisAlignedBB>> = cuboidBoxes(this, world, x, y, z);
    let fold = |pick: fn(&AxisAlignedBB) -> f64, min: bool| -> f64 {
        boxes
            .iter()
            .map(|bounds| pick(bounds))
            .fold(if min { f64::MAX } else { f64::MIN }, |a, b| if min { a.min(b) } else { a.max(b) })
    };
    if boxes.is_empty()
    {
        return AxisAlignedBB::getBoundingBoxFromPool(
            x as f64,
            y as f64,
            z as f64,
            x as f64 + 1.0,
            y as f64 + 1.0,
            z as f64 + 1.0,
        );
    }
    AxisAlignedBB::getBoundingBoxFromPool(
        fold(AxisAlignedBB::minX, true),
        fold(AxisAlignedBB::minY, true),
        fold(AxisAlignedBB::minZ, true),
        fold(AxisAlignedBB::maxX, false),
        fold(AxisAlignedBB::maxY, false),
        fold(AxisAlignedBB::maxZ, false),
    )
}

fn translucentPass(this: &Rc<Block>) -> i32
{
    i32::from(data(this).m_spec.model.as_ref().is_some_and(|model| model.translucent))
}

// spec drops replace the single idDropped path, each drop rolls its own chance and count
fn dropItems(this: &Rc<Block>, world: &Rc<World>, x: i32, y: i32, z: i32, _metadata: i32, luck: f32)
{
    if world.m_multiplayerWorld.get()
    {
        return;
    }
    let data: Rc<BlockData> = data(this);
    for drop in &data.m_drops
    {
        if world.m_rand.nextFloat() > luck * drop.m_chance
        {
            continue;
        }
        let count: i32 = drop.m_min + world.m_rand.nextIntBounded(drop.m_max - drop.m_min + 1);
        if count > 0
        {
            this.dropBlockAsItem_do(world, x, y, z, game::item::ItemStack::withDamage(drop.m_item, count, drop.m_meta));
        }
    }
}

fn soundFor(name: &str) -> Option<Rc<StepSound>>
{
    StepSound::sounds(|sounds| match name
    {
        "stone" => Some(Rc::clone(&sounds.m_stone)),
        "wood" => Some(Rc::clone(&sounds.m_wood)),
        "gravel" => Some(Rc::clone(&sounds.m_gravel)),
        "grass" => Some(Rc::clone(&sounds.m_grass)),
        "metal" => Some(Rc::clone(&sounds.m_metal)),
        "glass" => Some(Rc::clone(&sounds.m_glass)),
        "cloth" => Some(Rc::clone(&sounds.m_cloth)),
        "sand" => Some(Rc::clone(&sounds.m_sand)),
        "powder" => Some(Rc::clone(&sounds.m_powder)),
        _ => None,
    })
}

pub fn templateName(template: Option<&Identifier>, fallback: &'static str) -> Result<String, String>
{
    match template
    {
        None => Ok(fallback.to_owned()),
        Some(id) if id.isVanilla() => Ok(id.path().to_owned()),
        Some(id) => Err(format!("template {id} is not a vanilla template, templates are minecraft:<name>")),
    }
}

pub fn validate(spec: &BlockSpec) -> Result<(), String>
{
    let name: String = templateName(spec.template.as_ref(), templates::DEFAULT_BLOCK_TEMPLATE)?;
    if templates::blockTemplate(&name).is_none()
    {
        let known: Vec<&str> = templates::BLOCK_TEMPLATES.iter().map(|row| row.m_name).collect();
        return Err(format!("unknown block template minecraft:{name}, known: {}", known.join(", ")));
    }
    if let Some(material) = &spec.material
        && templates::material(material).is_none()
    {
        return Err(format!("unknown material {material}"));
    }
    if let Some(sound) = &spec.sound
        && soundFor(sound).is_none()
    {
        return Err(format!("unknown sound {sound}"));
    }
    Ok(())
}

pub fn requestTextures(spec: &BlockSpec)
{
    let faces: Option<&FaceTextures> = spec.textures.as_ref();
    let bed = spec.bed.iter().flat_map(|bed| {
        [&bed.top_head, &bed.top_foot, &bed.side_head, &bed.side_foot, &bed.end_head, &bed.end_foot, &bed.bottom]
    });
    let all = spec
        .texture
        .iter()
        .chain(
            faces
                .into_iter()
                .flat_map(|faces| [&faces.top, &faces.bottom, &faces.side, &faces.front].into_iter().flatten()),
        )
        .chain(bed);
    for texture in all
    {
        crate::atlas::request(Atlas::Terrain, texture);
    }
}

fn slot(texture: Option<&String>, fallback: i32) -> i32
{
    texture.and_then(|texture| super::textureSlot(texture, Atlas::Terrain)).unwrap_or(fallback)
}

fn faces(spec: &BlockSpec, sprite: i32) -> [i32; FACES] { faceSlots(spec.textures.as_ref(), sprite) }

fn faceSlots(textures: Option<&FaceTextures>, sprite: i32) -> [i32; FACES]
{
    let Some(faces) = textures
    else
    {
        return [sprite; FACES];
    };
    let side: i32 = slot(faces.side.as_ref(), sprite);
    [
        slot(faces.bottom.as_ref(), sprite),
        slot(faces.top.as_ref(), sprite),
        side,
        slot(faces.front.as_ref(), side),
        side,
        side,
    ]
}

fn resolveDrops(drops: &[DropSpec]) -> Result<Vec<ResolvedDrop>, String>
{
    drops
        .iter()
        .map(|drop| {
            let item: i32 = registry::with(|registries| registries.itemId(&drop.item))
                .ok_or_else(|| format!("drop {} is not a registered item", drop.item))?;
            let (min, max): (i32, i32) = match drop.count.as_slice()
            {
                [count] => (*count, *count),
                [min, max] if min <= max => (*min, *max),
                _ => return Err(format!("drop {} count must be [n] or [min, max]", drop.item)),
            };
            Ok(ResolvedDrop { m_item: item, m_min: min, m_max: max, m_meta: drop.meta, m_chance: drop.chance })
        })
        .collect()
}

// a variant's textures apply to every state whose properties match all of its when entries
fn stateFaces(table: &crate::states::StateTable, base: [i32; FACES]) -> Option<Vec<[i32; FACES]>>
{
    let turned: bool = table.has(FACING) || table.has(AXIS);
    if table.variants().is_empty() && !turned
    {
        return None;
    }
    let faces: Vec<[i32; FACES]> = (0..table.count())
        .map(|index| {
            let values: std::collections::BTreeMap<String, String> = table.properties(index);
            let faces: [i32; FACES] = table
                .variants()
                .iter()
                .filter(|variant| variant.when.iter().all(|(key, value)| values.get(key) == Some(value)))
                .fold(base, |faces, variant| {
                    let sprite: i32 = slot(variant.texture.as_ref(), faces[0]);
                    match (&variant.texture, &variant.textures)
                    {
                        (_, Some(textures)) => faceSlots(Some(textures), sprite),
                        (Some(_), None) => [sprite; FACES],
                        (None, None) => faces,
                    }
                });
            turn(faces, &values)
        })
        .collect();
    Some(faces)
}

// a facing state moves the front sprite onto that face, an axis state lays the top and bottom sprites along it
const FACING: &str = "facing";
const AXIS: &str = "axis";
const SIDE_NAMES: [&str; FACES] = ["down", "up", "north", "south", "west", "east"];

fn turn(faces: [i32; FACES], values: &std::collections::BTreeMap<String, String>) -> [i32; FACES]
{
    let [bottom, top, side, front, ..] = faces;
    if let Some(face) = values.get(FACING).and_then(|facing| SIDE_NAMES.iter().position(|name| name == facing))
    {
        let mut turned: [i32; FACES] = [side; FACES];
        if face >= 2
        {
            turned[0] = bottom;
            turned[1] = top;
        }
        turned[face] = front;
        return turned;
    }
    match values.get(AXIS).map(String::as_str)
    {
        Some("x") => [side, side, side, side, bottom, top],
        Some("z") => [side, side, bottom, top, side, side],
        Some("none") => [side; FACES],
        _ => faces,
    }
}

// a hidden page id has no item of its own, what its template would drop as itself drops as the base block
fn pageIdDropped(this: &Rc<Block>, metadata: i32, random: &jvm::random::Random) -> i32
{
    let parent: &BlockClass = this.m_class.m_parent.expect("a derived block class keeps its template");
    let dropped: i32 = (parent.idDropped)(this, metadata, random);
    if dropped != this.m_blockID
    {
        return dropped;
    }
    crate::states::locate(&data(this).m_id, 0).map_or(dropped, |(base, _)| base)
}

fn patch(class: &mut BlockClass, spec: &BlockSpec, stated: bool, page: usize)
{
    if page > 0
    {
        class.idDropped = pageIdDropped;
    }
    if spec.textures.is_some() || spec.bed.is_some() || stated
    {
        class.getBlockTextureFromSideAndMetadata = faceTexture;
    }
    if spec.drops.is_some()
    {
        class.dropBlockAsItemWithChance = dropItems;
    }
    if spec.color.is_some()
    {
        class.getRenderColor = color;
        class.colorMultiplier = colorAt;
    }
    if let Some(model) = &spec.model
    {
        if model.render_type.is_some_and(|kind| kind != 0)
        {
            class.getRenderType = renderType;
            class.isOpaqueCube = notOpaque;
            class.renderAsNormalBlock = notOpaque;
        }
        if !model.cuboids.is_empty()
        {
            class.getRenderType = modelRenderType;
            class.isOpaqueCube = notOpaque;
            class.renderAsNormalBlock = notOpaque;
            class.getCollidingBoundingBoxes = modelCollisions;
            class.getSelectedBoundingBoxFromPool = modelSelection;
        }
        if model.translucent
        {
            class.getRenderBlockPass = translucentPass;
            class.isOpaqueCube = notOpaque;
        }
    }
    if spec.collision.is_some()
    {
        class.getCollidingBoundingBoxes = specCollisions;
    }
    if let Some(behavior) = &spec.behavior
    {
        crate::behavior::patchBlock(class, behavior);
    }
    if spec.tile_entity.is_some()
    {
        crate::behavior::patchTileBlock(class);
    }
}

// runs after every registry froze, so drop and tile entity references resolve to numeric ids
pub fn build(row: &Row<crate::registry::BlockEntry>, owner: usize) -> Result<(), String>
{
    let spec: BlockSpec = row.m_value.m_spec.clone().ok_or("a modded block row has a spec")?;
    let numeric: i32 = row.m_numeric;
    let templateId: String = templateName(spec.template.as_ref(), templates::DEFAULT_BLOCK_TEMPLATE)?;
    let template: &templates::BlockTemplate =
        templates::blockTemplate(&templateId).ok_or_else(|| format!("unknown template {templateId}"))?;
    let material: Rc<Material> = spec.material.as_deref().and_then(templates::material).unwrap_or_else(Material::rock);
    let sprite: i32 = slot(spec.texture.as_ref(), DEFAULT_SPRITE);
    let drops: Vec<ResolvedDrop> = resolveDrops(spec.drops.as_deref().unwrap_or_default())?;
    // a hidden page id is the same block as its base, so it saves, drops and renders under the base identifier
    let (id, page): (Identifier, usize) = row.m_value.m_page.clone().unwrap_or_else(|| (row.m_id.clone(), 0));
    let table: Option<crate::states::StateTable> = crate::states::table(&id);
    if table.is_some()
    {
        crate::states::bindPage(numeric, &id, page);
    }
    let stateFaces: Option<Vec<[i32; FACES]>> =
        table.as_ref().and_then(|table| stateFaces(table, faces(&spec, sprite)));
    let stated: bool = stateFaces.is_some();
    super::putBlock(
        numeric,
        BlockData {
            m_stateFaces: stateFaces,
            m_page: page,
            m_id: id.clone(),
            m_owner: owner,
            m_faces: faces(&spec, sprite),
            m_bed: spec.bed.as_ref().map(|bed| {
                [
                    &bed.top_head,
                    &bed.top_foot,
                    &bed.side_head,
                    &bed.side_foot,
                    &bed.end_head,
                    &bed.end_foot,
                    &bed.bottom,
                ]
                .map(|texture| slot(Some(texture), sprite))
            }),
            m_drops: drops,
            m_tool: spec.tool.clone(),
            m_spec: spec.clone(),
        },
    );
    let class: &'static BlockClass =
        crate::classes::deriveBlockClass(templates::templateClass(template), &row.m_id.to_string(), |class| {
            patch(class, &spec, stated, page)
        });
    let mut block: Rc<Block> = game::hooks::withBlockClass(class, || (template.m_create)(numeric, sprite, material));
    block.m_blockIndexInTexture.set(sprite);
    if let Some(hardness) = spec.hardness
    {
        block = block.setHardness(hardness);
    }
    if let Some(resistance) = spec.resistance
    {
        block = block.setResistance(resistance);
    }
    if let Some(light) = spec.light
    {
        block = block.setLightValue(light);
    }
    if let Some(slipperiness) = spec.slipperiness
    {
        block.m_slipperiness.set(slipperiness);
    }
    if let Some(opacity) = spec.opacity
    {
        block = block.setLightOpacity(opacity);
    }
    if let Some(sound) = spec.sound.as_deref().and_then(soundFor)
    {
        block = block.setStepSound(sound);
    }
    if spec.tick_randomly == Some(true)
        || spec
            .behavior
            .as_ref()
            .is_some_and(|behavior| behavior.methods.contains(&mod_protocol::spec::BlockMethod::UpdateTick))
    {
        block = block.setTickOnLoad(true);
    }
    let key: String = format!("{}.{}", id.namespace(), id.path().replace('/', "."));
    block = block.setBlockName(&key);
    if let Some(name) = &spec.name
    {
        super::lang::addDefault(&format!("tile.{key}.name"), name);
    }
    if spec.tile_entity.is_some()
    {
        Block::setIsBlockContainerEntry(numeric, true);
    }
    if let Some(flammability) = spec.flammability
    {
        game::block::block_fire::setFlammable(numeric, flammability.encouragement, flammability.flammability);
    }
    if let Some(ticks) = spec.fuel
    {
        super::putFuel(numeric, ticks);
    }
    if page == 0
    {
        super::items::buildBlockItem(&block, spec.item.as_ref(), owner, &row.m_id)?;
    }
    block.initializeBlock();
    Ok(())
}
