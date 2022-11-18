use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::Config;
use log4rs::config::{Appender, Root};
use log4rs::encode::pattern::PatternEncoder;
use log::LevelFilter;
use crate::CONFIGURATION;

pub(crate) fn log_config(level: LevelFilter) -> Config {
    Config::builder()
//        .appender(Appender::builder()
//            .build("stdout", Box::new(stdout_appender()))
//        )
        .appender(Appender::builder()
            .build("file", Box::new(file_appender())))
        .build(Root::builder()
            .appender("file")
//            .appender("stdout")
            .build(level)
        )
        .unwrap()
}

#[allow(dead_code)]
fn stdout_appender() -> ConsoleAppender {
    ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{m}{n}")))
        .build()
}

fn file_appender() -> FileAppender {
    FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} {l:<5} - {m}{n}")))
        .build(CONFIGURATION.log_path.as_path())
        .unwrap()
}