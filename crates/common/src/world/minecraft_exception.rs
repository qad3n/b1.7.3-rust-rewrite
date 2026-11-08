use jvm::throwable::{RUNTIME, ThrowableClass};

pub static MINECRAFT_EXCEPTION: ThrowableClass =
    ThrowableClass { m_name: "net.minecraft.src.MinecraftException", m_parent: Some(&RUNTIME) };
