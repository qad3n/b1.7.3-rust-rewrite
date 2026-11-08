use super::{EnumStatus, PlayerFields, fields, setPlayerSpawnCoordinate};
use crate::block::{Block, block_bed};
use crate::entity::Entity;
use crate::world::{ChunkCoordinates, IChunkProvider, World};
use std::rc::Rc;

pub fn sleepInBedAt(this: &Rc<Entity>, x: i32, y: i32, z: i32) -> EnumStatus
{
    let player: &PlayerFields = fields(this);
    let world: Rc<World> = this.world();
    if !world.m_multiplayerWorld.get()
    {
        if this.isPlayerSleeping() || !this.isEntityAlive()
        {
            return EnumStatus::OtherProblem;
        }
        if world.worldProvider().m_isNether.get()
        {
            return EnumStatus::NotPossibleHere;
        }
        if world.isDaytime()
        {
            return EnumStatus::NotPossibleNow;
        }
        if (this.posX() - x as f64).abs() > 3.0
            || (this.posY() - y as f64).abs() > 2.0
            || (this.posZ() - z as f64).abs() > 3.0
        {
            return EnumStatus::TooFarAway;
        }
    }
    this.setSize(0.2, 0.2);
    this.m_yOffset.set(0.2);
    if world.blockExists(x, y, z)
    {
        let i: i32 = world.getBlockMetadata(x, y, z);
        let j: i32 = block_bed::getDirectionFromMetadata(i);
        let mut f: f32 = 0.5;
        let mut f1: f32 = 0.5;
        match j
        {
            0 => f1 = 0.9,
            2 => f1 = 0.1,
            1 => f = 0.1,
            3 => f = 0.9,
            _ =>
            {}
        }
        setSleepingCameraOffset(player, j);
        this.setPosition((x as f32 + f) as f64, (y as f32 + 0.9375) as f64, (z as f32 + f1) as f64);
    }
    else
    {
        this.setPosition((x as f32 + 0.5) as f64, (y as f32 + 0.9375) as f64, (z as f32 + 0.5) as f64);
    }
    player.m_sleeping.set(true);
    player.m_sleepTimer.set(0);
    player.m_bedChunkCoordinates.set(Some(ChunkCoordinates::new(x, y, z)));
    this.m_motionY.set(0.0);
    this.m_motionZ.set(0.0);
    this.m_motionX.set(0.0);
    if !world.m_multiplayerWorld.get()
    {
        world.updateAllPlayersSleepingFlag();
    }
    EnumStatus::Ok
}

fn setSleepingCameraOffset(player: &PlayerFields, facing: i32)
{
    player.m_sleepingCameraOffsetX.set(0.0);
    player.m_sleepingCameraOffsetZ.set(0.0);
    match facing
    {
        0 => player.m_sleepingCameraOffsetZ.set(-1.8),
        1 => player.m_sleepingCameraOffsetX.set(1.8),
        2 => player.m_sleepingCameraOffsetZ.set(1.8),
        3 => player.m_sleepingCameraOffsetX.set(-1.8),
        _ =>
        {}
    }
}

pub fn wakeUpPlayer(this: &Rc<Entity>, resetSleepTimer: bool, updateAllPlayersSleeping: bool, setSpawnPoint: bool)
{
    let player: &PlayerFields = fields(this);
    let world: Rc<World> = this.world();
    this.setSize(0.6, 1.8);
    this.resetHeight();
    let chunkcoordinates: Option<ChunkCoordinates> = player.m_bedChunkCoordinates.get();
    if let Some(bed) = chunkcoordinates
        && world.getBlockId(bed.m_x, bed.m_y, bed.m_z) == Block::blockBed().m_blockID
    {
        block_bed::setBedOccupied(&world, bed.m_x, bed.m_y, bed.m_z, false);
        let chunkcoordinates1: ChunkCoordinates =
            match block_bed::getNearestEmptyChunkCoordinates(&world, bed.m_x, bed.m_y, bed.m_z, 0)
            {
                Some(found) => found,
                None => ChunkCoordinates::new(bed.m_x, bed.m_y + 1, bed.m_z),
            };
        this.setPosition(
            (chunkcoordinates1.m_x as f32 + 0.5) as f64,
            (chunkcoordinates1.m_y as f32 + this.m_yOffset.get() + 0.1) as f64,
            (chunkcoordinates1.m_z as f32 + 0.5) as f64,
        );
    }
    player.m_sleeping.set(false);
    if !world.m_multiplayerWorld.get() && updateAllPlayersSleeping
    {
        world.updateAllPlayersSleepingFlag();
    }
    if resetSleepTimer
    {
        player.m_sleepTimer.set(0);
    }
    else
    {
        player.m_sleepTimer.set(100);
    }
    if setSpawnPoint
    {
        setPlayerSpawnCoordinate(this, player.m_bedChunkCoordinates.get());
    }
}

pub(super) fn isInBed(this: &Rc<Entity>) -> bool
{
    let bed: ChunkCoordinates = jvm::throwable::OrNpe::orNpe(fields(this).m_bedChunkCoordinates.get());
    this.world().getBlockId(bed.m_x, bed.m_y, bed.m_z) == Block::blockBed().m_blockID
}

pub fn loadSpawnPoint(world: &Rc<World>, pos: &ChunkCoordinates) -> Option<ChunkCoordinates>
{
    let ichunkprovider: Rc<dyn IChunkProvider> = world.getIChunkProvider();
    ichunkprovider.prepareChunk((pos.m_x - 3) >> 4, (pos.m_z - 3) >> 4);
    ichunkprovider.prepareChunk((pos.m_x + 3) >> 4, (pos.m_z - 3) >> 4);
    ichunkprovider.prepareChunk((pos.m_x - 3) >> 4, (pos.m_z + 3) >> 4);
    ichunkprovider.prepareChunk((pos.m_x + 3) >> 4, (pos.m_z + 3) >> 4);
    if world.getBlockId(pos.m_x, pos.m_y, pos.m_z) != Block::blockBed().m_blockID
    {
        return None;
    }
    block_bed::getNearestEmptyChunkCoordinates(world, pos.m_x, pos.m_y, pos.m_z, 0)
}

pub fn getBedOrientationInDegrees(this: &Entity) -> f32
{
    if let Some(bed) = fields(this).m_bedChunkCoordinates.get()
    {
        let i: i32 = this.world().getBlockMetadata(bed.m_x, bed.m_y, bed.m_z);
        match block_bed::getDirectionFromMetadata(i)
        {
            0 => return 90.0,
            1 => return 0.0,
            2 => return 270.0,
            3 => return 180.0,
            _ =>
            {}
        }
    }
    0.0
}

pub fn isPlayerFullyAsleep(this: &Entity) -> bool
{
    fields(this).m_sleeping.get() && fields(this).m_sleepTimer.get() >= 100
}

pub fn getSleepingTime(this: &Entity) -> i32 { fields(this).m_sleepTimer.get() }
