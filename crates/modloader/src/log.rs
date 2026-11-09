use jvm::logging::logger;

pub const LOADER: &str = "loader";

// every line names the mod it came from so a modpack log stays readable
pub fn debug(modId: &str, message: &str) { logger().debug(&format!("[{modId}] {message}")); }

pub fn info(modId: &str, message: &str) { logger().info(&format!("[{modId}] {message}")); }

pub fn warn(modId: &str, message: &str) { logger().warning(&format!("[{modId}] {message}")); }

pub fn error(modId: &str, message: &str) { logger().severe(&format!("[{modId}] {message}")); }
