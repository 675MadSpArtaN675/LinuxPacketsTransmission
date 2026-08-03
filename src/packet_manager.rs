pub mod utility {
    pub enum PacketManagerResultCode {
        Success,
        SuccessCollection(Vec<String>),
        Error(String, i32, Vec<String>)
    }
}

pub mod packet_manager_trait {
    use super::utility::PacketManagerResultCode;

    pub trait PacketManager {
        fn install(&mut self, packets: &Vec<String>) -> PacketManagerResultCode;
        fn remove(&mut self, packets: &Vec<String>) -> PacketManagerResultCode;

        fn update(&mut self) -> PacketManagerResultCode;
        fn show_updates(&self) -> Vec<String>;
        fn update_system(&mut self) -> PacketManagerResultCode;

        fn repos(&self) -> Vec<String>;
        fn add_repo(&self, repo_name: &str, repo_url: &str);
        fn remove_repo(&self, repo_name: &str);

        fn search(&self, packets: Vec<String>) -> Vec<String>;
        fn applications_list(&self) -> Vec<String>;
    }
}

use packet_manager_trait::PacketManager;
use std::process::{Child, ChildStderr, ChildStdout, Command, ExitStatus, Stdio};
use std::io::{BufReader, BufRead};
use regex::{Regex, regex};
use std::fmt::{Display, Error};

use crate::packet_manager::utility::PacketManagerResultCode;

pub struct PacketManagerCommand {
    basic_command: String,
    install_command: String,
    remove_command: String,
    search_command: String,
    check_update_command: String,
    update_command: String,
    system_update_command: String,
}

impl PacketManagerCommand {
    pub fn new_empty(base_command_name: String) -> PacketManagerCommand {
        let command_obj: PacketManagerCommand = PacketManagerCommand{
            basic_command: base_command_name,
            install_command: String::new(),
            remove_command: String::new(),
            search_command: String::new(),
            check_update_command: String::new(),
            update_command: String::new(),
            system_update_command: String::new()
        };

        return command_obj;
    }

    pub fn new(base_command_name: String) -> PacketManagerCommand {
        return get_packet_manager_preset(base_command_name);
    }
}

fn get_packet_manager_preset(base_command_name: String) -> PacketManagerCommand {
    let mut command_obj: PacketManagerCommand = PacketManagerCommand::new_empty(base_command_name.clone());

    match base_command_name.as_str() {
        "zypper" => {
            create_standard_commands(&mut command_obj);
            command_obj.check_update_command = String::from("refresh");
            command_obj.check_update_command = String::from("dist-upgrade");
        },
        "dnf" => {
            create_standard_commands(&mut command_obj);
            command_obj.update_command = String::from("upgrade");
            command_obj.check_update_command = String::from("check");
        },
        "apt" => {
            create_standard_commands(&mut command_obj);
        }

        _ => {},
    };

    return command_obj;
}

fn create_standard_commands(command_obj: &mut PacketManagerCommand) {
    command_obj.install_command = String::from("install");
    command_obj.remove_command = String::from("remove");
    command_obj.search_command = String::from("search");
    command_obj.update_command = String::from("update");
    command_obj.check_update_command = String::from("upgrade");
}



#[derive(Clone, Copy)]
pub enum Stage {
    Install,
    Remove,
    Update,
    Showing
}

impl Stage {
    pub fn to_string(&self) -> &str {
        let result: &str = match &self {
            Stage::Install => "Install",
            Stage::Update => "Update",
            Stage::Remove => "Remove",
            Stage::Showing => "Showing"
        };

        return result;
    }

    pub fn to_string_obj(&self) -> String {
        return String::from(self.to_string());
    }
}

impl Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub struct ErroredPacket {
    pub name: String,
    pub stage: Stage
}

