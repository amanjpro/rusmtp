use docopt::Docopt;
use dirs::home_dir;
use std::process::exit;


#[derive(Deserialize, Debug)]
pub struct Args {
    pub arg_recipients: Vec<String>,
    pub flag_account: Option<String>,
    pub flag_rusmtprc: String,
    flag_help: bool,
    flag_version: bool,
}

pub fn rusmtpd_usage(app_name: &str) -> String {
    let home_dir = home_dir().expect("Cannot find the home directory");
    let home_dir = home_dir.display();
    format!("
        {}

        Usage: {0}
               {0} --rusmtprc=<string>
               {0} --help
               {0} --version

        Options:
            --rusmtprc=<string>      Path to the rusmtprc [default: {}/.rusmtprc]
            -h, --help               Show this help.
            -v, --version            Show the version.
        ", app_name, home_dir)
}

pub fn rusmtpc_usage(app_name: &str) -> String {
    let home_dir = home_dir().expect("Cannot find the home directory");
    let home_dir = home_dir.display();
    format!("
        {}

        Usage: {0} [--rusmtprc=<string>] [--account=<string>] [--] <recipients>...
               {0} --help
               {0} --version

        Options:
            --account=<string>       The account on which the email should be sent.
                                     If none is provided, the default account would
                                     be chosen.
            --rusmtprc=<string>      Path to the rusmtprc [default: {}/.rusmtprc]
            -h, --help               Show this help.
            -v, --version            Show the version.
        ", app_name, home_dir)
}

pub fn process_args(app_name: &str, usage: &str) -> Args {

    let app_version = env!("CARGO_PKG_VERSION");

    let args: Args = Docopt::new(usage)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!("{}, v {}", app_name, app_version);
        exit(0);
    }

    if args.flag_help {
        println!("{}", usage);
        exit(0);
    }

    args
}



