pub struct Args {
    pub rom_path: std::path::PathBuf,
}

pub fn parse_args() -> Result<Args, lexopt::Error> {
    use lexopt::prelude::*;

    let mut rom_path = None;
    let mut parser = lexopt::Parser::from_env();

    while let Some(arg) = parser.next()? {
        match arg {
            Value(path) => {
                assert!(rom_path.is_none());
                rom_path = Some(path.parse()?);
            }
            Long("help") => {
                println!("Usage: gbemu ROM_PATH");
                std::process::exit(0);
            }
            _ => return Err(arg.unexpected()),
        }
    }

    Ok(Args {
        rom_path: rom_path.ok_or("missing argument ROM_PATH")?,
    })
}
