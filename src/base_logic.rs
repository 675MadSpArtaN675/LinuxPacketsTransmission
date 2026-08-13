mod utility_logic {
    use package_manager_automatic::command_struct::packet_manager_trait::{JsonTransformable, PackageNamed};

    use json::JsonValue;
    use tokio::net::{TcpStream, TcpListener};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::runtime::{Runtime, Builder};

    pub(super) use regex::Regex;
    pub(super) use json::array;
    pub(super) use log::{debug, info, error};

    pub(super) fn create_runtime() -> Option<Runtime> {
        let mut runtime_builder = Builder::new_current_thread();
        runtime_builder.enable_all();

        let runtime_result = runtime_builder.build();

        if let Ok(runtime) = runtime_result {
            debug!("Runtime created...");
            return Some(runtime);
        }
        else if let Err(error) = runtime_result {
            error!("Error when creating runtime in json: {}", error)
        }

        return None;
    }

    pub(super) fn create_json_of_packet_list(filter_patterns: Vec<String>, packets_list: Vec<impl JsonTransformable + PackageNamed + Clone>) -> String {
        info!("Input filters: {}", filter_patterns.clone().iter().map(|s| format!("'{}' ", s)).collect::<Vec<String>>().join(" "));
        let installed_packages_list = packets_list;

        let mut _array = array![];

        for package in installed_packages_list.iter() {
            if filter_patterns.len() > 0 {
                for pattern_line in filter_patterns.iter() {
                    let pattern = Regex::new(&pattern_line).unwrap();

                    if pattern.is_match(&package.get_name().as_str()) {
                        if let Err(added_value) = _array.push(package.clone().to_json()) {
                            error!("Error of serialization: {}", added_value);
                        }
                    }
                }
            }
            else {
                if let Err(added_value) = _array.push(package.clone().to_json()) {
                    error!("Error of serialization: {}", added_value);
                }
            }
        }

        return json::stringify(_array);
    }

    pub(super) fn parse_json_of_package_list<T>(json_text: String) -> Option<Vec<T>> where T: Clone + JsonTransformable<ReturnType = T>  {
        let json_obj_result = json::parse(json_text.as_str());

        if let Ok(JsonValue::Array(json_objs)) = json_obj_result {
            let mut packages: Vec<T> = Vec::new();
            for json_obj in json_objs {
                packages.push(T::from_json_obj(json_obj.clone()).clone());
            }

            return Some(packages);
        }

        return None;
    }

    pub(super) async fn send_list(ip_address: String, port: u32, json_list: String, chunk_size_of_list: usize) {
        info!("Sending list to {}:{}. Chunk size: {}", ip_address, port, chunk_size_of_list);
        let listener_connect = TcpStream::connect(format!("{}:{}", ip_address, port)).await;

        if let Ok(mut listener) = listener_connect {
            debug!("Parting json to chunks.");
            let json_list_parted = json_list.as_bytes().chunks(chunk_size_of_list);

            for chunk in json_list_parted {
                let write_result = listener.write(chunk).await;

                if let Ok(bytes_readed) = write_result {
                    info!("Writed bytes count: {}", bytes_readed);
                }
                else {
                    error!("Error when sending packages: {}", write_result.unwrap_err());
                }
            }
            debug!("File is send successfully...");
        }
        else if let Err(error) = listener_connect {
            error!("Error when sending: {}", error);
        }
    }

    pub(super) async fn send_list_to_any_clients(ip_addresses: &Vec<String>, port: u32, json_list: String, chunk_size_of_list: usize) {
        info!("Sending list on any ip addresses...");

        for ip_address in ip_addresses {
            send_list(ip_address.clone(), port, json_list.clone(), chunk_size_of_list).await;
        }

        info!("Sending is finished...");
    }

    pub(super) async fn recieve_list(port: u32) -> Option<String> {
        info!("Listening port {} to get package list", port);
        let listener_result = TcpListener::bind(format!("0.0.0.0:{}", port)).await;

        if let Ok(listener) = listener_result {
            let listener_stream = listener.accept().await;

            if let Ok(connected_stream) = listener_stream {
                let (mut stream, socket_addr) = connected_stream;
                info!("Recieving from address {}", socket_addr);

                let mut error: Option<std::io::Error> = stream.take_error().unwrap_or(None);
                let mut json_bytes_full: Vec<u8> = Vec::new();
                while error.is_none() {
                    let mut json_bytes: Vec<u8> = Vec::new();
                    let bytes_getted = stream.read_buf(&mut json_bytes).await;

                    if let Ok(bytes_count) = bytes_getted {
                        debug!("Bytes getted: {}", bytes_count);

                        if bytes_count <= 0 {
                            break;
                        }

                        json_bytes_full.extend(json_bytes);
                    }
                    else if let Err(error) = bytes_getted {
                        error!("Error in getting bytes: {}", error);
                    }

                    error = stream.take_error().unwrap_or(None);
                }

                if let Some(err_) = error {
                    error!("Error of getting bytes: {}", err_);
                }

                let transformed_string_result = String::from_utf8(json_bytes_full);

                if let Ok(transformed_string) = transformed_string_result {
                    info!("Line recieved successfully");
                    return Some(transformed_string);
                }
                else if let Err(transformed_error) = transformed_string_result {
                    error!("Error when transforming getted line: {}", transformed_error);
                }
            }
            else if let Err(error) = listener_stream {
                error!("Error in accept: {}", error);
            }
        }
        else if let Err(error) = listener_result {
            error!("Error in bind: {}", error);
        }

        return None;
    }
}

