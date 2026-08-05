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
        Search,
        Showing
    }

    impl Stage {
        pub fn to_string(&self) -> &str {
            let result: &str = match &self {
                Stage::Install => "Install",
                Stage::Update => "Update",
                Stage::Remove => "Remove",
                Stage::Search => "Search",
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

    pub enum InstallFlag {
        Installed,
        InstalledPlus,
        NoInstalled
    }

    impl InstallFlag {
        pub fn str_to_enum(line: &str) -> InstallFlag {
            return match line {
                "i" => InstallFlag::Installed,
                "i+" => InstallFlag::InstalledPlus,
                "n" => InstallFlag::NoInstalled,

                _ => InstallFlag::NoInstalled
            };
        }
    }

    pub struct FoundPackage {
        pub name: String,
        pub description: String,
        pub install_flag: InstallFlag,
        pub type_of_packet: String
    }
}

pub mod command_basic_structure {
    pub struct PacketManagerCommand {
        pub basic_command: String,
        pub install_command: String,
        pub remove_command: String,
        pub search_command: String,
        pub list_command: String,
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
                list_command: String::new(),
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
                command_obj.install_command += " -y";
                command_obj.remove_command += " -y";
                command_obj.list_command = "search --installed-only".to_string();
                command_obj.update_command += " -y";
                command_obj.check_update_command = String::from("refresh");
                command_obj.system_update_command = String::from("dist-upgrade -y");
            },
            "dnf" => {
                create_standard_commands(&mut command_obj);
                command_obj.list_command = "list --installed".to_string();
                command_obj.update_command = String::from("upgrade");
                command_obj.check_update_command = String::from("check");
            },
            "apt" => {
                create_standard_commands(&mut command_obj);
                command_obj.list_command = "list installed".to_string();
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
    use std::rc::Rc;
    use std::cell::RefCell;

    use super::utility::{Stage, PacketManagerResultCode, ErroredPacket};

    use std::io::{BufReader, BufRead, Read};
    use std::process::{Child, ChildStderr, ChildStdout, Command, ExitStatus, Stdio};

    pub fn perform_command(basic_command: String,
                    secondary_subcommand: String,
                    stage: Stage,
                    packets: &Vec<String>,
                    is_admin: bool,
                    error_parser: Option<&mut Box<dyn FnMut(&mut String)>>,
                    output_parser: Option<&mut Box<dyn FnMut(&mut String)>>
    ) -> PacketManagerResultCode
    {
        let mut command_parts: Vec<String> = Vec::new();

        if is_admin {
            command_parts.push("sudo".to_string());
        }

        command_parts.extend(basic_command.split(" ").filter(|line| !line.is_empty()).map(|line| line.to_string()));
        command_parts.extend(secondary_subcommand.split(" ").filter(|line| !line.is_empty()).map(|line| line.to_string()));

        let mut _install_command: Command = Command::new(command_parts.remove(0));
        _install_command.stdout(Stdio::piped())
                        .stderr(Stdio::piped());

        for arg in command_parts {
            _install_command.arg(arg);
        }

        _install_command.args(packets);

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
        packets: &Vec<String>,
        is_admin: bool
    ) -> PacketManagerResultCode
    {
        perform_command(basic_command, secondary_subcommand, stage, packets, is_admin, None, None)
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

    pub fn catch_err(command_return: PacketManagerResultCode, stage: Stage, errored_packages: &Rc<RefCell<Vec<ErroredPacket>>>) -> PacketManagerResultCode {
        return match command_return {
            PacketManagerResultCode::Error(message, status_code, packets) => {
                for packet in packets.clone() {
                    errored_packages.borrow_mut().push(ErroredPacket { name: packet, stage: stage });
                }

                return PacketManagerResultCode::Error(message, status_code, packets);
            },
            PacketManagerResultCode::Success(val) => PacketManagerResultCode::Success(val),
        };
    }
}
pub mod packet_manager_trait {
    use crate::packet_manager::utility::FoundPackage;

use super::utility::{PacketManagerResultCode, Stage};

    pub type ParserOutput = Box<dyn FnMut(&mut String) + 'static>;

    pub trait PacketManager {
        fn get_performers(&mut self, stage: Stage) -> (Option<&mut ParserOutput>, Option<&mut ParserOutput>);

        fn install(&mut self, packets: &Vec<String>) -> PacketManagerResultCode;
        fn remove(&mut self, packets: &Vec<String>) -> PacketManagerResultCode;

        fn search(&mut self, packets: Vec<String>) -> Vec<FoundPackage>;
        fn packets_list(&mut self) -> Vec<FoundPackage>;

        fn update(&mut self) -> PacketManagerResultCode;
        fn show_updates(&mut self) -> Vec<String>;
        fn update_system(&mut self) -> PacketManagerResultCode;

        fn repos(&self) -> Vec<String>;
        fn add_repo(&self, repo_name: &str, repo_url: &str);
        fn remove_repo(&self, repo_name: &str);
    }
}

pub mod parsers {
    pub(self) mod zypper_module {
        use regex::{Regex, regex};
        use itertools::Itertools;

        pub(super) fn zypper_parse_install_errors(line: &mut String, packets_list: &mut Vec<String>) {
            let not_found_pattern: Regex = regex!(r#".*"(?<packet_name>[\w\d\s]+)".*((not found)|(не найден))"#).clone();

            if let Some(captures) = not_found_pattern.captures(line) {
                if let Some(named_capt) = captures.name("packet_name") {
                    let packet_name = named_capt.as_str().to_string();

                    packets_list.push(packet_name);
                }
            }
        }
        pub(super) fn print_line(line: &mut String) {
            println!("Line: {}", line.clone());
        }

        pub(super) fn parse_table(line: &mut String, partitioner: &str, packets_found: &mut Vec<String>) {
            let pattern = Regex::new(r#"[\s|]+(S|Name|Summary|Type)[\s|]+"#).unwrap();
            let start_pattern: Regex = Regex::new(r#"^i\+?"#).unwrap();

            if line.contains(partitioner) && !pattern.is_match(line) {
                let mut line_preready = line.clone().split("|").map(|item| {item.trim()}).join("|");

                if !start_pattern.is_match(&line_preready) {
                    line_preready = "n".to_string() + line_preready.as_str();
                }

                packets_found.push(line_preready);
            }
        }
    }

    use std::{ops::DerefMut, rc::Rc};
    use std::cell::RefCell;
    use std::collections::HashMap;

    use super::utility::Stage;
    use super::packet_manager_trait::ParserOutput;
    use super::utility::ErroredPacket;

    use zypper_module::{zypper_parse_install_errors, print_line, parse_table};

    pub fn fill_error_performers(packet_manager_name: &String, base_map_of_functions: &mut HashMap<Stage, ParserOutput>, errored_packets_collection: &Rc<RefCell<Vec<ErroredPacket>>>) {
        let name = packet_manager_name.as_str();
        match name {
            "zypper" => {
                let errored_packages = errored_packets_collection.clone();
                base_map_of_functions.insert(Stage::Install, Box::new(move |line: &mut String| {
                    let mut _pack_list: Vec<String> = Vec::new();
                    zypper_parse_install_errors(line, &mut _pack_list);

                    errored_packages.borrow_mut().extend(_pack_list.iter().map(|item| ErroredPacket { name: item.clone(), stage: Stage::Install}));
                }));
            },

            _ => {}
        }
    }

    pub fn fill_performers(packet_manager_name: &String, base_map_of_functions: &mut HashMap<Stage, ParserOutput>, valid_lines_collection: &Rc<RefCell<Vec<String>>>) {
        let name = packet_manager_name.as_str();
        match name {
            "zypper" => {
                let ptr_collection = valid_lines_collection.clone();
                base_map_of_functions.insert(Stage::Install, Box::new(print_line));
                base_map_of_functions.insert(Stage::Update, Box::new(print_line));
                base_map_of_functions.insert(Stage::Remove, Box::new(print_line));
                base_map_of_functions.insert(Stage::Showing, Box::new(print_line));
                base_map_of_functions.insert(Stage::Search,
                    Box::new(
                        move |line: &mut String| {
                            parse_table(line, "|", ptr_collection.borrow_mut().deref_mut());
                    }
                ));
            },

            _ => {}
        }
    }
}

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use utility::{PacketManagerResultCode, Stage, ErroredPacket, FoundPackage, InstallFlag};
use packet_manager_trait::{PacketManager, ParserOutput};

use parsers::{fill_error_performers, fill_performers};
use command_performers::{perform_command, catch_err};
use command_basic_structure::PacketManagerCommand;

fn create_packages(valid_lines: &Rc<RefCell<Vec<String>>>) -> Vec<FoundPackage> {
    let mut packages: Vec<FoundPackage> = vec![];
    let mut ref_to_lines = valid_lines.borrow_mut();

    for parsed_line in ref_to_lines.iter().map(|line| line.split("|").map(|s| s.to_string())) {
        let line: Vec<String> = parsed_line.collect();

        packages.push(FoundPackage {name: line[1].clone(), description: line[2].clone(), install_flag: InstallFlag::str_to_enum(line[0].as_str()), type_of_packet: line[3].clone()});
    }

    ref_to_lines.clear();

    return packages;
}

pub struct PacketManagerCommandExecutor
{
    command_obj: PacketManagerCommand,

    errored_packages: Rc<RefCell<Vec<ErroredPacket>>>,
    valid_lines: Rc<RefCell<Vec<String>>>,

    stage_performers: HashMap<Stage, ParserOutput>,
    stage_errors_performers: HashMap<Stage, ParserOutput>
}

impl PacketManagerCommandExecutor {
    pub fn new_empty() -> PacketManagerCommandExecutor
    {
        return PacketManagerCommandExecutor {
            command_obj: PacketManagerCommand::new_empty("zypper".to_string()),
            errored_packages: Rc::new(RefCell::new(vec![])),
            valid_lines: Rc::new(RefCell::new(vec![])),
            stage_performers: HashMap::new(),
            stage_errors_performers: HashMap::new()
        };
    }

    pub fn new(base_name: String) -> PacketManagerCommandExecutor {
        let mut pm_executer: PacketManagerCommandExecutor = PacketManagerCommandExecutor {
            command_obj: PacketManagerCommand::new(base_name.clone()),
            errored_packages: Rc::new(RefCell::new(vec![])),
            valid_lines: Rc::new(RefCell::new(vec![])),
            stage_performers: HashMap::new(),
            stage_errors_performers: HashMap::new()
        };

        fill_error_performers(&base_name.clone(), &mut pm_executer.stage_errors_performers, &pm_executer.errored_packages);
        fill_performers(&base_name.clone(), &mut pm_executer.stage_performers, &pm_executer.valid_lines);

        return pm_executer;
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
        let (out_parser, err_parser) = self.get_performers(Stage::Install);

        let command_return: PacketManagerResultCode = perform_command(basic, install, Stage::Install, packets, true, err_parser, out_parser);

        return catch_err(command_return, Stage::Install, &mut self.errored_packages);
    }

    fn remove(&mut self, packets: &Vec<String>) -> PacketManagerResultCode {
        let (basic, sec) = (self.command_obj.basic_command.clone(), self.command_obj.remove_command.clone());
        let (out_parser, err_parser) = self.get_performers(Stage::Remove);

        let command_return: PacketManagerResultCode = perform_command(basic, sec, Stage::Remove, packets, true, err_parser, out_parser);

        return catch_err(command_return, Stage::Remove, &mut self.errored_packages);
    }

    fn update(&mut self) -> PacketManagerResultCode {
        let (basic, sec) = (self.command_obj.basic_command.clone(), self.command_obj.update_command.clone());
        let (out_parser, err_parser) = self.get_performers(Stage::Update);

        let command_return: PacketManagerResultCode = perform_command(basic, sec, Stage::Update, &Vec::new(), true, err_parser, out_parser);

        return catch_err(command_return, Stage::Update, &mut self.errored_packages);
    }

    fn show_updates(&mut self) -> Vec<String> {
        let (basic, sec) = (self.command_obj.basic_command.clone(), self.command_obj.check_update_command.clone());
        let (out_parser, err_parser) = self.get_performers(Stage::Showing);

        let command_return: PacketManagerResultCode = perform_command(basic, sec, Stage::Showing, &Vec::new(), true, err_parser, out_parser);

        let errored_packets: Rc<RefCell<Vec<ErroredPacket>>> = Rc::new(RefCell::new(vec![]));
        let status_code = catch_err(command_return, Stage::Showing, &errored_packets);

        return match status_code {
            PacketManagerResultCode::Success(addons) => addons.unwrap_or(vec![]),
            _ => vec![]
        };
    }

    fn update_system(&mut self) -> PacketManagerResultCode {
        let (basic, sec) = (self.command_obj.basic_command.clone(), self.command_obj.system_update_command.clone());
        let (out_parser, err_parser) = self.get_performers(Stage::Update);

        let command_return: PacketManagerResultCode = perform_command(basic, sec, Stage::Update, &vec![], true, err_parser, out_parser);

        return catch_err(command_return, Stage::Update, &mut self.errored_packages);
    }

    fn search(&mut self, packets: Vec<String>) -> Vec<FoundPackage> {
        let (basic, sec) = (self.command_obj.basic_command.clone(), self.command_obj.search_command.clone());

        for packet in packets {
            let (out_parser, err_parser) = self.get_performers(Stage::Search);

            let command_return: PacketManagerResultCode = perform_command(basic.clone(), sec.clone(), Stage::Search, &vec![packet], true, err_parser, out_parser);
        }


        return create_packages(&self.valid_lines);
    }

    fn packets_list(&mut self) -> Vec<FoundPackage> {
        let (basic, sec) = (self.command_obj.basic_command.clone(), self.command_obj.list_command.clone());
        let (out_parser, err_parser) = self.get_performers(Stage::Search);

        let command_return: PacketManagerResultCode = perform_command(basic, sec, Stage::Search, &vec![], true, err_parser, out_parser);

        return create_packages(&self.valid_lines);
    }

    fn repos(&self) -> Vec<String> {
        return vec![];
    }

    fn add_repo(&self, repo_name: &str, repo_url: &str) {
    }

    fn remove_repo(&self, repo_name: &str) {

    }

}