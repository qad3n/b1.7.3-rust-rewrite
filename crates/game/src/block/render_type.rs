// beta's block shapes by render type id, a mod names one to draw its block in that shape
pub struct RenderTypeRow
{
    pub m_id: i32,
    pub m_name: &'static str,
}

pub const RENDER_TYPES: [RenderTypeRow; 18] = [
    RenderTypeRow { m_id: 0, m_name: "block" },
    RenderTypeRow { m_id: 1, m_name: "cross" },
    RenderTypeRow { m_id: 2, m_name: "torch" },
    RenderTypeRow { m_id: 3, m_name: "fire" },
    RenderTypeRow { m_id: 4, m_name: "fluid" },
    RenderTypeRow { m_id: 5, m_name: "redstone_wire" },
    RenderTypeRow { m_id: 6, m_name: "crops" },
    RenderTypeRow { m_id: 7, m_name: "door" },
    RenderTypeRow { m_id: 8, m_name: "ladder" },
    RenderTypeRow { m_id: 9, m_name: "rail" },
    RenderTypeRow { m_id: 10, m_name: "stairs" },
    RenderTypeRow { m_id: 11, m_name: "fence" },
    RenderTypeRow { m_id: 12, m_name: "lever" },
    RenderTypeRow { m_id: 13, m_name: "cactus" },
    RenderTypeRow { m_id: 14, m_name: "bed" },
    RenderTypeRow { m_id: 15, m_name: "repeater" },
    RenderTypeRow { m_id: 16, m_name: "piston_base" },
    RenderTypeRow { m_id: 17, m_name: "piston_head" },
];

pub fn byName(name: &str) -> Option<i32> { RENDER_TYPES.iter().find(|row| row.m_name == name).map(|row| row.m_id) }
