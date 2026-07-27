use std::ptr::null;

use iced::{keyboard::key::Code::Comma, wgpu::wgc::command, widget::text::base};

pub struct CommandOfPacketManager {
    basic_command: String,
    install_command: String,
    remove_command: String,
    search_command: String,
    check_update_command: String,
    update_command: String,
    add_repo_command: String,
    remove_repo_command: String
}

impl CommandOfPacketManager {
    pub fn new_empty(base_command_name: String) -> CommandOfPacketManager {
        let command_obj = CommandOfPacketManager{
            basic_command: base_command_name,
            install_command: String::new(),
            remove_command: String::new(),
            search_command: String::new(),
            check_update_command: String::new(),
            update_command: String::new(),
            add_repo_command: String::new(),
            remove_repo_command: String::new()
        };

        return command_obj;
    }
}

fn get_packet_manager_preset(base_command_name: String) {
    let mut command_obj: CommandOfPacketManager = CommandOfPacketManager::new_empty(base_command_name.clone());
    for name in ["zypper", "dnf"] {
        let name_obj: String = String::from(name);

        if base_command_name == name_obj {
            command_obj.install_command.insert_str(0, "install");
            command_obj.remove_command.insert_str(0, "remove");
            command_obj.search_command.insert_str(0, "search");
            break;
        }
    }
}

mod utility {
    pub enum PacketManagerResultCode {
        Success,
        Error(String, Vec<String>)
    }
}

mod packet_manager_trait {
    use super::utility::PacketManagerResultCode;

    pub trait PacketManager {
        fn install(self, packets: Vec<String>) -> PacketManagerResultCode;
        fn remove(self, packets: Vec<String>) -> PacketManagerResultCode;

        fn update(self) -> PacketManagerResultCode;
        fn show_updates(self) -> Vec<String>;
        fn update_system(self) -> PacketManagerResultCode;

        fn repos(self) -> Vec<String>;
        fn add_repo(self, repo_name: &str, repo_url: &str);
        fn remove_repo(self, repo_name: &str);

        fn search(self, packets: Vec<String>) -> Vec<String>;
        fn applications_list(self) -> Vec<String>;
    }
}