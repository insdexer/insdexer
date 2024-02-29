use log::Level;
use pretty_env_logger::env_logger::fmt::{Color, Style, StyledValue};

fn colored_level<'a>(style: &'a mut Style, level: Level) -> StyledValue<'a, &'static str> {
    match level {
        Level::Trace => style.set_color(Color::Magenta).value("TRACE"),
        Level::Debug => style.set_color(Color::Blue).value("DEBUG"),
        Level::Info => style.set_color(Color::Green).value("INFO "),
        Level::Warn => style.set_color(Color::Yellow).value("WARN "),
        Level::Error => style.set_color(Color::Red).value("ERROR"),
    }
}

pub fn init_log() {
    let mut builder = pretty_env_logger::formatted_timed_builder();

    if let Ok(s) = ::std::env::var("RUST_LOG") {
        builder.parse_filters(&s);
    }

    builder.format(|f, record| {
        use std::io::Write;
        let mut style = f.style();
        let level = colored_level(&mut style, record.level());
        let time = f.timestamp_millis();
        writeln!(f, "{} {} {}", time, level, record.args())
    });

    builder.init();
}
