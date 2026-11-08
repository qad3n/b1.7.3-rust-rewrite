use crate::block::{Block, block_rail};
use crate::entity::entity_boat::{LerpFields, clearDeadRider, mountBy, pushNearby, scale};
use crate::entity::{ENTITY, Entity, EntityClass, EntityExt, entity_item, entity_player};
use crate::inventory::IInventory;
use crate::item::{Item, ItemStack};
use crate::math::Vec3D;
use crate::math::math_helper::{floor_double, sqrt_double};
use crate::world::World;
use common::nbt::{NBTBase, NBTTagCompound, NBTTagList};
use jvm::throwable::OrNpe;
use std::cell::{Cell, RefCell};
use std::f64::consts::PI;
use std::rc::Rc;

const ADJACENT_RAIL_POSITIONS_BY_SHAPE: [[[i32; 3]; 2]; 10] = [
    [[0, 0, -1], [0, 0, 1]],
    [[-1, 0, 0], [1, 0, 0]],
    [[-1, -1, 0], [1, 0, 0]],
    [[-1, 0, 0], [1, -1, 0]],
    [[0, 0, -1], [0, -1, 1]],
    [[0, -1, -1], [0, 0, 1]],
    [[0, 0, 1], [1, 0, 0]],
    [[0, 0, 1], [-1, 0, 0]],
    [[0, 0, -1], [-1, 0, 0]],
    [[0, 0, -1], [1, 0, 0]],
];

pub struct MinecartFields
{
    m_cargoItems: RefCell<Vec<Option<Rc<ItemStack>>>>,
    pub m_minecartCurrentDamage: Cell<i32>,
    pub m_minecartTimeSinceHit: Cell<i32>,
    pub m_minecartRockDirection: Cell<i32>,
    m_flipped: Cell<bool>,
    pub m_minecartType: Cell<i32>,
    pub m_fuel: Cell<i32>,
    pub m_pushX: Cell<f64>,
    pub m_pushZ: Cell<f64>,
    m_lerp: LerpFields,
}

impl EntityExt for MinecartFields
{
    fn asAny(&self) -> &dyn std::any::Any { self }
}

pub fn fields(this: &Entity) -> &MinecartFields { this.ext::<MinecartFields>() }

pub struct MinecartInventory
{
    m_cart: Rc<Entity>,
}

pub fn inventory(this: &Rc<Entity>) -> Rc<dyn IInventory> { Rc::new(MinecartInventory { m_cart: Rc::clone(this) }) }

fn railShape(metadata: i32) -> &'static [[i32; 3]; 2]
{
    match ADJACENT_RAIL_POSITIONS_BY_SHAPE.get(metadata as usize)
    {
        Some(shape) if metadata >= 0 => shape,
        _ => jvm::throwable::throw(&jvm::throwable::ARRAY_INDEX, &metadata.to_string()),
    }
}

fn railMetadata(world: &World, l: i32, i: i32, j: i32, k: i32) -> i32
{
    let mut i1: i32 = world.getBlockMetadata(i, j, k);
    if block_rail::getIsPowered(&Block::byId(l).orNpe())
    {
        i1 &= 7;
    }
    i1
}

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(
        &CLASS,
        Box::new(MinecartFields {
            m_cargoItems: RefCell::new(Vec::new()),
            m_minecartCurrentDamage: Cell::new(0),
            m_minecartTimeSinceHit: Cell::new(0),
            m_minecartRockDirection: Cell::new(0),
            m_flipped: Cell::new(false),
            m_minecartType: Cell::new(0),
            m_fuel: Cell::new(0),
            m_pushX: Cell::new(0.0),
            m_pushZ: Cell::new(0.0),
            m_lerp: LerpFields::new(),
        }),
    );
    Entity::init(&entity, world);
    let cart: &MinecartFields = fields(&entity);
    *cart.m_cargoItems.borrow_mut() = vec![None; 36];
    cart.m_minecartRockDirection.set(1);
    entity.m_preventEntitySpawning.set(true);
    entity.setSize(0.98, 0.7);
    entity.m_yOffset.set(entity.m_height.get() / 2.0);
    entity
}

