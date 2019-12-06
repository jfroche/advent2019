extern crate env_logger;

pub fn init_logging() {
    env_logger::builder().format_timestamp_millis().init();
}
