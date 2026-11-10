use crate::host::context::{entityInfo, withWorld};
use crate::state;
use common::nbt::{NBTBase, NBTTagCompound};
use game::entity::{Entity, EntityClass, entity_creature, entity_living};
use game::world::World;
use jvm::string::JString;
use mod_protocol::Identifier;
use mod_protocol::abi::{BlockPos, EntityCall, GuestCall, GuestReply};
use mod_protocol::spec::{EntityMethod, EntitySpec};
use std::cell::{OnceCell, RefCell};
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

pub const MOD_DATA: &str = "ModData";
// the ModData key holding the bytes mirrored to clients, saved with the entity like any other mod data
// a mod id and the synced bytes it owns on one entity
type OwnedBytes = (String, Vec<u8>);

// one blob per entity before abi 3, the first mod to read it after an upgrade takes it over
const LEGACY_SYNCED_KEY: &str = "loader:synced";
// the loader namespace is reserved, so no mod's own data key can take this form
const SYNCED_PREFIX: &str = "loader:synced/";
const PATH_RANGE: f32 = 16.0;

type Constructor = fn(Option<Rc<World>>) -> Rc<Entity>;
type ClassPatch = (EntityMethod, fn(&mut EntityClass));
type BiomeRow = (&'static str, fn() -> Rc<game::worldgen::BiomeGenBase>);

pub struct EntityTemplate
{
    pub m_name: &'static str,
    pub m_class: &'static EntityClass,
    pub m_create: Constructor,
}

// a thrown or shot entity the mod drives: the loader moves it by its motion and faces it along its flight,
// the mod's tick handles gravity, drag and impacts
fn projectileUpdate(this: &Rc<Entity>)
{
    this.onEntityUpdate();
    let [x, y, z] = [this.m_motionX.get(), this.m_motionY.get(), this.m_motionZ.get()];
    this.setPosition(this.m_posX.get() + x, this.m_posY.get() + y, this.m_posZ.get() + z);
    let horizontal: f64 = (x * x + z * z).sqrt();
    if horizontal > 0.0 || y != 0.0
    {
        this.m_rotationYaw.set((x.atan2(z) as f32).to_degrees());
        this.m_rotationPitch.set((y.atan2(horizontal) as f32).to_degrees());
    }
}

pub const PROJECTILE: EntityClass = EntityClass {
    m_name: "EntityModProjectile",
    m_parent: Some(&game::entity::entity_snowball::CLASS),
    onUpdate: projectileUpdate,
    ..game::entity::entity_snowball::CLASS
};

// a block in flight the mod steers, drawn like falling sand, moved by its motion with no gravity or landing
pub const MOVING_BLOCK: EntityClass = EntityClass {
    m_name: "EntityModBlock",
    m_parent: Some(&game::entity::entity_falling_sand::CLASS),
    onUpdate: projectileUpdate,
    ..game::entity::entity_falling_sand::CLASS
};

// falling sand's shape with its position at the block's center
fn createMovingBlock(world: Option<Rc<World>>) -> Rc<Entity>
{
    const SIZE: f32 = 0.98;
    let entity: Rc<Entity> = game::entity::entity_falling_sand::new(world);
    entity.m_preventEntitySpawning.set(true);
    entity.setSize(SIZE, SIZE);
    entity.m_yOffset.set(SIZE / 2.0);
    entity
}

// EntityMob that never looks for a target and only fights whoever hurts it, the pig zombie's anger without the zombie
pub const NEUTRAL_MONSTER: EntityClass = EntityClass {
    m_name: "EntityModNeutralMob",
    m_parent: Some(&game::entity::entity_mob::CLASS),
    findPlayerToAttack: |_| None,
    ..game::entity::entity_mob::CLASS
};

pub const ENTITY_TEMPLATES: [EntityTemplate; 18] = [
    EntityTemplate { m_name: "projectile", m_class: &PROJECTILE, m_create: game::entity::entity_snowball::new },
    // EntityArrow itself: flies, sticks in blocks, drawn by RenderArrow and picked up when a player shot it
    EntityTemplate {
        m_name: "arrow",
        m_class: &game::entity::entity_arrow::CLASS,
        m_create: game::entity::entity_arrow::new,
    },
    EntityTemplate { m_name: "moving_block", m_class: &MOVING_BLOCK, m_create: createMovingBlock },
    // EntityMob itself: hunts players in sight, melee, gone on peaceful, never burns
    EntityTemplate {
        m_name: "monster",
        m_class: &game::entity::entity_mob::CLASS,
        m_create: game::entity::entity_mob::new,
    },
    EntityTemplate { m_name: "neutral_monster", m_class: &NEUTRAL_MONSTER, m_create: game::entity::entity_mob::new },
    EntityTemplate {
        m_name: "pig",
        m_class: &game::entity::entity_pig::CLASS,
        m_create: game::entity::entity_pig::new,
    },
    EntityTemplate {
        m_name: "cow",
        m_class: &game::entity::entity_cow::CLASS,
        m_create: game::entity::entity_cow::new,
    },
    EntityTemplate {
        m_name: "chicken",
        m_class: &game::entity::entity_chicken::CLASS,
        m_create: game::entity::entity_chicken::new,
    },
    EntityTemplate {
        m_name: "squid",
        m_class: &game::entity::entity_squid::CLASS,
        m_create: game::entity::entity_squid::new,
    },
    EntityTemplate {
        m_name: "zombie",
        m_class: &game::entity::entity_zombie::CLASS,
        m_create: game::entity::entity_zombie::new,
    },
    EntityTemplate {
        m_name: "skeleton",
        m_class: &game::entity::entity_skeleton::CLASS,
        m_create: game::entity::entity_skeleton::new,
    },
    EntityTemplate {
        m_name: "spider",
        m_class: &game::entity::entity_spider::CLASS,
        m_create: game::entity::entity_spider::new,
    },
    EntityTemplate {
        m_name: "creeper",
        m_class: &game::entity::entity_creeper::CLASS,
        m_create: game::entity::entity_creeper::new,
    },
    EntityTemplate {
        m_name: "sheep",
        m_class: &game::entity::entity_sheep::CLASS,
        m_create: game::entity::entity_sheep::new,
    },
    EntityTemplate {
        m_name: "ghast",
        m_class: &game::entity::entity_ghast::CLASS,
        m_create: game::entity::entity_ghast::new,
    },
    EntityTemplate {
        m_name: "slime",
        m_class: &game::entity::entity_slime::CLASS,
        m_create: game::entity::entity_slime::new,
    },
    EntityTemplate {
        m_name: "wolf",
        m_class: &game::entity::entity_wolf::CLASS,
        m_create: game::entity::entity_wolf::new,
    },
    EntityTemplate {
        m_name: "zombie_pigman",
        m_class: &game::entity::entity_pig_zombie::CLASS,
        m_create: game::entity::entity_pig_zombie::new,
    },
];

pub struct EntityType
{
    pub m_id: Identifier,
    pub m_owner: usize,
    pub m_spec: EntitySpec,
    pub m_template: &'static EntityTemplate,
    m_class: OnceCell<&'static EntityClass>,
    m_texture: Option<JString>,
}

thread_local! {
    // synced bytes that reached a client before the entity they belong to
    static STASHED: RefCell<HashMap<i32, Vec<OwnedBytes>>> = RefCell::new(HashMap::new());
    static TYPES: RefCell<HashMap<Identifier, Rc<EntityType>>> = RefCell::new(HashMap::new());
    // per entity mod bytes by entity id, saved under ModData for every entity that has them
    static DATA: RefCell<HashMap<i32, BTreeMap<String, Vec<u8>>>> = RefCell::new(HashMap::new());
}

fn template(spec: &EntitySpec) -> Result<&'static EntityTemplate, String>
{
    let name: String = crate::content::blocks::templateName(spec.template.as_ref(), "pig")?;
    ENTITY_TEMPLATES.iter().find(|row| row.m_name == name).ok_or_else(|| {
        let known: Vec<&str> = ENTITY_TEMPLATES.iter().map(|row| row.m_name).collect();
        format!("unknown entity template minecraft:{name}, known: {}", known.join(", "))
    })
}

// mod textures load through the /mods/ path the client texture loader serves from mod assets
pub fn modTexturePath(texture: &str) -> Option<String>
{
    let id: Identifier = Identifier::parse(texture).ok()?;
    if id.isVanilla()
    {
        let extension: &str = if id.path().ends_with(".png")
        {
            ""
        }
        else
        {
            ".png"
        };
        Some(format!("/{}{extension}", id.path()))
    }
    else
    {
        Some(format!("/mods/{}/textures/{}.png", id.namespace(), id.path()))
    }
}

pub fn register(id: Identifier, owner: usize, spec: EntitySpec) -> Result<(), String>
{
    let template: &'static EntityTemplate = template(&spec)?;
    let texture: Option<JString> =
        spec.texture.as_deref().and_then(modTexturePath).map(|path| JString::from(path.as_str()));
    let entityType: EntityType = EntityType {
        m_id: id.clone(),
        m_owner: owner,
        m_spec: spec,
        m_template: template,
        m_class: OnceCell::new(),
        m_texture: texture,
    };
    let ownerId: String = state::modId(owner);
    crate::registry::with(|registries| {
        let entry = crate::registry::EntityEntry { m_saveKey: id.to_string(), m_spec: Some(entityType.m_spec.clone()) };
        registries.m_entities.register(id.clone(), &ownerId, entry).map_err(|error| error.to_string())
    })?;
    TYPES.with(|types| types.borrow_mut().insert(id, Rc::new(entityType)));
    Ok(())
}

pub fn types() -> Vec<Rc<EntityType>> { TYPES.with(|types| types.borrow().values().cloned().collect()) }

pub fn typeOf(entity: &Entity) -> Option<Rc<EntityType>>
{
    let id: Identifier = Identifier::parse(entity.className()).ok()?;
    TYPES.with(|types| types.borrow().get(&id).cloned())
}

fn parentClass(this: &Entity) -> &'static EntityClass
{
    this.m_class.m_parent.expect("a derived entity class keeps its template")
}

fn callEntity(this: &Rc<Entity>, call: EntityCall) -> Option<GuestReply>
{
    let entityType: Rc<EntityType> = typeOf(this)?;
    let handle: u32 = entityType.m_spec.behavior.as_ref()?.handle;
    let runtime: Rc<state::ModRuntime> = state::modAt(entityType.m_owner)?;
    let request: GuestCall = GuestCall::Entity { handle, call };
    match this.m_worldObj.get()
    {
        Some(world) => withWorld(&world, || runtime.call(&request)),
        None => runtime.call(&request),
    }
}

fn tick(this: &Rc<Entity>)
{
    (parentClass(this).onUpdate)(this);
    if !this.m_isDead.get()
    {
        callEntity(this, EntityCall::Tick { entity: entityInfo(this) });
    }
}

fn livingUpdate(this: &Rc<Entity>)
{
    (parentClass(this).onLivingUpdate)(this);
    callEntity(this, EntityCall::LivingUpdate { entity: entityInfo(this) });
}

fn interact(this: &Rc<Entity>, player: &Rc<Entity>) -> bool
{
    match callEntity(this, EntityCall::Interact { entity: entityInfo(this), player: player.m_entityId.get() })
    {
        Some(GuestReply::Bool(handled)) => handled,
        _ => (parentClass(this).interact)(this, player),
    }
}

// the guest answers whether the hit lands, the template then applies it
fn hurt(this: &Rc<Entity>, source: Option<&Rc<Entity>>, amount: i32) -> bool
{
    let call: EntityCall =
        EntityCall::Hurt { entity: entityInfo(this), source: source.map(|source| source.m_entityId.get()), amount };
    match callEntity(this, call)
    {
        Some(GuestReply::Bool(false)) => false,
        Some(GuestReply::Int(changed)) => (parentClass(this).attackEntityFrom)(this, source, changed),
        _ => (parentClass(this).attackEntityFrom)(this, source, amount),
    }
}

fn death(this: &Rc<Entity>, killer: Option<&Rc<Entity>>)
{
    callEntity(
        this,
        EntityCall::Death { entity: entityInfo(this), source: killer.map(|killer| killer.m_entityId.get()) },
    );
    (parentClass(this).onDeath)(this, killer);
}

fn canSpawn(this: &Rc<Entity>) -> bool
{
    match callEntity(this, EntityCall::CanSpawn { entity: entityInfo(this) })
    {
        Some(GuestReply::Bool(allowed)) => allowed,
        _ => (parentClass(this).getCanSpawnHere)(this),
    }
}

fn sound(this: &Entity, pick: fn(&mod_protocol::spec::EntitySounds) -> &Option<String>) -> Option<Option<String>>
{
    let entityType: Rc<EntityType> = typeOf(this)?;
    let chosen: &Option<String> = pick(entityType.m_spec.sounds.as_ref()?);
    chosen.as_ref().map(|name| Some(name.clone()).filter(|name| !name.is_empty()))
}

fn livingSound(this: &Rc<Entity>) -> Option<String>
{
    sound(this, |sounds| &sounds.living).unwrap_or_else(|| (parentClass(this).getLivingSound)(this))
}

fn hurtSound(this: &Rc<Entity>) -> Option<String>
{
    sound(this, |sounds| &sounds.hurt).unwrap_or_else(|| (parentClass(this).getHurtSound)(this))
}

fn deathSound(this: &Rc<Entity>) -> Option<String>
{
    sound(this, |sounds| &sounds.death).unwrap_or_else(|| (parentClass(this).getDeathSound)(this))
}

// an Ai behavior replaces the template's AI tick entirely
fn ai(this: &Rc<Entity>) { callEntity(this, EntityCall::Ai { entity: entityInfo(this) }); }

fn steeredMove(this: &Rc<Entity>, strafe: f32, forward: f32)
{
    let steering: Option<mod_protocol::spec::SteeringSpec> =
        typeOf(this).and_then(|entityType| entityType.m_spec.steering);
    if !steering.is_some_and(|steering| crate::steering::travel(this, &steering))
    {
        (parentClass(this).moveEntityWithHeading)(this, strafe, forward);
    }
}

fn texture(this: &Rc<Entity>) -> Option<JString>
{
    typeOf(this)
        .and_then(|entityType| entityType.m_texture.clone())
        .or_else(|| (parentClass(this).getEntityTexture)(this))
}

fn dropItems(this: &Rc<Entity>)
{
    let Some(entityType) = typeOf(this)
    else
    {
        return;
    };
    let Some(drops) = &entityType.m_spec.drops
    else
    {
        return (parentClass(this).dropFewItems)(this);
    };
    for drop in drops
    {
        let Some(item) = crate::registry::with(|registries| registries.itemId(&drop.item))
        else
        {
            continue;
        };
        if this.m_rand.nextFloat() > drop.chance
        {
            continue;
        }
        let (min, max): (i32, i32) = match drop.count.as_slice()
        {
            [count] => (*count, *count),
            [min, max] => (*min, *max),
            _ => (1, 1),
        };
        let count: i32 = min + this.m_rand.nextIntBounded((max - min + 1).max(1));
        if count > 0
        {
            this.entityDropItem(game::item::ItemStack::withDamage(item, count, drop.meta), 0.0);
        }
    }
}

fn wants(entityType: &EntityType, method: EntityMethod) -> bool
{
    entityType.m_spec.behavior.as_ref().is_some_and(|behavior| behavior.methods.contains(&method))
}

fn classFor(entityType: &Rc<EntityType>) -> &'static EntityClass
{
    entityType.m_class.get_or_init(|| {
        crate::classes::deriveEntityClass(entityType.m_template.m_class, &entityType.m_id.to_string(), |class| {
            class.getEntityTexture = texture;
            class.dropFewItems = dropItems;
            if entityType.m_spec.steering.is_some()
            {
                class.moveEntityWithHeading = steeredMove;
            }
            if entityType.m_spec.persistent
            {
                class.canDespawn = |_| false;
            }
            if entityType.m_spec.immovable
            {
                class.canBePushed = |_| false;
                class.addVelocity = |_, _, _, _| {};
            }
            if entityType.m_spec.sounds.is_some()
            {
                class.getLivingSound = livingSound;
                class.getHurtSound = hurtSound;
                class.getDeathSound = deathSound;
            }
            let methods: [ClassPatch; 7] = [
                (EntityMethod::Tick, |class| class.onUpdate = tick),
                (EntityMethod::LivingUpdate, |class| class.onLivingUpdate = livingUpdate),
                (EntityMethod::Interact, |class| class.interact = interact),
                (EntityMethod::Hurt, |class| class.attackEntityFrom = hurt),
                (EntityMethod::Death, |class| class.onDeath = death),
                (EntityMethod::CanSpawn, |class| class.getCanSpawnHere = canSpawn),
                (EntityMethod::Ai, |class| class.updatePlayerActionState = ai),
            ];
            for (method, apply) in methods
            {
                if wants(entityType, method)
                {
                    apply(class);
                }
            }
        })
    })
}

pub fn create(entityType: &Rc<EntityType>, world: Option<Rc<World>>) -> Rc<Entity>
{
    let class: &'static EntityClass = classFor(entityType);
    let entity: Rc<Entity> = game::hooks::withEntityClass(class, || (entityType.m_template.m_create)(world));
    if let Some([width, height]) = entityType.m_spec.size
    {
        entity.setSize(width, height);
    }
    if let Some(health) = entityType.m_spec.health
        && entity.instanceOf("EntityLiving")
    {
        entity_living::fields(&entity).m_health.set(health);
    }
    if let Some(speed) = entityType.m_spec.speed
        && entity.instanceOf("EntityLiving")
    {
        entity_living::fields(&entity).m_moveSpeed.set(speed);
    }
    if entityType.m_spec.fire_immune
    {
        entity.m_isImmuneToFire.set(true);
    }
    entity
}

pub fn createByKey(key: &str, world: &Rc<World>) -> Option<Rc<Entity>>
{
    let id: Identifier = crate::remap::rename(&Identifier::parse(key).ok()?);
    let entityType: Rc<EntityType> = TYPES.with(|types| types.borrow().get(&id).cloned())?;
    Some(create(&entityType, Some(Rc::clone(world))))
}

pub fn spawn(world: &Rc<World>, kind: &Identifier, pos: [f64; 3], data: Vec<u8>) -> Result<i32, String>
{
    let entity: Rc<Entity> = match TYPES.with(|types| types.borrow().get(kind).cloned())
    {
        Some(entityType) => create(&entityType, Some(Rc::clone(world))),
        None =>
        {
            let key: String = crate::registry::with(|registries| {
                registries.m_entities.get(kind).map(|row| row.m_value.m_saveKey.clone())
            })
            .ok_or_else(|| format!("{kind} is not a registered entity"))?;
            game::entity::EntityList::createEntityInWorld(Some(&JString::from(key.as_str())), world)
                .ok_or_else(|| format!("{kind} could not be created"))?
        }
    };
    entity.setLocationAndAngles(pos[0], pos[1], pos[2], world.m_rand.nextFloat() * 360.0, 0.0);
    if !data.is_empty()
    {
        setData(&entity, kind.namespace(), data);
    }
    world.entityJoinedWorld(&entity);
    Ok(entity.m_entityId.get())
}

pub fn data(entity: &Entity, key: &str) -> Vec<u8>
{
    DATA.with(|data| {
        data.borrow().get(&entity.m_entityId.get()).and_then(|map| map.get(key).cloned()).unwrap_or_default()
    })
}

pub fn setData(entity: &Entity, key: &str, bytes: Vec<u8>)
{
    DATA.with(|data| data.borrow_mut().entry(entity.m_entityId.get()).or_default().insert(key.to_owned(), bytes));
}

fn syncedKey(owner: &str) -> String { format!("{SYNCED_PREFIX}{owner}") }

// each mod keeps its own synced bytes on an entity, so two mods syncing data on the player never overwrite each other
pub fn synced(entity: &Entity, owner: &str) -> Vec<u8>
{
    let key: String = syncedKey(owner);
    let bytes: Vec<u8> = data(entity, &key);
    if !bytes.is_empty()
    {
        return bytes;
    }
    let legacy: Vec<u8> = data(entity, LEGACY_SYNCED_KEY);
    if !legacy.is_empty()
    {
        DATA.with(|data| {
            if let Some(map) = data.borrow_mut().get_mut(&entity.m_entityId.get())
            {
                map.remove(LEGACY_SYNCED_KEY);
            }
        });
        setData(entity, &key, legacy.clone());
    }
    legacy
}

pub fn setSynced(entity: &Entity, owner: &str, bytes: Vec<u8>) { setData(entity, &syncedKey(owner), bytes); }

// every mod's synced bytes on an entity, by owner
fn allSynced(entity: &Entity) -> Vec<OwnedBytes>
{
    DATA.with(|data| {
        data.borrow()
            .get(&entity.m_entityId.get())
            .map(|map| {
                map.iter()
                    .filter_map(|(key, bytes)| Some((key.strip_prefix(SYNCED_PREFIX)?.to_owned(), bytes.clone())))
                    .collect()
            })
            .unwrap_or_default()
    })
}

// the server keeps the bytes and mirrors them to the players near the entity
pub fn publishSynced(entity: &Rc<Entity>, owner: &str, bytes: Vec<u8>)
{
    setSynced(entity, owner, bytes.clone());
    if let Some(world) = entity.m_worldObj.get()
    {
        let message =
            crate::network::loader::EntitySynced { id: entity.m_entityId.get(), owner: owner.to_owned(), data: bytes };
        let pos: [f64; 3] = [entity.m_posX.get(), entity.m_posY.get(), entity.m_posZ.get()];
        crate::network::loader::broadcastNear(crate::network::loader::ENTITY_SYNCED, &world, pos, &message);
    }
}

pub fn stashSynced(id: i32, owner: String, bytes: Vec<u8>)
{
    STASHED.with(|stash| stash.borrow_mut().entry(id).or_default().push((owner, bytes)));
}

pub fn claimStashedSynced(entity: &Entity)
{
    for (owner, bytes) in STASHED.with(|stash| stash.borrow_mut().remove(&entity.m_entityId.get())).unwrap_or_default()
    {
        setSynced(entity, &owner, bytes);
    }
}

// living entities draw through RenderLiving, projectiles through the plain model renderer
pub fn isLivingClass(className: &str) -> bool
{
    TYPES.with(|types| {
        types
            .borrow()
            .values()
            .find(|entityType| entityType.m_id.to_string() == className)
            .is_some_and(|entityType| !matches!(entityType.m_template.m_name, "projectile" | "arrow"))
    })
}

pub fn model(entity: &Entity) -> Option<mod_protocol::spec::EntityModelSpec> { typeOf(entity)?.m_spec.model.clone() }

pub fn modelByClass(className: &str) -> Option<mod_protocol::spec::EntityModelSpec>
{
    TYPES
        .with(|types| types.borrow().values().find(|entityType| entityType.m_id.to_string() == className).cloned())
        .and_then(|entityType| entityType.m_spec.model.clone())
}

// the guest's draw program for this frame, None draws the spec at rest
pub fn pose(entity: &Rc<Entity>, input: mod_protocol::abi::PoseInput) -> Option<mod_protocol::abi::Pose>
{
    let entityType: Rc<EntityType> = typeOf(entity)?;
    if !wants(&entityType, EntityMethod::Pose)
    {
        return None;
    }
    match callEntity(entity, EntityCall::Pose { input })
    {
        Some(GuestReply::Pose(pose)) => Some(pose),
        _ => None,
    }
}

pub fn forget(entity: &Entity) { DATA.with(|data| data.borrow_mut().remove(&entity.m_entityId.get())); }

// each mod key becomes one byte array in the ModData compound of the entity
pub fn writeData(entity: &Entity, nbt: &NBTTagCompound)
{
    let Some(map) = DATA.with(|data| data.borrow().get(&entity.m_entityId.get()).cloned())
    else
    {
        return;
    };
    let compound: Rc<NBTBase> = NBTTagCompound::create();
    for (key, bytes) in map
    {
        compound.compound().setByteArray(&key, Rc::new(bytes.iter().map(|byte| *byte as i8).collect()));
    }
    nbt.setCompoundTag(MOD_DATA, &compound);
}

pub fn readData(entity: &Entity, nbt: &NBTTagCompound)
{
    if !nbt.hasKey(MOD_DATA)
    {
        return;
    }
    let compound: Rc<NBTBase> = nbt.getCompoundTag(MOD_DATA);
    for tag in compound.compound().getValues()
    {
        let bytes: Vec<u8> = tag
            .byteArrayTag()
            .m_byteArray
            .get()
            .map(|bytes| bytes.iter().map(|byte| *byte as u8).collect())
            .unwrap_or_default();
        setData(entity, &tag.getKey().to_string(), bytes);
    }
}

pub fn moveTowards(entity: &Rc<Entity>, pos: BlockPos, _speed: f32)
{
    let Some(world) = entity.m_worldObj.get()
    else
    {
        return;
    };
    if entity.instanceOf("EntityCreature")
    {
        let path = world.getEntityPathToXYZ(entity, pos.x, pos.y, pos.z, PATH_RANGE);
        entity_creature::fields(entity).m_pathToEntity.set(path);
    }
}

pub fn hasTypes() -> bool { TYPES.with(|types| !types.borrow().is_empty()) }

pub fn keyOf(entity: &Entity) -> Option<String> { typeOf(entity).map(|entityType| entityType.m_id.to_string()) }

pub fn numericOf(entity: &Entity) -> Option<i32>
{
    let entityType: Rc<EntityType> = typeOf(entity)?;
    crate::registry::with(|registries| registries.m_entities.numeric(&entityType.m_id))
}

pub(crate) fn biomesFor(spawn: &mod_protocol::spec::SpawnSpec) -> Vec<Rc<game::worldgen::BiomeGenBase>>
{
    use game::worldgen::BiomeGenBase;
    let all: [BiomeRow; 11] = [
        ("rainforest", BiomeGenBase::rainforest),
        ("swampland", BiomeGenBase::swampland),
        ("seasonal_forest", BiomeGenBase::seasonalForest),
        ("forest", BiomeGenBase::forest),
        ("savanna", BiomeGenBase::savanna),
        ("shrubland", BiomeGenBase::shrubland),
        ("taiga", BiomeGenBase::taiga),
        ("desert", BiomeGenBase::desert),
        ("plains", BiomeGenBase::plains),
        ("ice_desert", BiomeGenBase::iceDesert),
        ("tundra", BiomeGenBase::tundra),
    ];
    all.iter()
        .filter(|(name, _)| {
            spawn.biomes.is_empty() || spawn.biomes.iter().any(|id| id.isVanilla() && id.path() == *name)
        })
        .map(|(_, biome)| biome())
        .collect()
}

pub(crate) fn spawnList(
    biome: &game::worldgen::BiomeGenBase,
    kind: mod_protocol::spec::CreatureKind,
) -> &Rc<jvm::array_list::JArrayList<game::worldgen::SpawnListEntry>>
{
    match kind
    {
        mod_protocol::spec::CreatureKind::Monster => &biome.m_spawnableMonsterList,
        mod_protocol::spec::CreatureKind::Creature => &biome.m_spawnableCreatureList,
        mod_protocol::spec::CreatureKind::Water => &biome.m_spawnableWaterCreatureList,
    }
}

// natural spawning finds mod entities by their identifier, which the spawner hands to the loader
fn addSpawn(biome: &game::worldgen::BiomeGenBase, entityType: &EntityType, spawn: &mod_protocol::spec::SpawnSpec)
{
    let className: &'static str = Box::leak(entityType.m_id.to_string().into_boxed_str());
    spawnList(biome, spawn.kind).add(game::worldgen::SpawnListEntry::new(className, spawn.weight));
}

pub fn registerSpawns()
{
    for entityType in types()
    {
        if let Some(spawn) = &entityType.m_spec.spawn
        {
            for biome in biomesFor(spawn)
            {
                addSpawn(&biome, &entityType, spawn);
            }
        }
    }
}

// spawn entries naming a mod biome join it when its biome object is built
pub fn addSpawns(biome: &game::worldgen::BiomeGenBase, id: &Identifier)
{
    for entityType in types()
    {
        if let Some(spawn) = entityType.m_spec.spawn.as_ref().filter(|spawn| spawn.biomes.contains(id))
        {
            addSpawn(biome, &entityType, spawn);
        }
    }
    crate::modify::applySpawnsTo(biome, id);
}

pub fn createByNumeric(numeric: i32, world: &Rc<World>) -> Option<Rc<Entity>>
{
    let id: Identifier =
        crate::registry::with(|registries| registries.m_entities.byNumeric(numeric).map(|row| row.m_id.clone()))?;
    let entityType: Rc<EntityType> = TYPES.with(|types| types.borrow().get(&id).cloned())?;
    Some(create(&entityType, Some(Rc::clone(world))))
}

pub fn kindNamespace(entity: &Entity) -> String
{
    typeOf(entity).map(|entityType| entityType.m_id.namespace().to_owned()).unwrap_or_default()
}

mod tracking
{
    pub const RANGE: i32 = 80;
    pub const FREQUENCY: i32 = 3;
}

// the tracker reads range, update frequency and velocity sending from the entity spec instead of its class chain
pub fn tracking(entity: &Entity) -> Option<(i32, i32, bool)>
{
    let spec: EntitySpec = typeOf(entity)?.m_spec.clone();
    Some((
        spec.tracking_range.unwrap_or(tracking::RANGE),
        spec.update_frequency.unwrap_or(tracking::FREQUENCY),
        spec.send_velocity.unwrap_or(true),
    ))
}

// a mod entity waits for the player's mod handshake, a spawn sent before it would be dropped and never resent
pub fn canTrack(entity: &Entity, player: &Entity) -> bool
{
    let channel: Identifier = crate::network::loader::id(crate::network::loader::SPAWN_ENTITY);
    typeOf(entity).is_none()
        || crate::network::isLoopbackClient()
        || crate::netsync::peerHasChannel(player.m_entityId.get(), &channel)
}

// true when the loader handled the spawn, a vanilla client simply never sees mod entities
pub fn sendSpawn(entity: &Rc<Entity>, player: &Rc<Entity>) -> bool
{
    let Some(numeric) = numericOf(entity).filter(|_| typeOf(entity).is_some())
    else
    {
        return false;
    };
    let message = crate::network::loader::SpawnEntity {
        kind: numeric,
        id: entity.m_entityId.get(),
        pos: [entity.m_posX.get(), entity.m_posY.get(), entity.m_posZ.get()],
        rotation: [entity.m_rotationYaw.get(), entity.m_rotationPitch.get()],
        motion: [entity.m_motionX.get(), entity.m_motionY.get(), entity.m_motionZ.get()],
        data: data(entity, &kindNamespace(entity)),
    };
    let channel: Identifier = crate::network::loader::id(crate::network::loader::SPAWN_ENTITY);
    crate::network::sendToPlayer(&channel, player, mod_protocol::wire::encode(&message));
    sendMovingBlock(entity, player);
    sendSynced(entity, player);
    true
}

// a moving block's block and metadata live in falling sand's fields, which the spawn message does not carry
fn sendMovingBlock(entity: &Rc<Entity>, player: &Rc<Entity>)
{
    if !entity.instanceOf("EntityFallingSand")
    {
        return;
    }

    let fields = game::entity::entity_falling_sand::fields(entity);
    let message = crate::network::loader::MovingBlock {
        id: entity.m_entityId.get(),
        block: fields.m_blockID.get(),
        meta: fields.m_metadata.get(),
    };
    let channel: Identifier = crate::network::loader::id(crate::network::loader::MOVING_BLOCK);
    crate::network::sendToPlayer(&channel, player, mod_protocol::wire::encode(&message));
}

// the synced bytes of any entity, players included, for a player who just started seeing it
pub fn sendSynced(entity: &Rc<Entity>, player: &Rc<Entity>)
{
    let channel: Identifier = crate::network::loader::id(crate::network::loader::ENTITY_SYNCED);
    for (owner, data) in allSynced(entity).into_iter().filter(|(_, bytes)| !bytes.is_empty())
    {
        let update = crate::network::loader::EntitySynced { id: entity.m_entityId.get(), owner, data };
        crate::network::sendToPlayer(&channel, player, mod_protocol::wire::encode(&update));
    }
}