pub fn createAt(world: &Rc<World>, x: f64, y: f64, z: f64, kind: i32) -> Rc<Entity>
{
    let entity: Rc<Entity> = new(Some(Rc::clone(world)));
    entity.setPosition(x, y + entity.m_yOffset.get() as f64, z);
    entity.m_motionX.set(0.0);
    entity.m_motionY.set(0.0);
    entity.m_motionZ.set(0.0);
    entity.m_prevPosX.set(x);
    entity.m_prevPosY.set(y);
    entity.m_prevPosZ.set(z);
    fields(&entity).m_minecartType.set(kind);
    entity
}

fn slot(this: &Entity, slot: i32) -> Option<Rc<ItemStack>>
{
    let cargo: std::cell::Ref<Vec<Option<Rc<ItemStack>>>> = fields(this).m_cargoItems.borrow();
    match cargo.get(slot as usize)
    {
        Some(stack) if slot >= 0 => stack.clone(),
        _ => jvm::throwable::throw(&jvm::throwable::ARRAY_INDEX, &slot.to_string()),
    }
}

fn setSlot(this: &Entity, slot: i32, item: Option<Rc<ItemStack>>)
{
    let mut cargo: std::cell::RefMut<Vec<Option<Rc<ItemStack>>>> = fields(this).m_cargoItems.borrow_mut();
    match cargo.get_mut(slot as usize)
    {
        Some(stack) if slot >= 0 => *stack = item,
        _ => jvm::throwable::throw(&jvm::throwable::ARRAY_INDEX, &slot.to_string()),
    }
}

fn dropCargo(this: &Rc<Entity>)
{
    let world: Rc<World> = this.world();
    for i in 0..getSizeInventory()
    {
        let Some(itemstack) = slot(this, i)
        else
        {
            continue;
        };
        let f: f32 = this.m_rand.nextFloat() * 0.8 + 0.1;
        let f1: f32 = this.m_rand.nextFloat() * 0.8 + 0.1;
        let f2: f32 = this.m_rand.nextFloat() * 0.8 + 0.1;
        while itemstack.m_stackSize.get() > 0
        {
            let mut j: i32 = this.m_rand.nextIntBounded(21) + 10;
            if j > itemstack.m_stackSize.get()
            {
                j = itemstack.m_stackSize.get();
            }
            itemstack.m_stackSize.set(itemstack.m_stackSize.get() - j);
            let entityitem: Rc<Entity> = entity_item::create(
                &world,
                this.posX() + f as f64,
                this.posY() + f1 as f64,
                this.posZ() + f2 as f64,
                ItemStack::withDamage(itemstack.m_itemID.get(), j, itemstack.getItemDamage()),
            );
            let f3: f32 = 0.05;
            entityitem.m_motionX.set((this.m_rand.nextGaussian() as f32 * f3) as f64);
            entityitem.m_motionY.set((this.m_rand.nextGaussian() as f32 * f3 + 0.2) as f64);
            entityitem.m_motionZ.set((this.m_rand.nextGaussian() as f32 * f3) as f64);
            world.entityJoinedWorld(&entityitem);
        }
    }
}

fn isRider(this: &Entity, entity: &Rc<Entity>) -> bool
{
    this.m_riddenByEntity.get().is_some_and(|rider| Rc::ptr_eq(&rider, entity))
}

fn clampToRail(value: f64, limit: f64) -> f64
{
    let mut value: f64 = value;
    if value < -limit
    {
        value = -limit;
    }
    if value > limit
    {
        value = limit;
    }
    value
}

fn horizontalSpeed(this: &Entity) -> f64
{
    jvm::math::sqrt(this.m_motionX.get() * this.m_motionX.get() + this.m_motionZ.get() * this.m_motionZ.get())
}