use cwd::cwd;

use package_manager_automatic::utility::{FoundPackage, Repository};
use package_manager_automatic::PacketManagerCommandExecutor;
use package_manager_automatic::utility::PacketManagerResultCode;
use package_manager_automatic::command_struct::packet_manager_trait::PacketManager;

use utility_logic::*;
use crate::file_saver::{Saver, FileSaver};

pub struct AppBaseLogic {
    executor: PacketManagerCommandExecutor,

    filter_patterns: Vec<String>,
    filter_repo_patterns: Vec<String>,

    saver: Option<Box<dyn Saver>>,

    chunk_size: usize
}

impl AppBaseLogic {
    fn send_list_json(&mut self, ip_list: Vec<String>, port: u32, mut create_list: Box<dyn FnMut() -> String>) {
        info!("Sending installed packages list ion port {}", port);
        let runtime_opt = create_runtime();

        if let Some(runtime) = runtime_opt {
            debug!("Getting pacakge list in json");
            let json_list: String = create_list();

            if self.saver.is_some() {
                self.saver.as_ref().unwrap().save(&json_list);
            }

            runtime.block_on(async {
                send_list_to_any_clients(&ip_list, port, json_list, self.chunk_size).await;
            });

            debug!("Sending is finished...");
        }
    }

    fn recieve_list_json<T>(&mut self, port: u32, mut parser: Box<impl FnMut(String) -> Option<T>>) -> Option<T>  {
        info!("Recieving package on port {}", port);
        let runtime_result = create_runtime();

        if let Some(runtime) = runtime_result {
            let mut result_string: String = String::new();

            runtime.block_on(async {
                let readed_string_opt = recieve_list(port).await;

                if let Some(readed_string) = readed_string_opt {
                    result_string.push_str(readed_string.as_str());
                }
            });

            if self.saver.is_some() {
                self.saver.as_ref().unwrap().save(&result_string);
            }

            debug!("String recieved! Her len: {}", result_string.len());
            return parser(result_string);
        }

        return None;
    }

    pub fn new(basic_packet_manager: String, path_to_save: Option<String>) -> AppBaseLogic {
        info!("Creating base logic for packet manager '{}' ", basic_packet_manager);
        let mut saver: Option<Box<dyn Saver>> = None;
        if let Some(path) = path_to_save {
            info!("Save path of lists '{}'", path);

            saver = Some(Box::new(FileSaver::new(path)));
        }

        return AppBaseLogic { executor: PacketManagerCommandExecutor::new(basic_packet_manager), filter_patterns: vec![], filter_repo_patterns: vec![], saver: saver, chunk_size: 64usize};
    }

    pub fn get_repo_filters_count(&self) -> usize
    {
        return self.filter_repo_patterns.len();
    }

    pub fn get_package_filters_count(&self) -> usize {
        return self.filter_patterns.len();
    }

    pub fn set_other_packet_manager(&mut self, packet_manager_name: String) {
        self.executor = PacketManagerCommandExecutor::new(packet_manager_name);
    }

