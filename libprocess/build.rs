use ipc_gen_buildtime::generate_client;
fn main() {
    generate_client("../ipc_definitions/sm.toml");
    generate_client("../ipc_definitions/fs.toml");
    generate_client("../ipc_definitions/pcie.toml");
}
