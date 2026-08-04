pub mod utility {
    use std::fmt::Display;

    pub enum PacketManagerResultCode {
        Success(Option<Vec<String>>),
        Error(String, i32, Vec<String>)
    }

    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
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
}

pub mod command_basic_structure {
    pub struct PacketManagerCommand {
        pub basic_command: String,
        pub install_command: String,
        pub remove_command: String,
        pub search_command: String,
        pub check_update_command: String,
        pub update_command: String,
        pub system_update_command: String,
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
}

mod command_performers {
    use super::utility::{Stage, PacketManagerResultCode, ErroredPacket};

    use std::io::{BufReader, BufRead, Read};
    use std::process::{Child, ChildStderr, ChildStdout, Command, ExitStatus, Stdio};

    pub fn perform_command(basic_command: String,
                    secondary_subcommand: String,
                    stage: Stage,
                    packets: &Vec<String>,
                    error_parser: Option<&mut Box<dyn FnMut(&mut String)>>,
                    output_parser: Option<&mut Box<dyn FnMut(&mut String)>>
    ) -> PacketManagerResultCode
    {
        let full_command: String = "sudo".to_string();

        let mut _install_command: Command = Command::new(full_command.clone());

        _install_command.arg(format!("{}", basic_command))
                        .arg(secondary_subcommand.as_str())
                        .args(packets)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped());

        let install_process_result = _install_command.spawn();

        if install_process_result.is_err() {
            let error_message = format!("Error: {:?}", install_process_result.unwrap_err());
            return PacketManagerResultCode::Error(error_message, -1, packets.clone())
        }

        let mut install_process: Child = install_process_result.unwrap();

        let _out_stream: Option<ChildStdout> = install_process.stdout.take();
        let _err_stream: Option<ChildStderr> = install_process.stderr.take();

        let _status_code: Result<ExitStatus, std::io::Error> = install_process.wait();
        if _status_code.is_ok() {
            let status_code = _status_code.unwrap();
            let mut errored_packages: Vec<String> = Vec::new();
            let mut output_packages: Vec<String> = Vec::new();

            if _out_stream.is_some() {
                output_packages.extend(package_lines_parse(_out_stream.unwrap(), output_parser));
            }

            if _err_stream.is_some() {
                errored_packages.extend(package_lines_parse(_err_stream.unwrap(), error_parser));
            }

            if status_code.success() && !output_packages.is_empty() {
                return PacketManagerResultCode::Success(None);
            }
            else if status_code.success() {
                return PacketManagerResultCode::Success(Some(output_packages));
            }
            else {
                return PacketManagerResultCode::Error(format!("Error in stage: {}", stage), status_code.code().unwrap(), errored_packages);
            }
        }

        return PacketManagerResultCode::Error(format!("Error of process start. Stage: {}",  stage), -1, vec![])
    }

    pub fn perform_command_with_standart_parser(
        basic_command: String,
        secondary_subcommand: String,
        stage: Stage,
        packets: &Vec<String>
    ) -> PacketManagerResultCode
    {
        perform_command(basic_command, secondary_subcommand, stage, packets, None, None)
    }

    fn package_lines_parse<T>(_err_stream: T, parser: Option<&mut Box<dyn FnMut(&mut String)>>) -> Vec<String>
    where T: Read {
        let _buf_reader: BufReader<T> = BufReader::new(_err_stream);

        let mut default_function: Box<dyn FnMut(&mut String)> = Box::new(|line: &mut String| {});
        let parser_func: &mut Box<dyn FnMut(&mut String)> = parser.unwrap_or(&mut default_function);

        let mut _packages_result: Vec<String> = Vec::new();
        for line in _buf_reader.lines() {
            if line.is_ok() {
                let mut line_ref: String = line.unwrap();

                parser_func(&mut line_ref);
                _packages_result.push(line_ref.clone());
            }
        }

        return _packages_result;
    }

    pub fn catch_err(command_return: PacketManagerResultCode, stage: Stage, errored_packages: &mut Vec<ErroredPacket>) -> PacketManagerResultCode {
        return match command_return {
            PacketManagerResultCode::Error(message, status_code, packets) => {
                for packet in packets.clone() {
                    errored_packages.push(ErroredPacket { name: packet, stage: stage });
                }

                return PacketManagerResultCode::Error(message, status_code, packets);
            },
            PacketManagerResultCode::Success(val) => PacketManagerResultCode::Success(val),
        };
    }
}
pub mod packet_manager_trait {
    use super::utility::{PacketManagerResultCode, Stage};

