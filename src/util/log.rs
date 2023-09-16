use anyhow::Context;
use simplelog::{
    format_description, ColorChoice, ConfigBuilder, LevelFilter, TermLogger, TerminalMode,
};

pub(crate) fn init_logger() -> anyhow::Result<()> {
    let mut config = ConfigBuilder::default();

    config
        .set_target_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Off)
        .set_time_level(LevelFilter::Error)
        .set_time_format_custom(format_description!(
            version = 2,
            "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3][offset_hour sign:mandatory][offset_minute]"
        ));

    config.set_time_offset_to_local().unwrap();

    TermLogger::init(
        LevelFilter::Info,
        config.build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .context("couldn't init term logger")
}
