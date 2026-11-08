use jvm::throwable::{ERROR, RUNTIME, ThrowableClass};

pub static MINECRAFT_ERROR: ThrowableClass =
    ThrowableClass { m_name: "net.minecraft.src.MinecraftError", m_parent: Some(&ERROR) };

pub static MINECRAFT_EXCEPTION: ThrowableClass =
    ThrowableClass { m_name: "net.minecraft.src.MinecraftException", m_parent: Some(&RUNTIME) };
