mod command_line_params;
mod package_b_a_w_list;
mod packages_filter;
mod file_saver;
mod base_logic;
mod logic;

use base_logic::AppBaseLogic;
use command_line_params::*;
use logic::*;

use log::info;


fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}-'{}']: {}",
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

fn main() {
    if let Err(error_log) = setup_logger() {
        println!("Logger error: {}", error_log);
    }

    let auto_args = AutomatonArgs::parse();
    info!("Start parsing...");
    match auto_args.subcommands {
        RecieveCommand::Recieve(args) => {
            let mut standard_logic = AppBaseLogic::new(args.name, args.path_to_save);

            recieve_packages_and_repos(
                &mut standard_logic,
                args.port,
                !args.is_no_install_repositories,
                !args.is_no_install_packages
            );
        },
        RecieveCommand::Send(args) => {
            let mut standard_logic = AppBaseLogic::new(args.name, args.path_to_save);

            send_packages_and_repos(&mut standard_logic, &args.ip_to_send, args.port);
        },

        _ => {}
    }
}
