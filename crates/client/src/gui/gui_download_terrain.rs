use crate::gui::GuiScreen;
use crate::gui::gui_screen::{GUI_SCREEN, GuiScreenClass};
use crate::network::net_client_handler::NetClientHandler;
use game::locale::StringTranslate;
use game::network::packets::Packet0KeepAlive;
use std::cell::Cell;
use std::rc::Rc;

struct DownloadTerrainExt
{
    m_netHandler: Rc<NetClientHandler>,
    m_updateCounter: Cell<i32>,
}

const CLASS: GuiScreenClass = GuiScreenClass {
    m_name: "GuiDownloadTerrain",
    m_parent: Some(&GUI_SCREEN),
    m_interfaces: &[],
    keyTyped: |_, _, _| {},
    initGui: |this| this.m_controlList.clear(),
    updateScreen: |this| {
        let ext: &DownloadTerrainExt = this.ext::<DownloadTerrainExt>();
        ext.m_updateCounter.set(ext.m_updateCounter.get() + 1);
        if ext.m_updateCounter.get() % 20 == 0
        {
            ext.m_netHandler.addToSendQueue(Box::new(Packet0KeepAlive::default()));
        }
        ext.m_netHandler.processReadPackets();
    },
    actionPerformed: |_, _| {},
    drawScreen: |this, mouseX, mouseY, tickDelta| {
        this.drawBackground(0);
        let stringTranslate: Rc<StringTranslate> = StringTranslate::getInstance();
        this.m_gui.drawCenteredString(
            &this.fontRenderer(),
            &stringTranslate.translateKey("multiplayer.downloadingTerrain"),
            this.m_width.get() / 2,
            this.m_height.get() / 2 - 50,
            0xFFFFFF,
        );
        (GUI_SCREEN.drawScreen)(this, mouseX, mouseY, tickDelta);
    },
    ..GUI_SCREEN
};

pub struct GuiDownloadTerrain;

impl GuiDownloadTerrain
{
    pub fn new(networkHandler: &Rc<NetClientHandler>) -> Rc<GuiScreen>
    {
        let ext: DownloadTerrainExt =
            DownloadTerrainExt { m_netHandler: Rc::clone(networkHandler), m_updateCounter: Cell::new(0) };
        GuiScreen::construct(&CLASS, Box::new(ext))
    }
}
