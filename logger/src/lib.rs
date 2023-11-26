pub fn handle_panics() {
    human_panic::setup_panic!(Metadata {
        name: env!("CARGO_PKG_NAME").into(),
        version: env!("CARGO_PKG_VERSION").into(),
        authors: "Juliette Cordor <support+sfsu@maybejules.com>".into(),
        homepage: "github.com/jewelexx/sfsu".into(),
    });
}

pub fn initialize_logging() {
    use std::fs::File;

    struct LogWriter {
        fp: File,
    }

    impl LogWriter {
        pub fn new() -> Self {
            todo!()
        }
    }

    impl std::io::Write for LogWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            todo!()
        }

        fn flush(&mut self) -> std::io::Result<()> {
            todo!()
        }
    }
}
