use clap::{crate_authors, crate_version, App, AppSettings, Arg, ArgMatches, SubCommand};
use env_logger;
use lazy_static::{__lazy_static_create, __lazy_static_internal, lazy_static};
#[allow(unused_imports)]
use log::{debug, error, info, log, trace, warn};
use std::env;
use transportation::EncryptionPerspective;

lazy_static! {
    pub(crate) static ref MATCHES: ArgMatches<'static> = create_matches();
}

crate fn create_app() -> App<'static, 'static> {
    let metacommand = Arg::with_name("metacommand")
        .short("m")
        .long("metacommand")
        .takes_value(true)
        .multiple(true)
        .number_of_values(1)
        .help("A command to run after the connection is established. The same commands from the F10 prompt.");
    let identity = Arg::with_name("identity")
        .short("i")
        .long("identity")
        .takes_value(true)
        .help("Use [identity] as authentication information for connecting to the remote server.")
        .env("OXY_IDENTITY");
    let command = Arg::with_name("command").index(2);
    let l_portfwd = Arg::with_name("local port forward")
        .multiple(true)
        .short("L")
        .takes_value(true)
        .number_of_values(1)
        .help("Create a local portforward")
        .display_order(102);
    let r_portfwd = Arg::with_name("remote port forward")
        .multiple(true)
        .short("R")
        .number_of_values(1)
        .takes_value(true)
        .help("Create a remote portforward")
        .display_order(103);
    let d_portfwd = Arg::with_name("socks")
        .multiple(true)
        .short("D")
        .long("socks")
        .help("Bind a local port as a SOCKS5 proxy")
        .number_of_values(1)
        .takes_value(true)
        .display_order(104);
    let port = Arg::with_name("port")
        .short("p")
        .long("port")
        .help("The port used for TCP")
        .takes_value(true)
        .default_value("2600");
    let user = Arg::with_name("user")
        .long("user")
        .takes_value(true)
        .help("The remote username to log in with. Only applicable for servers using --su-mode");
    let via = Arg::with_name("via")
        .long("via")
        .takes_value(true)
        .multiple(true)
        .number_of_values(1)
        .help("Connect to a different oxy server first, then proxy traffic through the intermediary server.");
    let verbose = Arg::with_name("verbose")
        .long("verbose")
        .multiple(true)
        .short("v")
        .help("Increase debugging output");
    let xforward = Arg::with_name("X Forwarding").short("X").long("x-forwarding").help("Enable X forwarding");
    let trusted_xforward = Arg::with_name("Trusted X Forwarding")
        .short("Y")
        .long("trusted-x-forwarding")
        .help("Enable trusted X forwarding");
    let server_config = Arg::with_name("server config")
        .long("server-config")
        .help("Path to server.conf")
        .default_value("~/.config/oxy/server.conf");
    let client_config = Arg::with_name("client config")
        .long("client-config")
        .help("Path to client.conf")
        .default_value("~/.config/oxy/client.conf");
    let forced_command = Arg::with_name("forced command")
        .long("forced-command")
        .help("Restrict command execution to the specified command")
        .takes_value(true);
    let unsafe_reexec = Arg::with_name("unsafe reexec")
        .long("unsafe-reexec")
        .help("Bypass safety restrictions intended to avoid privilege elevation");
    let compression = Arg::with_name("compression")
        .short("C")
        .long("compress")
        .help("Enable ZLIB format compression of all transmitted data");
    let no_tmux = Arg::with_name("no tmux")
        .long("no-tmux")
        .help("Do not use a terminal multiplexer as the default pty command");
    let multiplexer = Arg::with_name("multiplexer")
        .long("multiplexer")
        .default_value("/usr/bin/tmux new-session -A -s oxy")
        .help(
            "The command to attach to a terminal multiplexer. Ignored if the first component is not an existent file, or if --no-tmux is supplied.",
        );
    let client_args = vec![
        metacommand.clone(),
        identity.clone(),
        l_portfwd,
        r_portfwd,
        d_portfwd,
        port.clone(),
        xforward,
        trusted_xforward,
        server_config.clone(),
        client_config.clone(),
        user,
        via,
        compression.clone(),
        verbose.clone(),
        command,
    ];
    let server_args = vec![
        server_config.clone(),
        client_config.clone(),
        forced_command,
        identity.clone(),
        port.clone(),
        verbose.clone(),
        no_tmux.clone(),
        multiplexer.clone(),
    ];

    let subcommands = vec![
        SubCommand::with_name("client")
            .about("Connect to an Oxy server.")
            .args(&client_args)
            .arg(Arg::with_name("destination").index(1).required(true)),
        SubCommand::with_name("reexec")
            .about("Service a single oxy connection. Not intended to be run directly, run by oxy server")
            .setting(AppSettings::Hidden)
            .arg(Arg::with_name("fd").long("fd").takes_value(true).required(true))
            .args(&server_args),
        SubCommand::with_name("server")
            .about("Listen for port knocks, accept TCP connections, then reexec for each one.")
            .args(&server_args)
            .arg(unsafe_reexec),
        SubCommand::with_name("serve-one")
            .about("Accept a single TCP connection, then service it in the same process.")
            .args(&server_args)
            .arg(Arg::with_name("bind-address").index(1).default_value("::0")),
        SubCommand::with_name("reverse-server")
            .about("Connect out to a listening client. Then, be a server.")
            .args(&server_args)
            .arg(Arg::with_name("destination").index(1).required(true)),
        SubCommand::with_name("reverse-client")
            .about("Bind a port and wait for a server to connect. Then, be a client.")
            .args(&client_args)
            .arg(Arg::with_name("bind-address").index(1).default_value("::0")),
        SubCommand::with_name("copy")
            .about("Copy files from any number of sources to one destination.")
            .arg(client_config)
            .arg(server_config)
            .arg(compression)
            .arg(Arg::with_name("location").index(1).multiple(true).number_of_values(1))
            .arg(identity.clone())
            .arg(verbose.clone()),
        SubCommand::with_name("guide").about("Print information to help a new user get the most out of Oxy."),
        SubCommand::with_name("keygen").about("Generate keys"),
    ];
    let subcommands: Vec<_> = subcommands.into_iter().map(|x| x.setting(AppSettings::UnifiedHelpMessage)).collect();
    App::new("oxy")
        .version(crate_version!())
        .author(crate_authors!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::UnifiedHelpMessage)
        .subcommands(subcommands)
}

