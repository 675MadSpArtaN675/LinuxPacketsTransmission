mod utility {
    pub enum PacketManagerResultCode {
        Success,
        Error(String, Vec<String>)
    }
}

mod PacketManagerTrait {
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
    }
}