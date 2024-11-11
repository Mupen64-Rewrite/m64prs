use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
    fmt::Arguments,
};

use glib::{LogField, LogLevel as GLogLevel, LogWriterOutput};
use log::kv;

pub fn retarget_glib_logs() {
    glib::log_set_default_handler(log_handler);
    glib::log_set_writer_func(log_writer);
}

fn log_handler(domain: Option<&str>, g_level: GLogLevel, message: &str) {
    let level = match g_level {
        GLogLevel::Error | GLogLevel::Critical => log::Level::Error,
        GLogLevel::Warning => log::Level::Warn,
        GLogLevel::Message | GLogLevel::Info => log::Level::Info,
        GLogLevel::Debug => log::Level::Debug,
    };
    let prefix = match g_level {
        GLogLevel::Error => "(ERROR) ",
        GLogLevel::Critical => "(CRITICAL) ",
        _ => "",
    };
    log::log!(target: domain.unwrap_or("<null>"), level, "{}{}", prefix, message);
}

fn log_writer(level: GLogLevel, fields: &[LogField<'_>]) -> LogWriterOutput {
    const NULL_MSG: &str = "<null>";
    const DEFAULT_LEVEL: log::Level = log::Level::Warn;

    let mut message: Option<String> = None;
    let mut target: Option<String> = None;
    let mut level: Option<log::Level> = Some(to_rust_level(level));
    let mut kv_source = HashMap::<String, String>::new();

    for field in fields {
        match field.key() {
            "MESSAGE" => {
                message = field.value_str().map(str::to_owned);
            }
            "GLIB_DOMAIN" => {
                target = field.value_str().map(str::to_owned);
            }
            "PRIORITY" => {
                level = match field.value_str().and_then(|x| x.parse::<u32>().ok()) {
                    Some(0..=3) => Some(log::Level::Error),
                    Some(4) => Some(log::Level::Warn),
                    Some(5..=6) => Some(log::Level::Info),
                    Some(7) => Some(log::Level::Debug),
                    _ => None,
                }
            }
            key => {
                kv_source.insert(
                    format!("gtk_{}", key.to_lowercase()),
                    field.value_str().unwrap_or("").to_owned(),
                );
            }
        }
    }

    let message = message.unwrap_or(NULL_MSG.to_owned());
    let target = target.unwrap_or(NULL_MSG.to_owned());
    let level = level.unwrap_or(DEFAULT_LEVEL);

    do_logging(format_args!("{}", message), &target, level, &kv_source);

    LogWriterOutput::Handled
}

fn to_rust_level(g_level: GLogLevel) -> log::Level {
    match g_level {
        GLogLevel::Error | GLogLevel::Critical => log::Level::Error,
        GLogLevel::Warning => log::Level::Warn,
        GLogLevel::Message | GLogLevel::Info => log::Level::Info,
        GLogLevel::Debug => log::Level::Debug,
    }
}

fn do_logging(args: Arguments, target: &str, level: log::Level, kv_source: &dyn kv::Source) {
    let mut builder = log::Record::builder();
    let record = builder
        .args(args)
        .target(target)
        .level(level)
        .key_values(kv_source)
        .build();

    log::logger().log(&record);
}
