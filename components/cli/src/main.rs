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
        report_error("Incorrect CLI arg", 2);
    };

    // check if file extension is ".csv"
    if !input_csv.ends_with(".csv") {
        log::info!("Incorrect file extension. Extension must be \".csv\"");
    }

    let file = std::fs::File::open(input_csv)?;
    engine::process_transactions(file, &mut std::io::stdout())?;

    Ok(())
}

fn print_help() {
    println!("Usage:\n   cargo run -- <input.csv> > <output.csv>");
}

fn report_error(msg: &str, error_code: i32) -> ! {
    eprintln!("{}: {}", Style::new().red().bold().apply_to("ERR"), msg);
    std::process::exit(error_code);
}
