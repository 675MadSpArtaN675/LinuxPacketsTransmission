pub mod utility {
    pub enum PacketManagerResultCode {
        Success,
        Error(String, Vec<String>)
    }
}

mod packet_manager_trait {
    use super::utility::PacketManagerResultCode;

    pub trait PacketManager {
        fn install(&self, packets: &Vec<String>) -> PacketManagerResultCode;
        fn remove(&self, packets: &Vec<String>) -> PacketManagerResultCode;

        fn update(&self) -> PacketManagerResultCode;
        fn show_updates(&self) -> Vec<String>;
        fn update_system(&self) -> PacketManagerResultCode;

        fn repos(&self) -> Vec<String>;
        fn add_repo(&self, repo_name: &str, repo_url: &str);
        fn remove_repo(&self, repo_name: &str);

        fn search(&self, packets: Vec<String>) -> Vec<String>;
        fn applications_list(&self) -> Vec<String>;
    }
}

use packet_manager_trait::PacketManager;
use std::process::{Command, Child, Stdio};

use crate::packet_manager::utility::PacketManagerResultCode;

pub struct PacketManagerCommand {
    basic_command: String,
    install_command: String,
    remove_command: String,
    search_command: String,
    check_update_command: String,
    update_command: String,
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


pub struct PacketManagerCommandExecutor
{
    command_obj: PacketManagerCommand

}

impl PacketManager for PacketManagerCommandExecutor {
    fn install(&self, packets: &Vec<String>) -> PacketManagerResultCode {
        let full_command: String = format!("{} {}", self.command_obj.basic_command, self.command_obj.install_command);

        let mut _install_command: Command = Command::new(full_command);
        _install_command.args(packets);
        let mut install_process: Child = _install_command.spawn().expect("Error of installer launch...");

        let _out_stream = install_process.stdout.take();
        let _err_stream = install_process.stderr.take();

        let _status_code = install_process.wait();
        if _status_code.is_ok() {
            if _out_stream.is_some() {

            }

            if _err_stream.is_some() {

            }

            return PacketManagerResultCode::Success;
        }

        return PacketManagerResultCode::Error(String::from("Error of process start"), vec![])

    }

    fn remove(&self, packets: &Vec<String>) -> PacketManagerResultCode {
    }

    fn update(self) -> PacketManagerResultCode {

    }

    fn show_updates(self) -> Vec<String> {

    }

    fn update_system(self) -> PacketManagerResultCode {

    }

    fn repos(self) -> Vec<String> {

    }

    fn add_repo(self, repo_name: &str, repo_url: &str) {

    }

    fn remove_repo(self, repo_name: &str) {

    }

    fn search(self, packets: Vec<String>) -> Vec<String> {

    }

    fn applications_list(self) -> Vec<String> {

    }
}