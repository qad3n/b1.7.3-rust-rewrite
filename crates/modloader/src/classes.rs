use game::block::BlockClass;
use game::entity::EntityClass;
use game::item::ItemClass;
use game::tileentity::TileEntityClass;

fn leakName(name: &str) -> &'static str { Box::leak(name.to_owned().into_boxed_str()) }

// copies a template vtable once per registered class and lets the caller swap only overridden entries
macro_rules! derive_class {
    ($name:ident, $class:ty) => {
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

#[cfg(test)]
mod tests
{
    use super::*;
    use game::block::Block;
    use std::rc::Rc;

    #[test]
    fn derivedClassKeepsTemplateAndOverrides()
    {
        let torch: Rc<Block> = Block::torchWood();
        let class: &'static BlockClass = deriveBlockClass(torch.m_class, "test:lamp", |class| {
            class.tickRate = |_| 3;
        });
        assert!(class.isSubclassOf("BlockTorch"));
        assert_eq!(class.m_name, "test:lamp");
        assert_eq!(class.getRenderType as usize, torch.m_class.getRenderType as usize);
        let lamp: Rc<Block> = game::hooks::withBlockClass(class, || {
            game::block::block_torch::create(250, torch.m_blockIndexInTexture.get())
        });
        assert_eq!(lamp.tickRate(), 3);
        assert_eq!(lamp.getRenderType(), torch.getRenderType());
    }
}
