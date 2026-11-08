use crate::tileentity::TileEntity;
use jvm::throwable::OrNpe;
use std::rc::Rc;

pub fn cast(tileEntity: Option<Rc<TileEntity>>, className: &str) -> Rc<TileEntity>
{
    castNullable(tileEntity, className).orNpe()
}

pub fn castNullable(tileEntity: Option<Rc<TileEntity>>, className: &str) -> Option<Rc<TileEntity>>
{
    if let Some(entity) = &tileEntity
        && !entity.instanceOf(className)
    {
        jvm::throwable::throw(
            &jvm::throwable::CLASS_CAST,
            &format!("net.minecraft.src.{} cannot be cast to net.minecraft.src.{}", entity.className(), className),
        );
    }
    tileEntity
}