fn create_matches() -> ArgMatches<'static> {
    trace!("Parsing arguments");
    let basic = create_app().get_matches_from_safe(::std::env::args());
    if basic.is_err() && basic.as_ref().unwrap_err().kind == ::clap::ErrorKind::HelpDisplayed {
        return create_app().get_matches();
    }
    if let Ok(matches) = basic {
        return matches;
    }
    trace!("Trying implicit 'client'");
    let mut args2: Vec<String> = ::std::env::args().collect();
    args2.insert(1, "client".to_string());
    if let Ok(matches) = create_app().get_matches_from_safe(args2) {
        return matches;
    }
    create_app().get_matches()
}

crate fn batched_metacommands() -> Vec<String> {
    let values = MATCHES.subcommand_matches(mode()).unwrap().values_of("metacommand");
    if values.is_none() {
        return Vec::new();
    }
    values.unwrap().map(|x| x.to_string()).collect()
}

crate fn process() {
    ::lazy_static::initialize(&MATCHES);
    let level = match matches().occurrences_of("verbose") {
        0 => "info",
        1 => "debug",
        2 => "trace",
        _ => "trace",
    };
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", format!("oxy={}", level));
    }
    env_logger::try_init().ok();
}

crate fn mode() -> String {
    MATCHES.subcommand_name().unwrap().to_string()
}

crate fn matches() -> &'static ArgMatches<'static> {
    MATCHES.subcommand_matches(mode()).unwrap()
}

crate fn destination() -> String {
    MATCHES.subcommand_matches(mode()).unwrap().value_of("destination").unwrap().to_string()
}

crate fn bind_address() -> String {
    MATCHES
        .subcommand_matches(mode())
        .unwrap()
        .value_of("bind-address")
        .unwrap_or("0.0.0.0")
        .to_string()
}

crate fn perspective() -> EncryptionPerspective {
    use transportation::EncryptionPerspective::{Alice, Bob};
    match mode().as_str() {
        "reexec" => Bob,
        "server" => Bob,
        "serve-one" => Bob,
        "reverse-server" => Bob,
        _ => Alice,
    }
}
