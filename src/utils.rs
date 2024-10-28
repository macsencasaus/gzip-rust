use std::process;

pub const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const KNOWN_SUFFIXES: &[&str] = &[".gz", ".z", "taz", ".tgz", "-gz", "-z", "_z"];

pub fn help() {
    let help_msg = [
        "Compress or uncompress FILEs (by default, compress FILES in-place).",
        "",
        "Mandatory arguments to long options are mandatory for short options too.",
        "",
        #[cfg(feature = "O_BINARY")]
        "  -a, --ascii       ascii text; convert end-of-line using local conventions",
        "  -c, --stdout      write on standard output, keep original files unchanged",
        "  -d, --decompress  decompress",
        "  -f, --force       force overwrite of output file and compress links",
        "  -h, --help        give this help",
        "  -k, --keep        keep (don't delete) input files",
        "  -l, --list        list compressed file contents",
        "  -L, --license     display software license",
        "  -n, --no-name     do not save or restore the original name and timestamp",
        "  -N, --name        save or restore the original name and timestamp",
        "  -q, --quiet       suppress all warnings",
        #[cfg(not(feature = "NO_DIR"))]
        "  -r, --recursive   operate recursively on directories",
        "      --rsyncable   make rsync-friendly archive",
        "  -S, --suffix=SUF  use suffix SUF on compressed files",
        "      --synchronous synchronous output (safer if system crashes, but slower)",
        "  -t, --test        test compressed file integrity",
        "  -v, --verbose     verbose mode",
        "  -V, --version     display version number",
        "  -1, --fast        compress faster",
        "  -9, --best        compress better",
        #[cfg(feature = "LZW")]
        "  -Z, --lzw         produce output compatible with old compress",
        #[cfg(feature = "LZW")]
        "  -b, --bits=BITS   max number of bits per code (implies -Z)",
        "",
        "With no FILE, or when FILE is -, read standard input.",
        "",
        "Report bugs to <bug-gzip@gnu.org>.",
    ];

    println!("Usage: {} [OPTION]... [FILE]...", PACKAGE_NAME);
    for line in help_msg.iter() {
        println!("{}", line);
    }
}

pub fn license() {
    let license_msg = [
        "This is free software.  You may redistribute copies of it under the terms of",
        "the GNU General Public License <https://www.gnu.org/licenses/gpl.html>.",
        "There is NO WARRANTY, to the extent permitted by law.",
    ];

    println!("{} {}", PACKAGE_NAME, PACKAGE_VERSION);
    for line in license_msg.iter() {
        println!("{}", line);
    }
}

pub fn version() {
    license();
    println!();
    println!("Written by Macsen Casaus.");
}

pub fn try_help() {
    eprintln!("Try `{} --help' for more information.", PACKAGE_NAME);
}

pub enum ExitCode {
    Ok,
    Error,
    Warning,
}

pub fn exit_ok() -> ! {
    process::exit(ExitCode::Ok as i32)
}

pub fn exit_error() -> ! {
    process::exit(ExitCode::Error as i32)
}

pub fn exit_warning() -> ! {
    process::exit(ExitCode::Warning as i32)
}
