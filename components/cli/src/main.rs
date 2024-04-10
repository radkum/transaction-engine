use std::ffi::OsString;

use console::Style;

fn main() -> anyhow::Result<()> {
    let level_filter = log::LevelFilter::Error;
    env_logger::Builder::new().filter_level(level_filter).init();

    let args = std::env::args_os().collect::<Vec<OsString>>();

    if args.len() != 2 {
        print_help();
        return Ok(());
    }

    let Some(input_csv) = args[1].to_str() else {
        eprintln!("{}: incorrect CLI arg", Style::new().red().bold().apply_to("ERR"),);
        std::process::exit(2);
    };

    if let Err(e) = engine::process_file(input_csv, &mut std::io::stdout()) {
        eprintln!("{}: {:?}", Style::new().red().bold().apply_to("ERR"), e);
        std::process::exit(1);
    };
    Ok(())
}

fn print_help() {
    println!("Usage:\n   cargo run -- <input.csv> > <output.csv>");
}
