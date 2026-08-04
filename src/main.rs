mod packet_manager;

use packet_manager::packet_manager_trait::*;
use packet_manager::command_basic_structure::PacketManagerCommand;

use packet_manager::PacketManagerCommandExecutor;

fn main() {
    let command_data = PacketManagerCommand::new("zypper".to_string());
    let mut executor = PacketManagerCommandExecutor::new_empty();
    let vector_collection = vec![String::from("vim"), String::from("gvim")];
    let vector_collection_1 = vec![String::from("vim"), String::from("gvim"), "modiar".to_string(), "AlbusDombalor".to_string(),"libopenh264-8".to_string()];

    executor.install(&vector_collection);
    executor.install(&vector_collection_1);
}
