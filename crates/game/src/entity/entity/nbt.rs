use crate::entity::Entity;
use common::nbt::{NBTBase, NBTTagCompound, NBTTagDouble, NBTTagFloat, NBTTagList};
use std::rc::Rc;

impl Entity
{
    pub fn addEntityID(self: &Rc<Self>, nbt: &NBTTagCompound) -> bool
    {
        let string = self.getEntityString();
        if self.m_isDead.get() || string.is_none()
        {
            return false;
        }
        nbt.setString("id", string);
        self.writeToNBT(nbt);
        true
    }

    pub fn writeToNBT(self: &Rc<Self>, nbt: &NBTTagCompound)
    {
        nbt.setTag("Pos", &self.newDoubleNBTList(&[self.posX(), self.posY() + self.m_ySize.get() as f64, self.posZ()]));
        nbt.setTag(
            "Motion",
            &self.newDoubleNBTList(&[self.m_motionX.get(), self.m_motionY.get(), self.m_motionZ.get()]),
        );
        nbt.setTag("Rotation", &self.newFloatNBTList(&[self.m_rotationYaw.get(), self.m_rotationPitch.get()]));
        nbt.setFloat("FallDistance", self.m_fallDistance.get());
        nbt.setShort("Fire", self.m_fire.get() as i16);
        nbt.setShort("Air", self.m_air.get() as i16);
        nbt.setBoolean("OnGround", self.m_onGround.get());
        self.writeEntityToNBT(nbt);
    }

    pub fn readFromNBT(self: &Rc<Self>, nbt: &NBTTagCompound)
    {
        let nbttaglist: Rc<NBTBase> = nbt.getTagList("Pos");
        let nbttaglist1: Rc<NBTBase> = nbt.getTagList("Motion");
        let nbttaglist2: Rc<NBTBase> = nbt.getTagList("Rotation");
        let double =
            |list: &Rc<NBTBase>, index: i32| -> f64 { list.list().tagAt(index).doubleTag().m_doubleValue.get() };
        let float = |list: &Rc<NBTBase>, index: i32| -> f32 { list.list().tagAt(index).floatTag().m_floatValue.get() };
        self.m_motionX.set(double(&nbttaglist1, 0));
        self.m_motionY.set(double(&nbttaglist1, 1));
        self.m_motionZ.set(double(&nbttaglist1, 2));
        if self.m_motionX.get().abs() > 10.0
        {
            self.m_motionX.set(0.0);
        }
        if self.m_motionY.get().abs() > 10.0
        {
            self.m_motionY.set(0.0);
        }
        if self.m_motionZ.get().abs() > 10.0
        {
            self.m_motionZ.set(0.0);
        }

        let x: f64 = double(&nbttaglist, 0);
        self.m_posX.set(x);
        self.m_lastTickPosX.set(x);
        self.m_prevPosX.set(x);
        let y: f64 = double(&nbttaglist, 1);
        self.m_posY.set(y);
        self.m_lastTickPosY.set(y);
        self.m_prevPosY.set(y);
        let z: f64 = double(&nbttaglist, 2);
        self.m_posZ.set(z);
        self.m_lastTickPosZ.set(z);
        self.m_prevPosZ.set(z);
        let yaw: f32 = float(&nbttaglist2, 0);
        self.m_rotationYaw.set(yaw);
        self.m_prevRotationYaw.set(yaw);
        let pitch: f32 = float(&nbttaglist2, 1);
        self.m_rotationPitch.set(pitch);
        self.m_prevRotationPitch.set(pitch);
        self.m_fallDistance.set(nbt.getFloat("FallDistance"));
        self.m_fire.set(nbt.getShort("Fire") as i32);
        self.m_air.set(nbt.getShort("Air") as i32);
        self.m_onGround.set(nbt.getBoolean("OnGround"));
        self.setPosition(self.posX(), self.posY(), self.posZ());
        self.setRotation(self.m_rotationYaw.get(), self.m_rotationPitch.get());
        self.readEntityFromNBT(nbt);
    }

    pub fn newDoubleNBTList(&self, values: &[f64]) -> Rc<NBTBase>
    {
        let nbttaglist: Rc<NBTBase> = NBTTagList::create();
        for &d0 in values
        {
            nbttaglist.list().setTag(NBTTagDouble::create(d0));
        }
        nbttaglist
    }

    pub fn newFloatNBTList(&self, values: &[f32]) -> Rc<NBTBase>
    {
        let nbttaglist: Rc<NBTBase> = NBTTagList::create();
        for &f in values
        {
            nbttaglist.list().setTag(NBTTagFloat::create(f));
        }
        nbttaglist
    }
}
