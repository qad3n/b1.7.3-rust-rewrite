use crate::entity::{Entity, WatchedValue};
use crate::world::World;
use jvm::throwable::{self, CLASS_CAST};
use std::rc::Rc;

pub fn abstractMethod(name: &str) -> !
{
    throwable::throw(&throwable::ERROR, &format!("java.lang.AbstractMethodError: {name}"))
}

pub fn sameObject<T>(a: Option<&Rc<T>>, b: Option<&Rc<T>>) -> bool
{
    match (a, b)
    {
        (None, None) => true,
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        _ => false,
    }
}

pub fn watchedCastError(found: &WatchedValue, target: &str) -> !
{
    let name: &str = match found
    {
        WatchedValue::Byte(_) => "java.lang.Byte",
        WatchedValue::Short(_) => "java.lang.Short",
        WatchedValue::Int(_) => "java.lang.Integer",
        WatchedValue::Float(_) => "java.lang.Float",
        WatchedValue::String(_) => "java.lang.String",
        WatchedValue::ItemStack(_) => "net.minecraft.src.ItemStack",
        WatchedValue::ChunkCoordinates(_) => "net.minecraft.src.ChunkCoordinates",
    };
    throwable::throw(&CLASS_CAST, &format!("{name} cannot be cast to {target}"))
}

pub fn playSoundAtEntity(world: &World, entity: &Rc<Entity>, sound: Option<String>, volume: f32, pitch: f32)
{
    world.playSoundAtEntity(entity, sound.as_deref().unwrap_or(""), volume, pitch);
}

pub fn classCast(from: &str, to: &str) -> !
{
    throwable::throw(&CLASS_CAST, &format!("net.minecraft.src.{from} cannot be cast to net.minecraft.src.{to}"))
}
