use crate::base_logic::AppBaseLogic;
use package_manager_automatic::utility::InstallFlag;

use regex::Regex;

use log::{debug, info, warn};

pub fn recieve_packages_and_repos(standard_logic: &mut AppBaseLogic, port: u32, is_install_repos: bool, is_install_packages: bool) {
    let repository_list: Option<Vec<package_manager_automatic::utility::Repository>> = standard_logic.recieve_repository_list(port);
    let packages_list: Option<Vec<package_manager_automatic::utility::FoundPackage>> = standard_logic.recieve_packages_list(port);

    info!("Recieving on port {}. Is install repos: {}. Is install packages: {}", port, is_install_repos, is_install_packages);
    if let Some(repo_list) = repository_list{
        for repo in repo_list {
            if repo.enabled_status {
                info!("Repository: {}. Alias: {}", repo.uri, repo.alias);

                if is_install_repos {
                    standard_logic.add_repo(repo.uri, repo.alias);
                }
            }
        }
    }

    if let Some(pack_list) = packages_list {
        let found_packages: Vec<String> = pack_list.iter()
            .filter(|s| s.install_flag == InstallFlag::NoInstalled)
            .map(|s| s.name.clone())
            .collect();

        debug!("Getted packages: {}", if found_packages.len() > 0 { "None" } else { "" });
        for package in found_packages.iter() {
            debug!("\t- {}", package);
        }

        if is_install_packages {
            standard_logic.install_packages(found_packages);
        }
    }
}

pub fn send_packages_and_repos(standard_logic: &mut AppBaseLogic, ip_to_send: &Vec<String>, port: u32) {
    let pattern_of_ip = Regex::new(r#"\d{1,3}(\.\d{1,3}){3}"#).unwrap();
    let mut ips_to_send: Vec<String> = Vec::new();

    for ip_list in ip_to_send.iter() {
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

    debug!("Sending info about packages to ips: {}", if ips_to_send.len() > 0 { "" } else {"None"});

    if !ips_to_send.is_empty() {
        for ip in ips_to_send.iter() {
            debug!("\t{}", ip);
        }

        standard_logic.send_repository_list(ip_to_send.clone(), port);
        standard_logic.send_packages_list(ip_to_send.clone(), port);
    }
    else {
        warn!("No ip addreses to send is added...");
    }
}