    pub type ParserOutput = Box<dyn FnMut(&mut String) + 'static>;

    pub trait PacketManager {
        fn get_performers(&mut self, stage: Stage) -> (Option<&mut ParserOutput>, Option<&mut ParserOutput>);

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

pub mod parsers {
    // let not_found_pattern: Regex = regex!(r#".*"(?<packet_name>[\w\d\s]+)".*((not found)|(не найден))"#).clone();
}

use std::collections::HashMap;

use utility::{PacketManagerResultCode, Stage, ErroredPacket};
use packet_manager_trait::{PacketManager, ParserOutput};

use command_performers::{perform_command, catch_err};
use command_basic_structure::PacketManagerCommand;




pub struct PacketManagerCommandExecutor
{
    command_obj: PacketManagerCommand,
    errored_packages: Vec<ErroredPacket>,
    stage_performers: HashMap<Stage, ParserOutput>,
    stage_errors_performers: HashMap<Stage, Box<dyn FnMut(&mut String) + 'static>>
}

impl PacketManagerCommandExecutor {
    pub fn new_empty() -> PacketManagerCommandExecutor
    {
        return PacketManagerCommandExecutor {
            command_obj: PacketManagerCommand::new_empty("zypper".to_string()),
            errored_packages: vec![],
            stage_performers: HashMap::new(),
            stage_errors_performers: HashMap::new()
        };
    }
}

impl PacketManager for PacketManagerCommandExecutor {
    fn get_performers(&mut self, stage: Stage) -> (Option<&mut ParserOutput>, Option<&mut ParserOutput>) {
        let mut out_parser: Option<&mut Box<dyn FnMut(&mut String) + 'static>> = None;
        if self.stage_performers.contains_key(&stage) {
            out_parser = self.stage_performers.get_mut(&stage);
        }

        let mut err_parser: Option<&mut Box<dyn FnMut(&mut String) + 'static>> = None;
        if self.stage_errors_performers.contains_key(&stage) {
            err_parser = self.stage_errors_performers.get_mut(&stage);
        }

        return (out_parser, err_parser);
    }

    fn install(&mut self, packets: &Vec<String>) -> PacketManagerResultCode {
        let (basic, install) = (self.command_obj.basic_command.clone(), self.command_obj.install_command.clone());
        let (err_parser, out_parser) = self.get_performers(Stage::Install);

        let command_return: PacketManagerResultCode = perform_command(basic, install, Stage::Install, packets, err_parser, out_parser);

        return catch_err(command_return, Stage::Install, &mut self.errored_packages);
    }

    fn remove(&mut self, packets: &Vec<String>) -> PacketManagerResultCode {
        let command_return: PacketManagerResultCode = perform_command(self.command_obj.basic_command.clone(), self.command_obj.remove_command.clone(), Stage::Remove, packets, None, None);

        return catch_err(command_return, Stage::Remove, &mut self.errored_packages);
    }

    fn update(&mut self) -> PacketManagerResultCode {
        let command_return: PacketManagerResultCode = perform_command(self.command_obj.basic_command.clone(), self.command_obj.update_command.clone(), Stage::Update, &Vec::new(), None, None);

        return catch_err(command_return, Stage::Update, &mut self.errored_packages);
    }

    fn show_updates(&self) -> Vec<String> {
        let command_return: PacketManagerResultCode = perform_command(self.command_obj.basic_command.clone(), self.command_obj.update_command.clone(), Stage::Update, &Vec::new(), None , None);

        let mut errored_packets: Vec<ErroredPacket> = vec![];
        let status_code = catch_err(command_return, Stage::Showing, &mut errored_packets);

        return vec![];
    }

    fn update_system(&mut self) -> PacketManagerResultCode {
        let command_return: PacketManagerResultCode = perform_command(self.command_obj.basic_command.clone(), self.command_obj.system_update_command.clone(), Stage::Remove, &vec![], None, None);

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