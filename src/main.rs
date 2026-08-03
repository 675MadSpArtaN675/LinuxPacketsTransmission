mod packet_manager;

use packet_manager::*;
use packet_manager::packet_manager_trait::*;

fn main() {
    let command_data = PacketManagerCommand::new(String::from("zypper"));
    let mut executor = PacketManagerCommandExecutor{ command_obj: command_data, errored_packages: vec![] };
    let vector_collection = vec![String::from("vim"), String::from("gvim")];
    let vector_collection_1 = vec![String::from("vim"), String::from("gvim"), "modiar".to_string(), "AlbusDombalor".to_string(),"libopenh264-8".to_string()];

    executor.install(&vector_collection);
    executor.install(&vector_collection_1);
}
