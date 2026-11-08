pub fn runtimeMemory() -> (i64, i64, i64)
{
    let field = |path: &str, key: &str| -> i64 {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|text| {
                text.lines()
                    .find(|line| line.starts_with(key))
                    .and_then(|line| line.split_whitespace().nth(1))
                    .and_then(|value| value.parse::<i64>().ok())
            })
            .unwrap_or(0)
            * 1024
    };
    let total: i64 = field("/proc/self/status", "VmRSS:");
    let max: i64 = (field("/proc/meminfo", "MemTotal:") / 4).max(total).max(1);
    (max, total, 0)
}
