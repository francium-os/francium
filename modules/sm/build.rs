use ipc_gen_buildtime::generate_server;
fn main() {
	generate_server("../../ipc_definitions/sm.toml");
}