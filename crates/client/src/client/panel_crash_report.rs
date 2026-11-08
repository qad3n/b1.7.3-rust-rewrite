use crate::client::UnexpectedThrowable;
use crate::gui::calendar::formatDefault;
use jvm::string::JString;
use jvm::throwable::{THROWABLE, Throwable, catchThrowable};
use lwjgl::Sys;
use lwjgl::opengl::GL11;

const JAVA_VERSION: &str = "1.6.0_26";
const JAVA_VENDOR: &str = "Sun Microsystems Inc.";
const JAVA_VM_NAME: &str = "Java HotSpot(TM) 64-Bit Server VM";
const JAVA_VM_INFO: &str = "mixed mode";

fn osName() -> String
{
    match std::env::consts::OS
    {
        "linux" => "Linux".to_owned(),
        "windows" => "Windows".to_owned(),
        "macos" => "Mac OS X".to_owned(),
        other => other.to_owned(),
    }
}

fn osArch() -> String
{
    match std::env::consts::ARCH
    {
        "x86_64" => "amd64".to_owned(),
        "x86" => "x86".to_owned(),
        other => other.to_owned(),
    }
}

fn osVersion() -> String
{
    std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .map_or_else(|_| "null".to_owned(), |text| text.trim().to_owned())
}

fn glString(name: i32) -> String { GL11::glGetString(name).unwrap_or_else(|| "null".to_owned()) }

fn hexHash(text: &str) -> String { format!("{:x}", JString::from(text).hashCode() as u32) }

pub struct PanelCrashReport;

impl PanelCrashReport
{
    pub fn new(report: &UnexpectedThrowable) -> PanelCrashReport
    {
        // the awt crash panel is not ported so its text goes to stderr
        eprint!("{}", PanelCrashReport::text(report));
        PanelCrashReport
    }

    fn text(report: &UnexpectedThrowable) -> String
    {
        let string: String = format!("{}\n", report.m_exception);
        let mut string2: String = String::new();
        let mut string3: String = String::new();
        let properties: Result<(), Throwable> = catchThrowable(&THROWABLE, || {
            string3 += &format!("Generated {}\n", formatDefault(jvm::system::currentTimeMillis()));
            string3 += "\n";
            string3 += "Minecraft: Minecraft Beta 1.7.3\n";
            string3 += &format!("OS: {} ({}) version {}\n", osName(), osArch(), osVersion());
            string3 += &format!("Java: {JAVA_VERSION}, {JAVA_VENDOR}\n");
            string3 += &format!("VM: {JAVA_VM_NAME} ({JAVA_VM_INFO}), {JAVA_VENDOR}\n");
            string3 += &format!("LWJGL: {}\n", Sys::getVersion());
            string2 = glString(7936);
            string3 += &format!("OpenGL: {} version {}, {}\n", glString(7937), glString(7938), glString(7936));
        });
        if let Err(throwable) = properties
        {
            string3 += &format!("[failed to get system properties ({throwable})]\n");
        }
        string3 += "\n";
        string3 += &string;
        let mut string4: String = String::from("\n\n");
        if string.contains("Pixel format not accelerated")
        {
            string4 += "      Bad video card drivers!      \n";
            string4 += "      -----------------------      \n";
            string4 += "\n";
            string4 += "Minecraft was unable to start because it failed to find an accelerated OpenGL mode.\n";
            string4 += "This can usually be fixed by updating the video card drivers.\n";
            if JString::from(string2.as_str()).toLowerCase().to_string().contains("nvidia")
            {
                string4 += "\n";
                string4 += "You might be able to find drivers for your video card here:\n";
                string4 += "  http://www.nvidia.com/\n";
            }
            else if JString::from(string2.as_str()).toLowerCase().to_string().contains("ati")
            {
                string4 += "\n";
                string4 += "You might be able to find drivers for your video card here:\n";
                string4 += "  http://www.amd.com/\n";
            }
        }
        else
        {
            string4 += "      Minecraft has crashed!      \n";
            string4 += "      ----------------------      \n";
            string4 += "\n";
            string4 += "Minecraft has stopped running because it encountered a problem.\n";
            string4 += "\n";
            string4 += "If you wish to report this, please copy this entire text and email it to support@mojang.com.\n";
            string4 += "Please include a description of what you did when the error occured.\n";
        }
        string4 += "\n\n\n";
        string4 += &format!("--- BEGIN ERROR REPORT {} --------\n", hexHash(&string4));
        string4 += &string3;
        string4 += &format!("--- END ERROR REPORT {} ----------\n", hexHash(&string4));
        string4 += "\n\n";
        string4
    }
}
