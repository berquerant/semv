mod cli;
mod mcp;
mod proc;
mod ver;

use crate::cli::Cli;
use crate::proc::{ProcessOptions, print_lines, process_versions, read_lines};
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let opt = Cli::parse();
    if opt.mcp {
        return mcp::run_mcp_server().await;
    }

    let options = ProcessOptions {
        sort: opt.sort,
        reverse_sort: opt.reverse_sort,
        filter_non_semver: opt.filter_non_semver,
        requirement: opt.version_requirement,
        verbose: opt.verbose,
    };
    print_lines(process_versions(read_lines(opt.targets), options));
    Ok(())
}
