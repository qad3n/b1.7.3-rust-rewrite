use crate::block::Block;
use crate::block::map_color::MapColor;
use crate::entity::Entity;
use crate::item::{Item, ItemClass, ItemStack, MapData, item_map_base};
use crate::math::math_helper;
use crate::network::packets::Packet131MapData;
use crate::world::{Chunk, MapDataBase, MapDataKind, World, WorldProvider};
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub fn getMapData(mapId: i16, world: &Rc<World>) -> Rc<MapDataBase>
{
    let mut string: String = format!("map_{mapId}");
    let mut mapData: Option<Rc<MapDataBase>> = world.loadItemData(MapDataKind::MapData, &format!("map_{mapId}"));
    if mapData.is_none()
    {
        let n: i32 = world.getUniqueDataId("map");
        string = format!("map_{n}");
        let data: Rc<MapDataBase> = MapDataBase::newMap(&string);
        world.setItemData(&string, Rc::clone(&data));
        mapData = Some(data);
    }
    mapData.orNpe()
}

pub fn getSavedMapData(item: &ItemStack, world: &Rc<World>) -> Rc<MapDataBase>
{
    let mut string: String = format!("map_{}", item.getItemDamage());
    let mut mapData: Option<Rc<MapDataBase>> =
        world.loadItemData(MapDataKind::MapData, &format!("map_{}", item.getItemDamage()));
    if mapData.is_none()
    {
        item.setItemDamage(world.getUniqueDataId("map"));
        string = format!("map_{}", item.getItemDamage());
        let data: Rc<MapDataBase> = MapDataBase::newMap(&string);
        data.m_map.m_centerX.set(world.getWorldInfo().getSpawnX());
        data.m_map.m_centerZ.set(world.getWorldInfo().getSpawnZ());
        data.m_map.m_scale.set(3);
        data.m_map.m_dimension.set(world.worldProvider().m_worldType.get() as i8);
        data.markDirty();
        world.setItemData(&string, Rc::clone(&data));
        mapData = Some(data);
    }
    mapData.orNpe()
}

