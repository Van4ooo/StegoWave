use chrono::Local;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::time::FormatTime;

struct CustomTime;

impl FormatTime for CustomTime {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        let now = Local::now();
        write!(w, "{}", now.format("%H:%M:%S"))
    }
}

#[inline]
pub fn conf_logger() {
    tracing_subscriber::fmt()
        .with_timer(CustomTime)
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}
