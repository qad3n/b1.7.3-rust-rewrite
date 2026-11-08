use crate::player::entity_player_mp;
use game::block::Block;
use game::entity::{Entity, entity_player};
use game::item::ItemStack;
use game::network::packets::Packet53BlockChange;
use game::world::World;
use jvm::field::Field;
use jvm::throwable::OrNpe;
use std::cell::Cell;
use std::rc::Rc;

pub struct ItemInWorldManager
{
    m_thisWorld: Rc<World>,
    pub m_thisPlayer: Field<Option<Rc<Entity>>>,
    m_field_672_d: Cell<f32>,
    m_miningStartTime: Cell<i32>,
    m_targetX: Cell<i32>,
    m_targetY: Cell<i32>,
    m_targetZ: Cell<i32>,
    m_ticks: Cell<i32>,
    m_wasMiningBlock: Cell<bool>,
    m_prevTargetX: Cell<i32>,
    m_prevTargetY: Cell<i32>,
    m_prevTargetZ: Cell<i32>,
    m_prevMiningStartTime: Cell<i32>,
}

impl ItemInWorldManager
{
    pub fn new(world: &Rc<World>) -> Rc<ItemInWorldManager>
    {
        Rc::new(ItemInWorldManager {
            m_thisPlayer: Field::new(None),
            m_field_672_d: Cell::new(0.0),
            m_miningStartTime: Cell::new(0),
            m_targetX: Cell::new(0),
            m_targetY: Cell::new(0),
            m_targetZ: Cell::new(0),
            m_ticks: Cell::new(0),
            m_wasMiningBlock: Cell::new(false),
            m_prevTargetX: Cell::new(0),
            m_prevTargetY: Cell::new(0),
            m_prevTargetZ: Cell::new(0),
            m_prevMiningStartTime: Cell::new(0),
            m_thisWorld: Rc::clone(world),
        })
    }

    fn thisPlayer(&self) -> Rc<Entity> { self.m_thisPlayer.get().orNpe() }

    pub fn tick(&self)
    {
        self.m_ticks.set(self.m_ticks.get() + 1);
        if self.m_wasMiningBlock.get()
        {
            let n: i32 = self.m_ticks.get() - self.m_prevMiningStartTime.get();
            let n2: i32 = self.m_thisWorld.getBlockId(
                self.m_prevTargetX.get(),
                self.m_prevTargetY.get(),
                self.m_prevTargetZ.get(),
            );
            if n2 != 0
            {
                let block: Rc<Block> = Block::byId(n2).orNpe();
                let f: f32 = block.blockStrength(&self.thisPlayer()) * (n + 1) as f32;
                if f >= 1.0
                {
                    self.m_wasMiningBlock.set(false);
                    self.tryMineBlock(self.m_prevTargetX.get(), self.m_prevTargetY.get(), self.m_prevTargetZ.get());
                }
            }
            else
            {
                self.m_wasMiningBlock.set(false);
            }
        }
    }

    pub fn startMiningBlock(&self, x: i32, y: i32, z: i32, face: i32)
    {
        self.m_thisWorld.onBlockHit(None, x, y, z, face);
        self.m_miningStartTime.set(self.m_ticks.get());
        let n: i32 = self.m_thisWorld.getBlockId(x, y, z);
        if n > 0
        {
            Block::byId(n).orNpe().onBlockClicked(&self.m_thisWorld, x, y, z, &self.thisPlayer());
        }
        if n > 0 && Block::byId(n).orNpe().blockStrength(&self.thisPlayer()) >= 1.0
        {
            self.tryMineBlock(x, y, z);
        }
        else
        {
            self.m_targetX.set(x);
            self.m_targetY.set(y);
            self.m_targetZ.set(z);
        }
    }