pub fn update(world: &Rc<World>, entity: &Rc<Entity>, base: &MapDataBase)
{
    let data: &MapData = &base.m_map;
    let provider: Rc<WorldProvider> = world.worldProvider();
    if provider.m_worldType.get() != data.m_dimension.get() as i32
    {
        return;
    }
    let n: i32 = 128;
    let n2: i32 = 128;
    let n3: i32 = 1 << data.m_scale.get();
    let n4: i32 = data.m_centerX.get();
    let n5: i32 = data.m_centerZ.get();
    let n6: i32 = math_helper::floor_double(entity.m_posX.get() - n4 as f64) / n3 + n / 2;
    let n7: i32 = math_helper::floor_double(entity.m_posZ.get() - n5 as f64) / n3 + n2 / 2;
    let mut n8: i32 = 128 / n3;
    if provider.m_hasNoSky.get()
    {
        n8 /= 2;
    }
    data.m_ticks.set(data.m_ticks.get() + 1);
    for i in n6 - n8 + 1..n6 + n8
    {
        if (i & 0xF) != (data.m_ticks.get() & 0xF)
        {
            continue;
        }
        let mut n9: i32 = 255;
        let mut n10: i32 = 0;
        let mut d: f64 = 0.0;
        for j in n7 - n8 - 1..n7 + n8
        {
            if i < 0 || j < -1 || i >= n || j >= n2
            {
                continue;
            }
            let n16: i32 = i - n6;
            let n17: i32 = j - n7;
            let bl: bool = n16 * n16 + n17 * n17 > (n8 - 2) * (n8 - 2);
            let n18: i32 = (n4 / n3 + i - n / 2) * n3;
            let n19: i32 = (n5 / n3 + j - n2 / 2) * n3;
            let mut nArray: [i32; 256] = [0; 256];
            let chunk: Rc<Chunk> = world.getChunkFromBlockCoords(n18, n19);
            let n23: i32 = n18 & 0xF;
            let n24: i32 = n19 & 0xF;
            let mut n25: i32 = 0;
            let mut d2: f64 = 0.0;
            if provider.m_hasNoSky.get()
            {
                let mut n15: i32 = n18.wrapping_add(n19.wrapping_mul(231871));
                n15 = n15.wrapping_mul(n15).wrapping_mul(31287121).wrapping_add(n15.wrapping_mul(11));
                if ((n15 >> 20) & 1) == 0
                {
                    nArray[Block::dirt().m_blockID as usize] += 10;
                }
                else
                {
                    nArray[Block::stone().m_blockID as usize] += 10;
                }
                d2 = 100.0;
            }
            else
            {
                for n15 in 0..n3
                {
                    for n14 in 0..n3
                    {
                        let mut n13: i32 = chunk.getHeightValue(n15 + n23, n14 + n24) + 1;
                        let mut n28: i32 = 0;
                        if n13 > 1
                        {
                            loop
                            {
                                let mut n12: i32 = 1;
                                n28 = chunk.getBlockID(n15 + n23, n13 - 1, n14 + n24);
                                if n28 == 0
                                {
                                    n12 = 0;
                                }
                                else if n13 > 0
                                    && n28 > 0
                                    && Rc::ptr_eq(
                                        &Block::byId(n28).orNpe().m_blockMaterial.m_materialMapColor,
                                        &MapColor::colors().m_airColor,
                                    )
                                {
                                    n12 = 0;
                                }
                                if n12 != 0
                                {
                                    break;
                                }
                                n13 -= 1;
                                n28 = chunk.getBlockID(n15 + n23, n13 - 1, n14 + n24);
                            }
                            if n28 != 0 && Block::byId(n28).orNpe().m_blockMaterial.getIsLiquid()
                            {
                                let mut n11: i32 = n13 - 1;
                                loop
                                {
                                    let n29: i32 = chunk.getBlockID(n15 + n23, n11, n14 + n24);
                                    n11 -= 1;
                                    n25 += 1;
                                    if !(n11 > 0 && n29 != 0 && Block::byId(n29).orNpe().m_blockMaterial.getIsLiquid())
                                    {
                                        break;
                                    }
                                }
                            }
                        }
                        d2 += n13 as f64 / (n3 * n3) as f64;
                        nArray[n28 as usize] += 1;
                    }
                }
            }
            n25 /= n3 * n3;
            let mut n15: i32 = 0;
            let mut n14: i32 = 0;
            for (n13, &count) in nArray.iter().enumerate()
            {
                if count <= n15
                {
                    continue;
                }
                n14 = n13 as i32;
                n15 = count;
            }
            let mut d3: f64 = (d2 - d) * 4.0 / (n3 + 4) as f64 + (((i + j) & 1) as f64 - 0.5) * 0.4;
            let mut n12: i32 = 1;
            if d3 > 0.6
            {
                n12 = 2;
            }
            if d3 < -0.6
            {
                n12 = 0;
            }
            let mut n11: i32 = 0;
            if n14 > 0
            {
                let mapColor: Rc<MapColor> = Block::byId(n14).orNpe().m_blockMaterial.m_materialMapColor.clone();
                if Rc::ptr_eq(&mapColor, &MapColor::colors().m_waterColor)
                {
                    d3 = n25 as f64 * 0.1 + ((i + j) & 1) as f64 * 0.2;
                    n12 = 1;
                    if d3 < 0.5
                    {
                        n12 = 2;
                    }
                    if d3 > 0.9
                    {
                        n12 = 0;
                    }
                }
                n11 = mapColor.m_colorIndex;
            }
            d = d2;
            if j < 0 || n16 * n16 + n17 * n17 >= n8 * n8 || bl && ((i + j) & 1) == 0
            {
                continue;
            }
            let by2: i8 = data.m_colors.borrow()[(i + j * n) as usize];
            let by: i8 = (n11 * 4 + n12) as i8;
            if by2 == by
            {
                continue;
            }
            if n9 > j
            {
                n9 = j;
            }
            if n10 < j
            {
                n10 = j;
            }
            data.m_colors.borrow_mut()[(i + j * n) as usize] = by;
        }
        if n9 > n10
        {
            continue;
        }
        data.markDirtyColumn(base, i, n9, n10);
    }
}

pub const CLASS: ItemClass = ItemClass {
    m_name: "ItemMap",
    m_parent: Some(&item_map_base::CLASS),
    onUpdate: |_, stack, world, entity, _, selected| {
        if world.m_multiplayerWorld.get()
        {
            return;
        }
        let mapData: Rc<MapDataBase> = getSavedMapData(stack, world);
        if entity.instanceOf("EntityPlayer")
        {
            mapData.m_map.tickHolder(entity, stack);
        }
        if selected
        {
            update(world, entity, &mapData);
        }
    },
    onCreated: |_, stack, world, player| {
        stack.setItemDamage(world.getUniqueDataId("map"));
        let string: String = format!("map_{}", stack.getItemDamage());
        let mapData: Rc<MapDataBase> = MapDataBase::newMap(&string);
        world.setItemData(&string, Rc::clone(&mapData));
        mapData.m_map.m_centerX.set(math_helper::floor_double(player.m_posX.get()));
        mapData.m_map.m_centerZ.set(math_helper::floor_double(player.m_posZ.get()));
        mapData.m_map.m_scale.set(3);
        mapData.m_map.m_dimension.set(world.worldProvider().m_worldType.get() as i8);
        mapData.markDirty();
    },
    getUpdatePacket: |_, stack, world, player| {
        let byArray: Vec<i8> = getSavedMapData(stack, world).m_map.getUpdateData(stack, world, player)?;
        Some(Box::new(Packet131MapData::new(
            Item::mapItem().m_shiftedIndex as i16,
            stack.getItemDamage() as i16,
            byArray.into_iter().map(|value| value as u8).collect(),
        )))
    },
    ..item_map_base::CLASS
};

pub fn create(id: i32) -> Rc<Item> { item_map_base::construct(&CLASS, Box::new(()), id).setMaxStackSize(1) }