#[allow(clippy::too_many_arguments)]
fn moveOnRail(
    this: &Rc<Entity>,
    world: &Rc<World>,
    cart: &MinecartFields,
    i: i32,
    j: i32,
    k: i32,
    l: i32,
    d0: f64,
) -> bool
{
    let mut flag: bool = false;
    let d2: f64 = 0.0078125;
    let vec3d: Option<Rc<Vec3D>> = snapPositionToRail(this, this.posX(), this.posY(), this.posZ());
    let mut i1: i32 = world.getBlockMetadata(i, j, k);
    this.m_posY.set(j as f64);
    let mut flag1: bool = false;
    let mut flag2: bool = false;
    if l == Block::railPowered().m_blockID
    {
        flag1 = (i1 & 8) != 0;
        flag2 = !flag1;
    }
    if block_rail::getIsPowered(&Block::byId(l).orNpe())
    {
        i1 &= 7;
    }
    if (2..=5).contains(&i1)
    {
        this.m_posY.set((j + 1) as f64);
    }
    match i1
    {
        2 => this.m_motionX.set(this.m_motionX.get() - d2),
        3 => this.m_motionX.set(this.m_motionX.get() + d2),
        4 => this.m_motionZ.set(this.m_motionZ.get() + d2),
        5 => this.m_motionZ.set(this.m_motionZ.get() - d2),
        _ =>
        {}
    }
    let aint: &[[i32; 3]; 2] = railShape(i1);
    let mut d3: f64 = (aint[1][0] - aint[0][0]) as f64;
    let mut d4: f64 = (aint[1][2] - aint[0][2]) as f64;
    let d5: f64 = jvm::math::sqrt(d3 * d3 + d4 * d4);
    let d6: f64 = this.m_motionX.get() * d3 + this.m_motionZ.get() * d4;
    if d6 < 0.0
    {
        d3 = -d3;
        d4 = -d4;
    }
    let mut d7: f64 = horizontalSpeed(this);
    this.m_motionX.set(d7 * d3 / d5);
    this.m_motionZ.set(d7 * d4 / d5);
    if flag2
    {
        let d8: f64 = horizontalSpeed(this);
        if d8 < 0.03
        {
            scale(this, 0.0, 0.0, 0.0);
        }
        else
        {
            scale(this, 0.5, 0.0, 0.5);
        }
    }

    let d26: f64;
    let d9: f64 = i as f64 + 0.5 + aint[0][0] as f64 * 0.5;
    let d10: f64 = k as f64 + 0.5 + aint[0][2] as f64 * 0.5;
    let d11: f64 = i as f64 + 0.5 + aint[1][0] as f64 * 0.5;
    let d12: f64 = k as f64 + 0.5 + aint[1][2] as f64 * 0.5;
    d3 = d11 - d9;
    d4 = d12 - d10;
    if d3 == 0.0
    {
        this.m_posX.set(i as f64 + 0.5);
        d26 = this.posZ() - k as f64;
    }
    else if d4 == 0.0
    {
        this.m_posZ.set(k as f64 + 0.5);
        d26 = this.posX() - i as f64;
    }
    else
    {
        let d13: f64 = this.posX() - d9;
        let d14: f64 = this.posZ() - d10;
        d26 = (d13 * d3 + d14 * d4) * 2.0;
    }
    this.m_posX.set(d9 + d3 * d26);
    this.m_posZ.set(d10 + d4 * d26);
    this.setPosition(this.posX(), this.posY() + this.m_yOffset.get() as f64, this.posZ());
    let mut d27: f64 = this.m_motionX.get();
    let mut d28: f64 = this.m_motionZ.get();
    if this.m_riddenByEntity.with(Option::is_some)
    {
        d27 *= 0.75;
        d28 *= 0.75;
    }
    d27 = clampToRail(d27, d0);
    d28 = clampToRail(d28, d0);
    this.moveEntity(d27, 0.0, d28);
    if aint[0][1] != 0 && floor_double(this.posX()) - i == aint[0][0] && floor_double(this.posZ()) - k == aint[0][2]
    {
        this.setPosition(this.posX(), this.posY() + aint[0][1] as f64, this.posZ());
    }
    else if aint[1][1] != 0
        && floor_double(this.posX()) - i == aint[1][0]
        && floor_double(this.posZ()) - k == aint[1][2]
    {
        this.setPosition(this.posX(), this.posY() + aint[1][1] as f64, this.posZ());
    }

    if this.m_riddenByEntity.with(Option::is_some)
    {
        scale(this, 0.997f32 as f64, 0.0, 0.997f32 as f64);
    }
    else
    {
        if cart.m_minecartType.get() == 2
        {
            let d29: f64 =
                sqrt_double(cart.m_pushX.get() * cart.m_pushX.get() + cart.m_pushZ.get() * cart.m_pushZ.get()) as f64;
            if d29 > 0.01
            {
                flag = true;
                cart.m_pushX.set(cart.m_pushX.get() / d29);
                cart.m_pushZ.set(cart.m_pushZ.get() / d29);
                let d17: f64 = 0.04;
                scale(this, 0.8f32 as f64, 0.0, 0.8f32 as f64);
                this.m_motionX.set(this.m_motionX.get() + cart.m_pushX.get() * d17);
                this.m_motionZ.set(this.m_motionZ.get() + cart.m_pushZ.get() * d17);
            }
            else
            {
                scale(this, 0.9f32 as f64, 0.0, 0.9f32 as f64);
            }
        }
        scale(this, 0.96f32 as f64, 0.0, 0.96f32 as f64);
    }

    let vec3d1: Option<Rc<Vec3D>> = snapPositionToRail(this, this.posX(), this.posY(), this.posZ());
    if let (Some(vec3d1), Some(vec3d)) = (&vec3d1, &vec3d)
    {
        let d16: f64 = (vec3d.y() - vec3d1.y()) * 0.05;
        d7 = horizontalSpeed(this);
        if d7 > 0.0
        {
            this.m_motionX.set(this.m_motionX.get() / d7 * (d7 + d16));
            this.m_motionZ.set(this.m_motionZ.get() / d7 * (d7 + d16));
        }
        this.setPosition(this.posX(), vec3d1.y(), this.posZ());
    }

    let k1: i32 = floor_double(this.posX());
    let l1: i32 = floor_double(this.posZ());
    if k1 != i || l1 != k
    {
        d7 = horizontalSpeed(this);
        this.m_motionX.set(d7 * (k1 - i) as f64);
        this.m_motionZ.set(d7 * (l1 - k) as f64);
    }

    if cart.m_minecartType.get() == 2
    {
        let d18: f64 =
            sqrt_double(cart.m_pushX.get() * cart.m_pushX.get() + cart.m_pushZ.get() * cart.m_pushZ.get()) as f64;
        if d18 > 0.01
            && this.m_motionX.get() * this.m_motionX.get() + this.m_motionZ.get() * this.m_motionZ.get() > 0.001
        {
            cart.m_pushX.set(cart.m_pushX.get() / d18);
            cart.m_pushZ.set(cart.m_pushZ.get() / d18);
            if cart.m_pushX.get() * this.m_motionX.get() + cart.m_pushZ.get() * this.m_motionZ.get() < 0.0
            {
                cart.m_pushX.set(0.0);
                cart.m_pushZ.set(0.0);
            }
            else
            {
                cart.m_pushX.set(this.m_motionX.get());
                cart.m_pushZ.set(this.m_motionZ.get());
            }
        }
    }

    if flag1
    {
        let d30: f64 = horizontalSpeed(this);
        if d30 > 0.01
        {
            let d19: f64 = 0.06;
            this.m_motionX.set(this.m_motionX.get() + this.m_motionX.get() / d30 * d19);
            this.m_motionZ.set(this.m_motionZ.get() + this.m_motionZ.get() / d30 * d19);
        }
        else if i1 == 1
        {
            if world.isBlockNormalCube(i - 1, j, k)
            {
                this.m_motionX.set(0.02);
            }
            else if world.isBlockNormalCube(i + 1, j, k)
            {
                this.m_motionX.set(-0.02);
            }
        }
        else if i1 == 0
        {
            if world.isBlockNormalCube(i, j, k - 1)
            {
                this.m_motionZ.set(0.02);
            }
            else if world.isBlockNormalCube(i, j, k + 1)
            {
                this.m_motionZ.set(-0.02);
            }
        }
    }
    flag
}

