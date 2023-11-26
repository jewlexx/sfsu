use std::{io::Write, path::PathBuf};

pub fn handle_panics() {
    human_panic::setup_panic!(Metadata {
        name: env!("CARGO_PKG_NAME").into(),
        version: env!("CARGO_PKG_VERSION").into(),
        authors: "Juliette Cordor <support+sfsu@maybejules.com>".into(),
        homepage: "github.com/jewelexx/sfsu".into(),
    });
}

pub fn initialize_logging(max_level: log::LevelFilter) -> std::io::Result<PathBuf> {
    use std::{fs::File, path::Path, sync::Mutex};

    use uuid::Uuid;

    let uuid = Uuid::now_v7();
    let path = std::env::temp_dir().join(format!("sfsu-log-{uuid}.txt"));

    use log::{Metadata, Record};

    struct FileLogger {
        fp: Mutex<File>,
    }

    impl FileLogger {
        pub fn new(path: impl AsRef<Path>) -> std::io::Result<Self> {
            let fp = File::create(path)?;

            Ok(Self { fp: Mutex::new(fp) })
        }
    }

    impl log::Log for FileLogger {
        fn enabled(&self, _metadata: &Metadata) -> bool {
            true
        }

        fn log(&self, record: &Record) {
            if self.enabled(record.metadata()) {
                writeln!(
                    &mut self.fp.lock().unwrap(),
                    "{}@{}:{} - {}",
                    record.level(),
                    record.file().unwrap(),
                    record.line().unwrap(),
                    record.args()
                )
                .expect("writing to log to succeed");
            }
        }

        fn flush(&self) {
            self.fp.lock().unwrap().flush().expect("flush file")
        }
    }

    let writer = FileLogger::new(&path)?;

    log::set_boxed_logger(Box::new(writer)).expect("set logger");
    log::set_max_level(max_level);

    Ok(path)
}
