use package_manager_automatic::command_struct::packet_manager_trait::PacketManager;
use package_manager_automatic::PacketManagerCommandExecutor;

fn main() {
    let mut executor = PacketManagerCommandExecutor::new("zypper".to_string());
    let vector_collection = vec![String::from("vim"), String::from("gvim")];
    let vector_collection_1 = vec![String::from("vim"), String::from("gvim"), "modiar".to_string(), "AlbusDombalor".to_string()];
    let vector_collection_2 = vec![String::from("vim"), String::from("gvim"), "modiar".to_string(), "AlbusDombalor".to_string(),"libopenh264-8".to_string()];

    // executor.install(&vector_collection);
    // executor.install(&vector_collection_1);

    // executor.remove(&vector_collection_1);
    // executor.install(&vector_collection);

    for package in executor.search(vector_collection_1) {
        println!("Search package: {:?}", package);
    }

    for package in executor.packets_list() {
        println!("Package: {:?}", package);
    }

    for repo in executor.repos() {
        println!("Repo: {:?}", repo);
    }

    // executor.show_updates();
    // executor.update();
    // executor.update_system();
}