fn onUpdate(this: &Rc<Entity>)
{
    let cart: &MinecartFields = fields(this);
    if cart.m_minecartTimeSinceHit.get() > 0
    {
        cart.m_minecartTimeSinceHit.set(cart.m_minecartTimeSinceHit.get() - 1);
    }
    if cart.m_minecartCurrentDamage.get() > 0
    {
        cart.m_minecartCurrentDamage.set(cart.m_minecartCurrentDamage.get() - 1);
    }
    let world: Rc<World> = this.world();
    if world.m_multiplayerWorld.get() && cart.m_lerp.steps() > 0
    {
        cart.m_lerp.step(this);
        return;
    }

    this.m_prevPosX.set(this.posX());
    this.m_prevPosY.set(this.posY());
    this.m_prevPosZ.set(this.posZ());
    this.m_motionY.set(this.m_motionY.get() - 0.04f32 as f64);
    let i: i32 = floor_double(this.posX());
    let mut j: i32 = floor_double(this.posY());
    let k: i32 = floor_double(this.posZ());
    if block_rail::isRailBlockAt(&world, i, j - 1, k)
    {
        j -= 1;
    }
    let d0: f64 = 0.4;
    let mut flag: bool = false;
    let l: i32 = world.getBlockId(i, j, k);
    if block_rail::isRailBlock(l)
    {
        flag = moveOnRail(this, &world, cart, i, j, k, l, d0);
    }
    else
    {
        this.m_motionX.set(clampToRail(this.m_motionX.get(), d0));
        this.m_motionZ.set(clampToRail(this.m_motionZ.get(), d0));
        if this.m_onGround.get()
        {
            scale(this, 0.5, 0.5, 0.5);
        }
        this.moveEntity(this.m_motionX.get(), this.m_motionY.get(), this.m_motionZ.get());
        if !this.m_onGround.get()
        {
            scale(this, 0.95f32 as f64, 0.95f32 as f64, 0.95f32 as f64);
        }
    }

    this.m_rotationPitch.set(0.0);
    let d23: f64 = this.m_prevPosX.get() - this.posX();
    let d24: f64 = this.m_prevPosZ.get() - this.posZ();
    if d23 * d23 + d24 * d24 > 0.001
    {
        this.m_rotationYaw.set((jvm::math::atan2(d24, d23) * 180.0 / PI) as f32);
        if cart.m_flipped.get()
        {
            this.m_rotationYaw.set(this.m_rotationYaw.get() + 180.0);
        }
    }
    let mut d25: f64 = (this.m_rotationYaw.get() - this.m_prevRotationYaw.get()) as f64;
    while d25 >= 180.0
    {
        d25 -= 360.0;
    }
    while d25 < -180.0
    {
        d25 += 360.0;
    }
    if d25 < -170.0 || d25 >= 170.0
    {
        this.m_rotationYaw.set(this.m_rotationYaw.get() + 180.0);
        cart.m_flipped.set(!cart.m_flipped.get());
    }
    this.setRotation(this.m_rotationYaw.get(), this.m_rotationPitch.get());
    pushNearby(this, &world, "EntityMinecart");
    clearDeadRider(this);
    if flag && this.m_rand.nextIntBounded(4) == 0
    {
        cart.m_fuel.set(cart.m_fuel.get() - 1);
        if cart.m_fuel.get() < 0
        {
            cart.m_pushX.set(0.0);
            cart.m_pushZ.set(0.0);
        }
        world.spawnParticle("largesmoke", this.posX(), this.posY() + 0.8, this.posZ(), 0.0, 0.0, 0.0);
    }
}