    pub fn finishMiningBlock(&self, x: i32, y: i32, z: i32)
    {
        if x == self.m_targetX.get() && y == self.m_targetY.get() && z == self.m_targetZ.get()
        {
            let n: i32 = self.m_ticks.get() - self.m_miningStartTime.get();
            let n2: i32 = self.m_thisWorld.getBlockId(x, y, z);
            if n2 != 0
            {
                let block: Rc<Block> = Block::byId(n2).orNpe();
                let f: f32 = block.blockStrength(&self.thisPlayer()) * (n + 1) as f32;
                if f >= 0.7
                {
                    self.tryMineBlock(x, y, z);
                }
                else if !self.m_wasMiningBlock.get()
                {
                    self.m_wasMiningBlock.set(true);
                    self.m_prevTargetX.set(x);
                    self.m_prevTargetY.set(y);
                    self.m_prevTargetZ.set(z);
                    self.m_prevMiningStartTime.set(self.m_miningStartTime.get());
                }
            }
        }
        self.m_field_672_d.set(0.0);
    }

    pub fn removeBlock(&self, x: i32, y: i32, z: i32) -> bool
    {
        let block: Option<Rc<Block>> = Block::byId(self.m_thisWorld.getBlockId(x, y, z));
        let n: i32 = self.m_thisWorld.getBlockMetadata(x, y, z);
        let bl: bool = self.m_thisWorld.setBlockWithNotify(x, y, z, 0);
        if let Some(block) = block
            && bl
        {
            block.onBlockDestroyedByPlayer(&self.m_thisWorld, x, y, z, n);
        }
        bl
    }

    pub fn tryMineBlock(&self, x: i32, y: i32, z: i32) -> bool
    {
        let n: i32 = self.m_thisWorld.getBlockId(x, y, z);
        let n2: i32 = self.m_thisWorld.getBlockMetadata(x, y, z);
        self.m_thisWorld.doEventFrom(
            Some(&self.thisPlayer()),
            2001,
            x,
            y,
            z,
            n + self.m_thisWorld.getBlockMetadata(x, y, z) * 256,
        );
        let bl: bool = self.removeBlock(x, y, z);
        let itemStack: Option<Rc<ItemStack>> = entity_player::getCurrentEquippedItem(&self.thisPlayer());
        if let Some(itemStack) = itemStack
        {
            itemStack.onDestroyBlock(n, x, y, z, &self.thisPlayer());
            if itemStack.m_stackSize.get() == 0
            {
                itemStack.onRemoved(&self.thisPlayer());
                entity_player::destroyCurrentEquippedItem(&self.thisPlayer());
            }
        }
        if bl && entity_player::canHarvestBlock(&self.thisPlayer(), &Block::byId(n).orNpe())
        {
            Block::byId(n).orNpe().harvestBlock(&self.m_thisWorld, &self.thisPlayer(), x, y, z, n2);
            entity_player_mp::netServerHandler(&self.thisPlayer()).sendPacket(Box::new(Packet53BlockChange::new(
                x,
                y,
                z,
                &self.m_thisWorld,
            )));
        }
        bl
    }

    pub fn useItem(&self, player: &Rc<Entity>, world: &Rc<World>, itemInHand: &Rc<ItemStack>) -> bool
    {
        let n: i32 = itemInHand.m_stackSize.get();
        let itemStack: Option<Rc<ItemStack>> = itemInHand.useItemRightClick(world, player);
        let same: bool = matches!(&itemStack, Some(itemStack) if Rc::ptr_eq(itemStack, itemInHand));
        if !same || itemStack.as_ref().is_some_and(|itemStack| itemStack.m_stackSize.get() != n)
        {
            let inventory: Rc<game::inventory::InventoryPlayer> = entity_player::inventory(player);
            let currentItem: usize = inventory.m_currentItem.get() as usize;
            inventory.m_mainInventory.borrow_mut()[currentItem] = itemStack.clone();
            if itemStack.orNpe().m_stackSize.get() == 0
            {
                inventory.m_mainInventory.borrow_mut()[currentItem] = None;
            }
            return true;
        }
        false
    }

    pub fn activeBlockOrUseItem(
        &self,
        player: &Rc<Entity>,
        world: &Rc<World>,
        itemInHand: Option<&Rc<ItemStack>>,
        x: i32,
        y: i32,
        z: i32,
        face: i32,
    ) -> bool
    {
        let n: i32 = world.getBlockId(x, y, z);
        if n > 0 && Block::byId(n).orNpe().blockActivated(world, x, y, z, player)
        {
            return true;
        }
        let Some(itemInHand) = itemInHand
        else
        {
            return false;
        };
        itemInHand.useItem(player, world, x, y, z, face)
    }
}
