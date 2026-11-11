use game::block::BlockClass;
use game::entity::EntityClass;
use game::item::ItemClass;
use game::tileentity::TileEntityClass;

fn leakName(name: &str) -> &'static str { Box::leak(name.to_owned().into_boxed_str()) }

// copies a template vtable once per registered class and lets the caller swap only overridden entries
macro_rules! derive_class
{
    ($name:ident, $class:ty) =>
    {
        pub fn $name(template: &'static $class, name: &str, patch: impl FnOnce(&mut $class)) -> &'static $class
        {
            let mut class: $class = *template;
            class.m_name = leakName(name);
            class.m_parent = Some(template);
            patch(&mut class);
            Box::leak(Box::new(class))
        }
    };
}

derive_class!(deriveBlockClass, BlockClass);
derive_class!(deriveItemClass, ItemClass);
derive_class!(deriveEntityClass, EntityClass);
derive_class!(deriveTileEntityClass, TileEntityClass);
