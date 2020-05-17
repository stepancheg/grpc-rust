use grpc::prelude::*;
use grpc_interop::interop_client;
use grpc_interop::test_grpc::TestServiceClient;

use clap::App;
use clap::Arg;
use grpc_interop::DEFAULT_PORT;

// The flags we use are defined in the gRPC Interopability doc
// https://github.com/grpc/grpc/blob/master/doc/interop-test-descriptions.md
fn main() {
    env_logger::init();

    let options = App::new("gRPC interopability client")
        .version("0.1")
        .author("Steve Jenson <stevej@buoyant.io>")
        .about("Interoperability Test Client for grpc-rust")
        .arg(
            Arg::with_name("server_host")
                .long("server_host")
                .help("The server host to connect to. For example, \"localhost\" or \"127.0.0.1\"")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("server_host_override")
                .long("server_host_override")
                .help(
                    "The server host to claim to be connecting to, for use in TLS and HTTP/2 \
                     :authority header. If unspecified, the value of --server_host will be used",
                )
                .takes_value(true),
        )
        .arg(
            Arg::with_name("server_port")
                .long("server_port")
                .help("The server port to connect to. For example, \"8080\"")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("test_case")
                .long("test_case")
                .help("The name of the test case to execute. For example, \"empty_unary\"")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("use_tls") // boolean
                .long("use_tls")
                .help("Whether to use a plaintext or encrypted connection")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("use_test_ca")
                .long("use_test_ca")
                .help("Whether to replace platform root CAs with ca.pem as the CA root")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("default_service_account")
                .long("default_service_account")
                .help("Email of the GCE default service account.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("oauth_scope")
                .long("oauth_scope")
                .help("OAuth scope. For example, \"https://www.googleapis.com/auth/xapi.zoo\"")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("service_account_key_file")
                .long("service_account_key_file")
                .help(
                    "The path to the service account JSON key file generated from GCE developer \
                     console.",
                )
                .takes_value(true),
        )
        .get_matches();

    let hostname = options.value_of("server_host").unwrap_or("127.0.0.1");
    let serverport = options
        .value_of("server_port")
        .map(|s| s.parse().unwrap())
        .unwrap_or(DEFAULT_PORT);

    let client =
        TestServiceClient::new_plain(hostname, serverport, Default::default()).expect("init");

    let testcase = options.value_of("test_case").unwrap_or("");
    'b: loop {
        for &(t, f) in interop_client::TESTS {
            if t == testcase {
                f(client);
                break 'b;
            }
        }

        panic!("no test_case specified or unknown test case");
    }
}