    pub fn get_packet_manager_name(&self) -> String {
        return self.executor.get_base_command_name();
    }

    pub fn set_chunk_size(&mut self, chunk_size: usize) {
        self.chunk_size = chunk_size;
    }

    pub fn get_chunk_size(&self) -> usize {
        return self.chunk_size;
    }

    pub fn add_filter(&mut self, pattern: String) {
        if !pattern.is_empty() && !self.filter_patterns.contains(&pattern) {
            self.filter_patterns.push(pattern);
        }
    }

    pub fn add_repo_filter(&mut self, pattern: String) {
        if !pattern.is_empty() && !self.filter_repo_patterns.contains(&pattern) {
            self.filter_repo_patterns.push(pattern);
        }
    }

    pub fn get_filter(&self, index: usize) -> Option<&String> {
        return self.filter_patterns.get(index);
    }

    pub fn get_repo_filter(&self, index: usize) -> Option<&String> {
        return self.filter_repo_patterns.get(index);
    }

    pub fn get_filter_mut(&mut self, index: usize) -> Option<&mut String> {
        return self.filter_patterns.get_mut(index);
    }

    pub fn get_repo_filter_mut(&mut self, index: usize) -> Option<&mut String> {
        return self.filter_repo_patterns.get_mut(index);
    }

    pub fn send_repository_list(&mut self, ip_list: Vec<String>, port: u32) {
        let patterns: Vec<String> = self.filter_repo_patterns.clone();
        let repositories: Vec<Repository> = self.executor.repos().clone();

        let create_list_func: Box<dyn FnMut() -> String> = Box::new(move || create_json_of_packet_list(patterns.clone(), repositories.clone()));

        self.send_list_json(ip_list, port, create_list_func);
    }

    pub fn recieve_repository_list(&mut self, port: u32) -> Option<Vec<Repository>>
    {
        let parser_func= Box::new(|line| parse_json_of_package_list::<Repository>(line) );

        return self.recieve_list_json(port, parser_func);
    }

    pub fn send_packages_list(&mut self, ip_list: Vec<String>, port: u32) {
        let patterns: Vec<String> = self.filter_patterns.clone();
        let packets: Vec<FoundPackage> = self.executor.packets_list().clone();

        let create_list_func: Box<dyn FnMut() -> String> = Box::new(move || { create_json_of_packet_list(patterns.clone(), packets.clone()) });

        self.send_list_json(ip_list, port, create_list_func);
    }

    pub fn recieve_packages_list(&mut self, port: u32) -> Option<Vec<FoundPackage>>
    {
        let parser_func= Box::new(|line| parse_json_of_package_list::<FoundPackage>(line) );

        return self.recieve_list_json(port, parser_func);
    }

    pub fn add_repo(&mut self, uri: String, alias: String) {
        if !uri.is_empty() && !alias.is_empty() && !self.executor.has_repo(alias.clone()) {
            self.executor.add_repo(alias.as_str(), uri.as_str());
        }
    }

    pub fn remove_repo(&mut self, alias: String) {
        if !alias.is_empty() && self.executor.has_repo(alias.clone()){
            self.executor.remove_repo(alias.as_str());
        }
    }

    pub fn install_packages(&mut self, packages: Vec<String>) {
        if !packages.is_empty() {
            let return_code = self.executor.install(&packages);

            if let PacketManagerResultCode::Error(name, ret_code, errored_packet)  = return_code {
                error!("Error in install stage. Name: {}. Return Code: {}.\nPackages error: {}", name, ret_code, errored_packet.join(" "));
            }
        }
    }

    pub fn remove_packages(&mut self, packages: Vec<String>)
    {
        if !packages.is_empty() {
            let return_code = self.executor.remove(&packages);

            if let PacketManagerResultCode::Error(name, ret_code, errored_packet)  = return_code {
                error!("Error in install stage. Name: {}. Return Code: {}.\nPackages error: {}", name, ret_code, errored_packet.join(" "));
            }
        }
    }

    pub fn search_by_any_pattern(&mut self, patterns: Vec<String>) -> Option<Vec<FoundPackage>> {
        if !patterns.is_empty() {
            return Some(self.executor.search(patterns.clone()));
        }

        return None;
    }
}