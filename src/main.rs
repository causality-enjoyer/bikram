use std::process::ExitCode;

fn main() -> ExitCode {
    match bikram::cli::run(std::env::args().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprint!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
