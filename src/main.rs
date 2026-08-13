mod base_logic;
use base_logic::AppBaseLogic;

use regex::Regex;
use package_manager_automatic::utility::{FoundPackage, InstallFlag};

use log::info;

use clap::{Parser, ArgAction};
use std::fmt::Debug;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct AutomatonArgs {
    #[arg(short, long, default_value_t = "zypper".to_string())]
    name: String,

    #[arg(long, action = ArgAction::Append)]
    ip_to_send: Vec<String>,

    #[arg(long, default_value_t = 2020)]
    port: u32,

    #[arg(short, long, default_value_t = false)]
    is_recieve: bool,
}
fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
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
    info!("Start parsing...");

    let auto_args = AutomatonArgs::parse();
    let mut standard_logic = AppBaseLogic::new(auto_args.name);

    if auto_args.is_recieve {
        let repository_list = standard_logic.recieve_repository_list(auto_args.port);

        let packages_list = standard_logic.recieve_packages_list(auto_args.port);

        if let Some(repo_list) = repository_list{
            for repo in repo_list {
                if repo.enabled_status {
                    info!("Repository: {}. Alias: {}", repo.uri, repo.alias);
                    // standard_logic.add_repo(repo.uri, repo.alias);
                }
            }
        }

        if let Some(pack_list) = packages_list {
            let found_packages: Vec<String> = pack_list.iter()
                .filter(|s| s.install_flag == InstallFlag::NoInstalled)
                .map(|s| s.name.clone())
                .collect();

            for package in found_packages {
                info!("Package: {}", package);
            }
            // standard_logic.install_packages(found_packages);
        }
    }
    else {
        let pattern_of_ip = Regex::new(r#"\d{1,3}(\.\d{1,3}){3}"#).unwrap();
        let mut ips_to_send: Vec<String> = Vec::new();

        for ip_list in auto_args.ip_to_send.clone() {
            if ip_list.contains(',') {
                let ip_filtered = ip_list
                    .split(',')
                    .map(|l| l.trim().to_string())
                    .filter(|s| s.len() > 0 && pattern_of_ip.is_match(s));

                ips_to_send.extend(ip_filtered);
            }
            else {
                ips_to_send.push(ip_list.trim().to_string());
            }
        }

        standard_logic.send_repository_list(auto_args.ip_to_send.clone(), auto_args.port);
        standard_logic.send_packages_list(auto_args.ip_to_send.clone(), auto_args.port);
    }

}
