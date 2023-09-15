use anyhow::Context;

pub(crate) fn init_logger() -> anyhow::Result<()> {
    let mut config = simplelog::ConfigBuilder::default();

    config
        .set_target_level(simplelog::LevelFilter::Off)
        .set_location_level(simplelog::LevelFilter::Off)
        .set_time_level(simplelog::LevelFilter::Error)
        .set_time_format_rfc3339();

    config.set_time_offset_to_local().unwrap();

    simplelog::TermLogger::init(
        simplelog::LevelFilter::Info,
        config.build(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    )
    .context("couldn't init term logger")
}
