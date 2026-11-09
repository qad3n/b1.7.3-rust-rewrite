use jvm::logging::logger;

pub const LOADER: &str = "loader";

// every line names the mod it came from so a modpack log stays readable
#[track_caller]
pub fn debug(modId: &str, message: &str) { logger().debug(&format!("[{modId}] {message}")); }

#[track_caller]
pub fn info(modId: &str, message: &str) { logger().info(&format!("[{modId}] {message}")); }

#[track_caller]
pub fn warn(modId: &str, message: &str) { logger().warning(&format!("[{modId}] {message}")); }

#[track_caller]
pub fn error(modId: &str, message: &str) { logger().severe(&format!("[{modId}] {message}")); }