pub fn snapPositionToRailWithOffset(this: &Entity, mut x: f64, mut y: f64, mut z: f64, offset: f64)
-> Option<Rc<Vec3D>>
{
    let world: Rc<World> = this.world();
    let floorX: i32 = floor_double(x);
    let mut floorY: i32 = floor_double(y);
    let floorZ: i32 = floor_double(z);
    if block_rail::isRailBlockAt(&world, floorX, floorY - 1, floorZ)
    {
        floorY -= 1;
    }
    let blockId: i32 = world.getBlockId(floorX, floorY, floorZ);
    if !block_rail::isRailBlock(blockId)
    {
        return None;
    }
    let blockMetadata: i32 = railMetadata(&world, blockId, floorX, floorY, floorZ);
    y = floorY as f64;
    if (2..=5).contains(&blockMetadata)
    {
        y = (floorY + 1) as f64;
    }
    let array: &[[i32; 3]; 2] = railShape(blockMetadata);
    let n: f64 = (array[1][0] - array[0][0]) as f64;
    let n2: f64 = (array[1][2] - array[0][2]) as f64;
    let sqrt: f64 = jvm::math::sqrt(n * n + n2 * n2);
    let n3: f64 = n / sqrt;
    let n4: f64 = n2 / sqrt;
    x += n3 * offset;
    z += n4 * offset;
    if array[0][1] != 0 && floor_double(x) - floorX == array[0][0] && floor_double(z) - floorZ == array[0][2]
    {
        y += array[0][1] as f64;
    }
    else if array[1][1] != 0 && floor_double(x) - floorX == array[1][0] && floor_double(z) - floorZ == array[1][2]
    {
        y += array[1][1] as f64;
    }
    snapPositionToRail(this, x, y, z)
}

