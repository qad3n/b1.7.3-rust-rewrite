use crate::block::Block;
use crate::crafting::{CraftingManager, Ingredient};
use crate::item::{Item, ItemStack};

fn dye(size: i32, metadata: i32) -> Ingredient
{
    Ingredient::Stack(ItemStack::withItemDamage(&Item::dyePowder(), size, metadata))
}

pub(super) fn addRecipes(manager: &CraftingManager)
{
    for i in 0..16
    {
        manager.addShapelessRecipe(
            ItemStack::withBlockDamage(
                &Block::cloth(),
                1,
                crate::block::block_cloth::getItemMetadataForBlockMetadata(i),
            ),
            &[
                dye(1, i),
                Ingredient::Stack(ItemStack::withItemDamage(&Item::byIdOrNpe(Block::cloth().m_blockID), 1, 0)),
            ],
        );
    }

    manager.addShapelessRecipe(
        ItemStack::withItemDamage(&Item::dyePowder(), 2, 11),
        &[Ingredient::Block(Block::plantYellow())],
    );
    manager.addShapelessRecipe(
        ItemStack::withItemDamage(&Item::dyePowder(), 2, 1),
        &[Ingredient::Block(Block::plantRed())],
    );
    manager.addShapelessRecipe(ItemStack::withItemDamage(&Item::dyePowder(), 3, 15), &[Ingredient::Item(Item::bone())]);
    manager.addShapelessRecipe(ItemStack::withItemDamage(&Item::dyePowder(), 2, 9), &[dye(1, 1), dye(1, 15)]);
    manager.addShapelessRecipe(ItemStack::withItemDamage(&Item::dyePowder(), 2, 14), &[dye(1, 1), dye(1, 11)]);
    manager.addShapelessRecipe(ItemStack::withItemDamage(&Item::dyePowder(), 2, 10), &[dye(1, 2), dye(1, 15)]);
    manager.addShapelessRecipe(ItemStack::withItemDamage(&Item::dyePowder(), 2, 8), &[dye(1, 0), dye(1, 15)]);
    manager.addShapelessRecipe(ItemStack::withItemDamage(&Item::dyePowder(), 2, 7), &[dye(1, 8), dye(1, 15)]);
    manager
        .addShapelessRecipe(ItemStack::withItemDamage(&Item::dyePowder(), 3, 7), &[dye(1, 0), dye(1, 15), dye(1, 15)]);
    manager.addShapelessRecipe(ItemStack::withItemDamage(&Item::dyePowder(), 2, 12), &[dye(1, 4), dye(1, 15)]);
    manager.addShapelessRecipe(ItemStack::withItemDamage(&Item::dyePowder(), 2, 6), &[dye(1, 4), dye(1, 2)]);
    manager.addShapelessRecipe(ItemStack::withItemDamage(&Item::dyePowder(), 2, 5), &[dye(1, 4), dye(1, 1)]);
    manager.addShapelessRecipe(ItemStack::withItemDamage(&Item::dyePowder(), 2, 13), &[dye(1, 5), dye(1, 9)]);
    manager
        .addShapelessRecipe(ItemStack::withItemDamage(&Item::dyePowder(), 3, 13), &[dye(1, 4), dye(1, 1), dye(1, 9)]);
    manager.addShapelessRecipe(
        ItemStack::withItemDamage(&Item::dyePowder(), 4, 13),
        &[dye(1, 4), dye(1, 1), dye(1, 1), dye(1, 15)],
    );
}
