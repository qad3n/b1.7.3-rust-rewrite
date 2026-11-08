use std::rc::Rc;

pub struct MapColor
{
    pub m_colorValue: i32,
    pub m_colorIndex: i32,
}

pub struct MapColors
{
    pub m_mapColorArray: [Option<Rc<MapColor>>; 16],
    pub m_airColor: Rc<MapColor>,
    pub m_grassColor: Rc<MapColor>,
    pub m_sandColor: Rc<MapColor>,
    pub m_clothColor: Rc<MapColor>,
    pub m_tntColor: Rc<MapColor>,
    pub m_iceColor: Rc<MapColor>,
    pub m_ironColor: Rc<MapColor>,
    pub m_foliageColor: Rc<MapColor>,
    pub m_snowColor: Rc<MapColor>,
    pub m_clayColor: Rc<MapColor>,
    pub m_dirtColor: Rc<MapColor>,
    pub m_stoneColor: Rc<MapColor>,
    pub m_waterColor: Rc<MapColor>,
    pub m_woodColor: Rc<MapColor>,
}

thread_local! {
    static COLORS: Rc<MapColors> = Rc::new(MapColors::create());
}

impl MapColors
{
    fn create() -> Self
    {
        let mut array: [Option<Rc<MapColor>>; 16] = Default::default();
        let mut color = |id: i32, value: i32| -> Rc<MapColor> {
            let created: Rc<MapColor> = Rc::new(MapColor { m_colorIndex: id, m_colorValue: value });
            array[id as usize] = Some(Rc::clone(&created));
            created
        };
        let airColor: Rc<MapColor> = color(0, 0);
        let grassColor: Rc<MapColor> = color(1, 8368696);
        let sandColor: Rc<MapColor> = color(2, 16247203);
        let clothColor: Rc<MapColor> = color(3, 0xA7A7A7);
        let tntColor: Rc<MapColor> = color(4, 0xFF0000);
        let iceColor: Rc<MapColor> = color(5, 0xA0A0FF);
        let ironColor: Rc<MapColor> = color(6, 0xA7A7A7);
        let foliageColor: Rc<MapColor> = color(7, 31744);
        let snowColor: Rc<MapColor> = color(8, 0xFFFFFF);
        let clayColor: Rc<MapColor> = color(9, 10791096);
        let dirtColor: Rc<MapColor> = color(10, 12020271);
        let stoneColor: Rc<MapColor> = color(11, 0x707070);
        let waterColor: Rc<MapColor> = color(12, 0x4040FF);
        let woodColor: Rc<MapColor> = color(13, 6837042);
        Self {
            m_mapColorArray: array,
            m_airColor: airColor,
            m_grassColor: grassColor,
            m_sandColor: sandColor,
            m_clothColor: clothColor,
            m_tntColor: tntColor,
            m_iceColor: iceColor,
            m_ironColor: ironColor,
            m_foliageColor: foliageColor,
            m_snowColor: snowColor,
            m_clayColor: clayColor,
            m_dirtColor: dirtColor,
            m_stoneColor: stoneColor,
            m_waterColor: waterColor,
            m_woodColor: woodColor,
        }
    }
}

impl MapColor
{
    pub fn colors() -> Rc<MapColors> { COLORS.with(Rc::clone) }
}