pub fn snapPositionToRail(this: &Entity, mut x: f64, y: f64, mut z: f64) -> Option<Rc<Vec3D>>
{
    let world: Rc<World> = this.world();
    let i: i32 = floor_double(x);
    let mut j: i32 = floor_double(y);
    let k: i32 = floor_double(z);
    if block_rail::isRailBlockAt(&world, i, j - 1, k)
    {
        j -= 1;
    }
    let l: i32 = world.getBlockId(i, j, k);
    if !block_rail::isRailBlock(l)
    {
        return None;
    }
    let i1: i32 = railMetadata(&world, l, i, j, k);
    let aint: &[[i32; 3]; 2] = railShape(i1);
    let d0: f64;
    let d1: f64 = i as f64 + 0.5 + aint[0][0] as f64 * 0.5;
    let d2: f64 = j as f64 + 0.5 + aint[0][1] as f64 * 0.5;
    let d3: f64 = k as f64 + 0.5 + aint[0][2] as f64 * 0.5;
    let d4: f64 = i as f64 + 0.5 + aint[1][0] as f64 * 0.5;
    let d5: f64 = j as f64 + 0.5 + aint[1][1] as f64 * 0.5;
    let d6: f64 = k as f64 + 0.5 + aint[1][2] as f64 * 0.5;
    let d7: f64 = d4 - d1;
    let d8: f64 = (d5 - d2) * 2.0;
    let d9: f64 = d6 - d3;
    if d7 == 0.0
    {
        x = i as f64 + 0.5;
        d0 = z - k as f64;
    }
    else if d9 == 0.0
    {
        z = k as f64 + 0.5;
        d0 = x - i as f64;
    }
    else
    {
        let d10: f64 = x - d1;
        let d11: f64 = z - d3;
        d0 = (d10 * d7 + d11 * d9) * 2.0;
    }
    x = d1 + d7 * d0;
    let mut y: f64 = d2 + d8 * d0;
    z = d3 + d9 * d0;
    if d8 < 0.0
    {
        y += 1.0;
    }
    if d8 > 0.0
    {
        y += 0.5;
    }
    Some(Vec3D::createVector(x, y, z))
}

fn applyEntityCollision(this: &Rc<Entity>, entity: &Rc<Entity>)
{
    if this.world().m_multiplayerWorld.get() || isRider(this, entity)
    {
        return;
    }
    let cart: &MinecartFields = fields(this);
    if entity.instanceOf("EntityLiving")
        && !entity.instanceOf("EntityPlayer")
        && cart.m_minecartType.get() == 0
        && this.m_motionX.get() * this.m_motionX.get() + this.m_motionZ.get() * this.m_motionZ.get() > 0.01
        && this.m_riddenByEntity.with(Option::is_none)
        && entity.m_ridingEntity.with(Option::is_none)
    {
        entity.mountEntity(Some(this));
    }
    let mut d0: f64 = entity.posX() - this.posX();
    let mut d1: f64 = entity.posZ() - this.posZ();
    let mut d2: f64 = d0 * d0 + d1 * d1;
    if !(d2 >= 1.0E-4f32 as f64)
    {
        return;
    }
    d2 = sqrt_double(d2) as f64;
    d0 /= d2;
    d1 /= d2;
    let mut d3: f64 = 1.0 / d2;
    if d3 > 1.0
    {
        d3 = 1.0;
    }
    d0 *= d3;
    d1 *= d3;
    d0 *= 0.1f32 as f64;
    d1 *= 0.1f32 as f64;
    d0 *= (1.0 - this.m_entityCollisionReduction.get()) as f64;
    d1 *= (1.0 - this.m_entityCollisionReduction.get()) as f64;
    d0 *= 0.5;
    d1 *= 0.5;
    if !entity.instanceOf("EntityMinecart")
    {
        this.addVelocity(-d0, 0.0, -d1);
        entity.addVelocity(d0 / 4.0, 0.0, d1 / 4.0);
        return;
    }
    let d4: f64 = entity.posX() - this.posX();
    let d5: f64 = entity.posZ() - this.posZ();
    let mut d6: f64 = d4 * entity.m_motionZ.get() + d5 * entity.m_prevPosX.get();
    d6 *= d6;
    if d6 > 5.0
    {
        return;
    }
    let mut d7: f64 = entity.m_motionX.get() + this.m_motionX.get();
    let mut d8: f64 = entity.m_motionZ.get() + this.m_motionZ.get();
    let otherType: i32 = fields(entity).m_minecartType.get();
    if otherType == 2 && cart.m_minecartType.get() != 2
    {
        scale(this, 0.2f32 as f64, 1.0, 0.2f32 as f64);
        this.addVelocity(entity.m_motionX.get() - d0, 0.0, entity.m_motionZ.get() - d1);
        scale(entity, 0.7f32 as f64, 1.0, 0.7f32 as f64);
    }
    else if otherType != 2 && cart.m_minecartType.get() == 2
    {
        scale(entity, 0.2f32 as f64, 1.0, 0.2f32 as f64);
        entity.addVelocity(this.m_motionX.get() + d0, 0.0, this.m_motionZ.get() + d1);
        scale(this, 0.7f32 as f64, 1.0, 0.7f32 as f64);
    }
    else
    {
        d7 /= 2.0;
        d8 /= 2.0;
        scale(this, 0.2f32 as f64, 1.0, 0.2f32 as f64);
        this.addVelocity(d7 - d0, 0.0, d8 - d1);
        scale(entity, 0.2f32 as f64, 1.0, 0.2f32 as f64);
        entity.addVelocity(d7 + d0, 0.0, d8 + d1);
    }
}