fn perform_command(basic_command: String, secondary_subcommand: String, stage: Stage, packets: &Vec<String>) -> PacketManagerResultCode
{
    let full_command: String = "sudo".to_string();

    let mut _install_command: Command = Command::new(full_command.clone());

    _install_command.arg(format!("{}", basic_command))
                    .arg(secondary_subcommand.as_str())
                    .arg("-y")
                    .args(packets)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());

    let install_process_result = _install_command.spawn();

    if install_process_result.is_err() {
        let error_message = format!("Error: {:?}", install_process_result.unwrap_err());
        return PacketManagerResultCode::Error(error_message, -1, packets.clone())
    }

    let not_found_pattern: Regex = regex!(r#".*"(?<packet_name>[\w\d\s]+)".*((not found)|(не найден))"#).clone();
    let mut install_process: Child = install_process_result.unwrap();

    let _out_stream: Option<ChildStdout> = install_process.stdout.take();
    let _err_stream: Option<ChildStderr> = install_process.stderr.take();


    let _status_code: Result<ExitStatus, std::io::Error> = install_process.wait();
    if _status_code.is_ok() {
        let status_code = _status_code.unwrap();
        let mut errored_packages: Vec<String> = Vec::new();

        if _err_stream.is_some() {
            let _buf_reader: BufReader<ChildStderr> = BufReader::new(_err_stream.unwrap());

            for line in _buf_reader.lines() {
                if line.is_ok() {
                    let line_ref = line.as_ref().unwrap();
                    for packet_error_line in not_found_pattern.captures_iter(line_ref.as_ref()) {
                        let package_name_result = packet_error_line.name("packet_name");

                        if package_name_result.is_some() {
                            let package_name = String::from(package_name_result.unwrap().as_str());

                            errored_packages.push(package_name);
                        }
                    }
                }
            }
        }

        if status_code.success() {
            return PacketManagerResultCode::Success;
        }
        else {
            return PacketManagerResultCode::Error(format!("Error in stage: {}", stage), status_code.code().unwrap(), errored_packages);
        }
    }

    return PacketManagerResultCode::Error(format!("Error of process start. Stage: {}",  stage), -1, vec![])
}

fn catch_err(command_return: PacketManagerResultCode, stage: Stage, errored_packages: &mut Vec<ErroredPacket>) -> PacketManagerResultCode {
    return match command_return {
        PacketManagerResultCode::Error(message, status_code, packets) => {
            for packet in packets.clone() {
                errored_packages.push(ErroredPacket { name: packet, stage: stage });
            }

            return PacketManagerResultCode::Error(message, status_code, packets);
        },
        PacketManagerResultCode::SuccessCollection(val) => PacketManagerResultCode::SuccessCollection(val),

        PacketManagerResultCode::Success => command_return
    };
}

pub struct PacketManagerCommandExecutor
{
    pub command_obj: PacketManagerCommand,
    pub errored_packages: Vec<ErroredPacket>
}

impl PacketManager for PacketManagerCommandExecutor {
    fn install(&mut self, packets: &Vec<String>) -> PacketManagerResultCode {
        println!("Installing packages...");
        let command_return: PacketManagerResultCode = perform_command(self.command_obj.basic_command.clone(), self.command_obj.install_command.clone(), Stage::Install, packets);

        return catch_err(command_return, Stage::Install, &mut self.errored_packages);
    }

    fn remove(&mut self, packets: &Vec<String>) -> PacketManagerResultCode {
        let command_return: PacketManagerResultCode = perform_command(self.command_obj.basic_command.clone(), self.command_obj.remove_command.clone(), Stage::Remove, packets);

        return catch_err(command_return, Stage::Remove, &mut self.errored_packages);
    }

    fn update(&mut self) -> PacketManagerResultCode {
        let command_return: PacketManagerResultCode = perform_command(self.command_obj.basic_command.clone(), self.command_obj.update_command.clone(), Stage::Update, &Vec::new());

        return catch_err(command_return, Stage::Update, &mut self.errored_packages);
    }

    fn show_updates(&self) -> Vec<String> {
        let command_return: PacketManagerResultCode = perform_command(self.command_obj.basic_command.clone(), self.command_obj.update_command.clone(), Stage::Update, &Vec::new());

        let mut errored_packets: Vec<ErroredPacket> = vec![];
        let status_code = catch_err(command_return, Stage::Showing, &mut errored_packets);

        return ;
    }

    fn update_system(&mut self) -> PacketManagerResultCode {
        let command_return: PacketManagerResultCode = perform_command(self.command_obj.basic_command.clone(), self.command_obj.system_update_command.clone(), Stage::Remove, &vec![]);

        return catch_err(command_return, Stage::Update, &mut self.errored_packages);
    }

    fn repos(&self) -> Vec<String> {
        return vec![];
    }

    fn add_repo(&self, repo_name: &str, repo_url: &str) {
    }

    fn remove_repo(&self, repo_name: &str) {

    }

    fn search(&self, packets: Vec<String>) -> Vec<String> {
        return vec![];
    }

    fn applications_list(&self) -> Vec<String> {
        return vec![];
    }
}