use crate::entity::{ENTITY, Entity, EntityClass, EntityExt, entity_item};
use crate::item::{Item, ItemStack};
use crate::math::math_helper::floor_double;
use crate::world::World;
use jvm::array_list::JArrayList;
use jvm::string::JString;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

pub struct EnumArt
{
    pub m_title: &'static str,
    pub m_sizeX: i32,
    pub m_sizeY: i32,
    pub m_offsetX: i32,
    pub m_offsetY: i32,
}

const fn artRow(title: &'static str, width: i32, height: i32, u: i32, v: i32) -> EnumArt
{
    EnumArt { m_title: title, m_sizeX: width, m_sizeY: height, m_offsetX: u, m_offsetY: v }
}

impl EnumArt
{
    pub const VALUES: [EnumArt; 25] = [
        artRow("Kebab", 16, 16, 0, 0),
        artRow("Aztec", 16, 16, 16, 0),
        artRow("Alban", 16, 16, 32, 0),
        artRow("Aztec2", 16, 16, 48, 0),
        artRow("Bomb", 16, 16, 64, 0),
        artRow("Plant", 16, 16, 80, 0),
        artRow("Wasteland", 16, 16, 96, 0),
        artRow("Pool", 32, 16, 0, 32),
        artRow("Courbet", 32, 16, 32, 32),
        artRow("Sea", 32, 16, 64, 32),
        artRow("Sunset", 32, 16, 96, 32),
        artRow("Creebet", 32, 16, 128, 32),
        artRow("Wanderer", 16, 32, 0, 64),
        artRow("Graham", 16, 32, 16, 64),
        artRow("Match", 32, 32, 0, 128),
        artRow("Bust", 32, 32, 32, 128),
        artRow("Stage", 32, 32, 64, 128),
        artRow("Void", 32, 32, 96, 128),
        artRow("SkullAndRoses", 32, 32, 128, 128),
        artRow("Fighters", 64, 32, 0, 96),
        artRow("Pointer", 64, 64, 0, 192),
        artRow("Pigscene", 64, 64, 64, 192),
        artRow("BurningSkull", 64, 64, 128, 192),
        artRow("Skeleton", 64, 48, 192, 64),
        artRow("DonkeyKong", 64, 48, 192, 112),
    ];

    pub fn maxArtTitleLength() -> i32 { "SkullAndRoses".len() as i32 }
}

pub struct PaintingFields
{
    m_ticksSinceValidation: Cell<i32>,
    pub m_direction: Cell<i32>,
    pub m_xPosition: Cell<i32>,
    pub m_yPosition: Cell<i32>,
    pub m_zPosition: Cell<i32>,
    pub m_art: Cell<Option<&'static EnumArt>>,
}

impl EntityExt for PaintingFields
{
    fn asAny(&self) -> &dyn std::any::Any { self }
}

pub fn fields(this: &Entity) -> &PaintingFields { this.ext::<PaintingFields>() }

fn art(this: &Entity) -> &'static EnumArt { fields(this).m_art.get().orNpe() }

pub fn new(world: Option<Rc<World>>) -> Rc<Entity>
{
    let entity: Rc<Entity> = Entity::alloc(
        &CLASS,
        Box::new(PaintingFields {
            m_ticksSinceValidation: Cell::new(0),
            m_direction: Cell::new(0),
            m_xPosition: Cell::new(0),
            m_yPosition: Cell::new(0),
            m_zPosition: Cell::new(0),
            m_art: Cell::new(None),
        }),
    );
    Entity::init(&entity, world);
    let painting: &PaintingFields = fields(&entity);
    painting.m_ticksSinceValidation.set(0);
    painting.m_direction.set(0);
    entity.m_yOffset.set(0.0);
    entity.setSize(0.5, 0.5);
    entity
}

fn newAt(world: &Rc<World>, x: i32, y: i32, z: i32) -> Rc<Entity>
{
    let entity: Rc<Entity> = new(Some(Rc::clone(world)));
    let painting: &PaintingFields = fields(&entity);
    painting.m_xPosition.set(x);
    painting.m_yPosition.set(y);
    painting.m_zPosition.set(z);
    entity
}

pub fn createAt(world: &Rc<World>, x: i32, y: i32, z: i32, direction: i32) -> Rc<Entity>
{
    let entity: Rc<Entity> = newAt(world, x, y, z);
    let painting: &PaintingFields = fields(&entity);
    let list: JArrayList<&'static EnumArt> = JArrayList::new();
    for art in &EnumArt::VALUES
    {
        painting.m_art.set(Some(art));
        setDirection(&entity, direction);
        if canSurvive(&entity)
        {
            list.add(art);
        }
    }
    if list.size() > 0
    {
        painting.m_art.set(Some(list.get(entity.m_rand.nextIntBounded(list.size()))));
    }
    setDirection(&entity, direction);
    entity
}

pub fn createWithMotive(world: &Rc<World>, x: i32, y: i32, z: i32, facing: i32, motive: &JString) -> Rc<Entity>
{
    let entity: Rc<Entity> = newAt(world, x, y, z);
    if let Some(art) = EnumArt::VALUES.iter().find(|art| JString::from(art.m_title) == *motive)
    {
        fields(&entity).m_art.set(Some(art));
    }
    setDirection(&entity, facing);
    entity
}

fn getPositionOffset(length: i32) -> f32
{
    match length
    {
        32 | 64 => 0.5,
        _ => 0.0,
    }
}

pub fn setDirection(this: &Entity, dir: i32)
{
    let painting: &PaintingFields = fields(this);
    painting.m_direction.set(dir);
    let n: f32 = (dir * 90) as f32;
    this.m_rotationYaw.set(n);
    this.m_prevRotationYaw.set(n);
    let art: &EnumArt = art(this);
    let mut n2: f32 = art.m_sizeX as f32;
    let n3: f32 = art.m_sizeY as f32;
    let mut n4: f32 = art.m_sizeX as f32;
    if dir == 0 || dir == 2
    {
        n4 = 0.5;
    }
    else
    {
        n2 = 0.5;
    }
    let n5: f32 = n2 / 32.0;
    let n6: f32 = n3 / 32.0;
    let n7: f32 = n4 / 32.0;
    let mut n8: f32 = painting.m_xPosition.get() as f32 + 0.5;
    let n9: f32 = painting.m_yPosition.get() as f32 + 0.5;
    let mut n10: f32 = painting.m_zPosition.get() as f32 + 0.5;
    let n11: f32 = 0.5625;
    match dir
    {
        0 => n10 -= n11,
        1 => n8 -= n11,
        2 => n10 += n11,
        3 => n8 += n11,
        _ =>
        {}
    }
    match dir
    {
        0 => n8 -= getPositionOffset(art.m_sizeX),
        1 => n10 += getPositionOffset(art.m_sizeX),
        2 => n8 += getPositionOffset(art.m_sizeX),
        3 => n10 -= getPositionOffset(art.m_sizeX),
        _ =>
        {}
    }
    let n12: f32 = n9 + getPositionOffset(art.m_sizeY);
    this.setPosition(n8 as f64, n12 as f64, n10 as f64);
    let n13: f32 = -0.00625;
    this.m_boundingBox.setBounds(
        (n8 - n5 - n13) as f64,
        (n12 - n6 - n13) as f64,
        (n10 - n7 - n13) as f64,
        (n8 + n5 + n13) as f64,
        (n12 + n6 + n13) as f64,
        (n10 + n7 + n13) as f64,
    );
}

pub fn canSurvive(this: &Rc<Entity>) -> bool
{
    let world: Rc<World> = this.world();
    if world.getCollidingBoundingBoxes(this, &this.m_boundingBox).size() > 0
    {
        return false;
    }
    let painting: &PaintingFields = fields(this);
    let art: &EnumArt = art(this);
    let n: i32 = art.m_sizeX / 16;
    let n2: i32 = art.m_sizeY / 16;
    let mut n3: i32 = painting.m_xPosition.get();
    let mut n4: i32 = painting.m_zPosition.get();
    let halfWidth: f64 = (art.m_sizeX as f32 / 32.0) as f64;
    match painting.m_direction.get()
    {
        0 | 2 => n3 = floor_double(this.posX() - halfWidth),
        1 | 3 => n4 = floor_double(this.posZ() - halfWidth),
        _ =>
        {}
    }
    let floorY: i32 = floor_double(this.posY() - (art.m_sizeY as f32 / 32.0) as f64);
    for i in 0..n
    {
        for j in 0..n2
        {
            let direction: i32 = painting.m_direction.get();
            let material: Rc<crate::block::Material> = if direction == 0 || direction == 2
            {
                world.getBlockMaterial(n3 + i, floorY + j, painting.m_zPosition.get())
            }
            else
            {
                world.getBlockMaterial(painting.m_xPosition.get(), floorY + j, n4 + i)
            };
            if !material.isSolid()
            {
                return false;
            }
        }
    }
    let list: Rc<JArrayList<Rc<Entity>>> = world.getEntitiesWithinAABBExcludingEntity(Some(this), &this.m_boundingBox);
    for k in 0..list.size()
    {
        if list.get(k).instanceOf("EntityPainting")
        {
            return false;
        }
    }
    true
}

fn dropAsItem(this: &Rc<Entity>)
{
    let world: Rc<World> = this.world();
    this.setEntityDead();
    world.entityJoinedWorld(&entity_item::create(
        &world,
        this.posX(),
        this.posY(),
        this.posZ(),
        ItemStack::withItem(&Item::painting()),
    ));
}

fn breakOnPush(this: &Rc<Entity>, dx: f64, dy: f64, dz: f64)
{
    if !this.world().m_multiplayerWorld.get() && dx * dx + dy * dy + dz * dz > 0.0
    {
        dropAsItem(this);
    }
}

pub const CLASS: EntityClass = EntityClass {
    m_name: "EntityPainting",
    m_parent: Some(&ENTITY),
    m_interfaces: &[],
    entityInit: |_| {},
    onUpdate: |this| {
        let painting: &PaintingFields = fields(this);
        let ticks: i32 = painting.m_ticksSinceValidation.get();
        painting.m_ticksSinceValidation.set(ticks + 1);
        if ticks == 100 && !this.world().m_multiplayerWorld.get()
        {
            painting.m_ticksSinceValidation.set(0);
            if !canSurvive(this)
            {
                dropAsItem(this);
            }
        }
    },
    canBeCollidedWith: |_| true,
    attackEntityFrom: |this, _source, _amount| {
        if !this.m_isDead.get() && !this.world().m_multiplayerWorld.get()
        {
            this.setEntityDead();
            this.setBeenAttacked();
            let world: Rc<World> = this.world();
            world.entityJoinedWorld(&entity_item::create(
                &world,
                this.posX(),
                this.posY(),
                this.posZ(),
                ItemStack::withItem(&Item::painting()),
            ));
        }
        true
    },
    writeEntityToNBT: |this, nbt| {
        let painting: &PaintingFields = fields(this);
        nbt.setByte("Dir", painting.m_direction.get() as i8);
        nbt.setString("Motive", Some(JString::from(art(this).m_title)));
        nbt.setInteger("TileX", painting.m_xPosition.get());
        nbt.setInteger("TileY", painting.m_yPosition.get());
        nbt.setInteger("TileZ", painting.m_zPosition.get());
    },
    readEntityFromNBT: |this, nbt| {
        let painting: &PaintingFields = fields(this);
        painting.m_direction.set(nbt.getByte("Dir") as i32);
        painting.m_xPosition.set(nbt.getInteger("TileX"));
        painting.m_yPosition.set(nbt.getInteger("TileY"));
        painting.m_zPosition.set(nbt.getInteger("TileZ"));
        let string: Option<JString> = nbt.getString("Motive");
        for art in &EnumArt::VALUES
        {
            if string.as_ref().is_some_and(|motive| JString::from(art.m_title) == *motive)
            {
                painting.m_art.set(Some(art));
            }
        }
        if painting.m_art.get().is_none()
        {
            painting.m_art.set(Some(&EnumArt::VALUES[0]));
        }
        setDirection(this, painting.m_direction.get());
    },
    moveEntity: breakOnPush,
    addVelocity: breakOnPush,
    ..ENTITY
};