fn getSizeInventory() -> i32 { 27 }

fn interact(this: &Rc<Entity>, player: &Rc<Entity>) -> bool
{
    let cart: &MinecartFields = fields(this);
    match cart.m_minecartType.get()
    {
        0 => return mountBy(this, player),
        1 =>
        {
            if !this.world().m_multiplayerWorld.get()
            {
                player.displayGUIChest(Some(&inventory(this)));
            }
        }
        2 =>
        {
            let playerInventory: Rc<crate::inventory::InventoryPlayer> = entity_player::inventory(player);
            if let Some(itemstack) = playerInventory.getCurrentItem()
                && itemstack.m_itemID.get() == Item::coal().m_shiftedIndex
            {
                itemstack.m_stackSize.set(itemstack.m_stackSize.get() - 1);
                if itemstack.m_stackSize.get() == 0
                {
                    playerInventory.setInventorySlotContents(playerInventory.m_currentItem.get(), None);
                }
                cart.m_fuel.set(cart.m_fuel.get() + 1200);
            }
            cart.m_pushX.set(this.posX() - player.posX());
            cart.m_pushZ.set(this.posZ() - player.posZ());
        }
        _ =>
        {}
    }
    true
}

impl IInventory for MinecartInventory
{
    fn getSizeInventory(&self) -> i32 { getSizeInventory() }

    fn getStackInSlot(&self, slotIndex: i32) -> Option<Rc<ItemStack>> { slot(&self.m_cart, slotIndex) }

    fn decrStackSize(&self, slotIndex: i32, amount: i32) -> Option<Rc<ItemStack>>
    {
        let stack: Rc<ItemStack> = slot(&self.m_cart, slotIndex)?;
        if stack.m_stackSize.get() <= amount
        {
            setSlot(&self.m_cart, slotIndex, None);
            return Some(stack);
        }
        let itemstack: Rc<ItemStack> = stack.splitStack(amount);
        if stack.m_stackSize.get() == 0
        {
            setSlot(&self.m_cart, slotIndex, None);
        }
        Some(itemstack)
    }

    fn setInventorySlotContents(&self, slotIndex: i32, item: Option<Rc<ItemStack>>)
    {
        setSlot(&self.m_cart, slotIndex, item.clone());
        if let Some(item) = item
            && item.m_stackSize.get() > self.getInventoryStackLimit()
        {
            item.m_stackSize.set(self.getInventoryStackLimit());
        }
    }

    fn getInvName(&self) -> String { "Minecart".to_owned() }

    fn getInventoryStackLimit(&self) -> i32 { 64 }

    fn onInventoryChanged(&self) {}

    fn canInteractWith(&self, player: &Rc<Entity>) -> bool
    {
        if self.m_cart.m_isDead.get()
        {
            false
        }
        else
        {
            !(player.getDistanceSqToEntity(&self.m_cart) > 64.0)
        }
    }
}

fn writeEntityToNBT(this: &Rc<Entity>, nbt: &NBTTagCompound)
{
    let cart: &MinecartFields = fields(this);
    nbt.setInteger("Type", cart.m_minecartType.get());
    match cart.m_minecartType.get()
    {
        2 =>
        {
            nbt.setDouble("PushX", cart.m_pushX.get());
            nbt.setDouble("PushZ", cart.m_pushZ.get());
            nbt.setShort("Fuel", cart.m_fuel.get() as i16);
        }
        1 =>
        {
            let nbttaglist: Rc<NBTBase> = NBTTagList::create();
            let length: i32 = cart.m_cargoItems.borrow().len() as i32;
            for i in 0..length
            {
                if let Some(stack) = slot(this, i)
                {
                    let nbttagcompound: Rc<NBTBase> = NBTTagCompound::create();
                    nbttagcompound.compound().setByte("Slot", i as i8);
                    stack.writeToNBT(nbttagcompound.compound());
                    nbttaglist.list().setTag(nbttagcompound);
                }
            }
            nbt.setTag("Items", &nbttaglist);
        }
        _ =>
        {}
    }
}

fn readEntityFromNBT(this: &Rc<Entity>, nbt: &NBTTagCompound)
{
    let cart: &MinecartFields = fields(this);
    cart.m_minecartType.set(nbt.getInteger("Type"));
    match cart.m_minecartType.get()
    {
        2 =>
        {
            cart.m_pushX.set(nbt.getDouble("PushX"));
            cart.m_pushZ.set(nbt.getDouble("PushZ"));
            cart.m_fuel.set(nbt.getShort("Fuel") as i32);
        }
        1 =>
        {
            let nbttaglist: Rc<NBTBase> = nbt.getTagList("Items");
            *cart.m_cargoItems.borrow_mut() = vec![None; getSizeInventory() as usize];
            for i in 0..nbttaglist.list().tagCount()
            {
                let tag: Rc<NBTBase> = nbttaglist.list().tagAt(i);
                let nbttagcompound: &NBTTagCompound = tag.compound();
                let j: i32 = nbttagcompound.getByte("Slot") as i32 & 255;
                if j >= 0 && j < cart.m_cargoItems.borrow().len() as i32
                {
                    setSlot(this, j, Some(ItemStack::withNbt(nbttagcompound)));
                }
            }
        }
        _ =>
        {}
    }
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityMinecart",
    m_parent: Some(&ENTITY),
    m_interfaces: &["IInventory"],
    canTriggerWalking: |_| false,
    entityInit: |_| {},
    getCollisionBox: |_, other| Some(Rc::clone(&other.m_boundingBox)),
    getBoundingBox: |_| None,
    canBePushed: |_| true,
    getMountedYOffset: |this| this.m_height.get() as f64 * 0.0 - 0.3f32 as f64,
    attackEntityFrom: |this, _source, amount| {
        if this.world().m_multiplayerWorld.get() || this.m_isDead.get()
        {
            return true;
        }
        let cart: &MinecartFields = fields(this);
        cart.m_minecartRockDirection.set(-cart.m_minecartRockDirection.get());
        cart.m_minecartTimeSinceHit.set(10);
        this.setBeenAttacked();
        cart.m_minecartCurrentDamage.set(cart.m_minecartCurrentDamage.get() + amount * 10);
        if cart.m_minecartCurrentDamage.get() > 40
        {
            if let Some(rider) = this.m_riddenByEntity.get()
            {
                rider.mountEntity(Some(this));
            }
            this.setEntityDead();
            this.dropItemWithOffset(Item::minecartEmpty().m_shiftedIndex, 1, 0.0);
            match cart.m_minecartType.get()
            {
                1 =>
                {
                    dropCargo(this);
                    this.dropItemWithOffset(Block::chest().m_blockID, 1, 0.0);
                }
                2 =>
                {
                    this.dropItemWithOffset(Block::stoneOvenIdle().m_blockID, 1, 0.0);
                }
                _ =>
                {}
            }
        }
        true
    },
    performHurtAnimation: |this| {
        jvm::system::println("Animating hurt");
        let cart: &MinecartFields = fields(this);
        cart.m_minecartRockDirection.set(-cart.m_minecartRockDirection.get());
        cart.m_minecartTimeSinceHit.set(10);
        cart.m_minecartCurrentDamage.set(cart.m_minecartCurrentDamage.get() + cart.m_minecartCurrentDamage.get() * 10);
    },
    canBeCollidedWith: |this| !this.m_isDead.get(),
    setEntityDead: |this| {
        dropCargo(this);
        (ENTITY.setEntityDead)(this);
    },
    onUpdate,
    writeEntityToNBT,
    readEntityFromNBT,
    getShadowSize: |_| 0.0,
    applyEntityCollision,
    interact,
    setPositionAndRotation2: |this, x, y, z, yaw, pitch, steps| {
        fields(this).m_lerp.setTarget(this, x, y, z, yaw, pitch, steps + 2)
    },
    setVelocity: |this, x, y, z| fields(this).m_lerp.setVelocity(this, x, y, z),
    ..ENTITY
};
