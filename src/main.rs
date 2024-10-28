mod utils;
use utils::{KNOWN_SUFFIXES, PACKAGE_NAME};

use atty::Stream;
use flate2::read::GzDecoder;
use flate2::Compression;
use flate2::GzBuilder;
use std::env;
use std::fs::{read_dir, remove_file, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

struct GzipOptions {
    to_stdout: bool,
    decompress: bool,
    force: bool,
    keep: bool,
    list: bool,
    no_name: bool,
    name: bool,
    quiet: bool,
    recursive: bool,
    resyncable: bool,
    test: bool,
    verbose: bool,
    level: u32,
}

fn parse_arg(arg: &String, options: &mut GzipOptions) {
    // Iterate over command-line arguments, skipping the first (program name).
    if arg.starts_with("--") {
        let flag = arg.trim_start_matches("--");

        match flag {
            "stdout" => options.to_stdout = true,
            "decompress" => options.decompress = true,
            "force" => options.force = true,
            "help" => {
                utils::help();
                utils::exit_ok();
            }
            "keep" => options.keep = true,
            "list" => options.list = true,
            "license" => {
                utils::license();
                utils::exit_ok();
            }
            "no-name" => options.no_name = true,
            "name" => options.name = true,
            "quiet" => options.quiet = true,
            "recursive" => options.recursive = true,
            "rsyncable" => options.resyncable = true,
            "test" => options.test = true,
            "verbose" => options.verbose = true,
            "version" => {
                utils::version();
                utils::exit_ok();
            }
            "fast" => options.level = 1,
            "slow" => options.level = 9,
            _ => {
                eprintln!("{}: invalid option -- {}", PACKAGE_NAME, arg);
                utils::try_help();
                utils::exit_error();
            }
        }
    } else if arg.starts_with('-') {
        for ch in arg.chars().skip(1) {
            match ch {
                'c' => options.to_stdout = true,
                'd' => options.decompress = true,
                'f' => options.force = true,
                'h' => {
                    utils::help();
                    utils::exit_ok()
                }
                'k' => options.keep = true,
                'l' => options.list = true,
                'L' => {
                    utils::license();
                    utils::exit_ok()
                }
                'n' => options.no_name = true,
                'N' => options.name = true,
                'q' => options.quiet = true,
                'r' => options.recursive = true,
                't' => options.test = true,
                'v' => options.verbose = true,
                'V' => {
                    utils::version();
                    utils::exit_ok()
                }
                '1' => options.level = 1,
                '9' => options.level = 9,
                _ => {
                    if let Some(num) = ch.to_digit(1) {
                        if (1..=9).contains(&num) {
                            options.level = num;
                        } else {
                            eprintln!("{}: invalid option -- {}", PACKAGE_NAME, arg);
                            utils::try_help();
                            utils::exit_error();
                        }
                    } else {
                        eprintln!("{}: invalid option -- {}", PACKAGE_NAME, arg);
                        utils::try_help();
                        utils::exit_error();
                    }
                }
            }
        }
    }
}

fn main() {
    let mut options = GzipOptions {
        to_stdout: false,
        decompress: false,
        force: false,
        keep: false,
        list: false,
        no_name: false,
        name: false,
        recursive: false,
        resyncable: false,
        verbose: false,
        quiet: false,
        test: false,
        level: 6,
    };

    let mut filenames: Vec<String> = Vec::new();

    for arg in env::args().skip(1) {
        if arg.starts_with("-") && arg.len() > 1 {
            parse_arg(&arg, &mut options);
            continue;
        }

        filenames.push(arg.clone());
    }

    if filenames.len() == 0 {
        treat_stdin(&options);
        return;
    }

    for filename in filenames {
        treat_file(&filename, &options);
    }
}

fn treat_file(filename: &str, options: &GzipOptions) {
    if filename == "-" {
        treat_stdin(&options);
        return;
    }
    let path = Path::new(filename);
    if path.is_dir() {
        if !options.recursive {
            if !options.quiet {
                eprintln!("{}: {} is a directory -- ignored", PACKAGE_NAME, filename)
            }
            utils::exit_warning();
        } else {
            for entry in read_dir(path).unwrap() {
                let entry = entry.unwrap();
                treat_file(&entry.path().to_str().unwrap(), options);
            }
        }
    } else {
        if options.decompress {
            decompress_file(filename, options)
        } else {
            compress_file(filename, options);
        }
        if !options.keep && !options.to_stdout {
            match remove_file(filename) {
                Ok(_) => {}
                Err(e) => {
                    if !options.quiet {
                        eprintln!("{}: {}", filename, e);
                    }
                    utils::exit_warning();
                }
            }
        }
    }
}

fn treat_stdin(options: &GzipOptions) {
    if !options.force && !options.to_stdout && !options.list && atty::is(Stream::Stdout) {
        if !options.quiet {
            eprintln!(
                "{}: compressed data not {} to a terminal. \
                Use -f to force compression.\n\
                For help, type: {} -h",
                PACKAGE_NAME, "written", PACKAGE_NAME
            )
        }
        utils::exit_error()
    }
    let mut input = io::stdin();
    let mut output = io::stdout();
    if options.decompress {
        decompress(&mut input, &mut output, options).unwrap();
    } else {
        compress(&mut input, &mut output, "", options).unwrap();
    }
}

fn compress_file(filename: &str, options: &GzipOptions) {
    let mut input = open_file(filename, options);

    let output_filename = format!("{}.gz", filename);
    let mut output = open_output(&output_filename, options);

    compress(&mut input, &mut output, &filename, options).unwrap();
}

fn decompress_file(filename: &str, options: &GzipOptions) {
    if !options.to_stdout && !has_valid_suffix(filename) {
        if options.verbose || (!options.quiet && !options.recursive) {
            eprintln!("{}: {}: unknown suffix -- ignored", PACKAGE_NAME, filename);
        }
        utils::exit_warning()
    }
    let input = open_file(filename, options);

    let mut decoder = GzDecoder::new(input);

    let output_filename: String;

    output_filename = decoder
        .header()
        .and_then(|header| if options.name { Some(header) } else { None })
        .and_then(|header| {
            header
                .filename()
                .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
        })
        .unwrap_or_else(|| trim_suffix(filename).to_string());

    let mut output = open_output(&output_filename, options);

    io::copy(&mut decoder, &mut output).unwrap();
}

fn compress<R: Read, W: Write>(
    input: R,
    output: W,
    input_filename: &str,
    options: &GzipOptions,
) -> io::Result<()> {
    let compression_level = match options.level {
        0..=9 => options.level,
        _ => 6, // Default level if invalid
    };

    let mut gz = GzBuilder::new();

    if !options.no_name {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();
        let current_time32 = u32::try_from(current_time).unwrap();
        gz = gz.filename(input_filename).mtime(current_time32);
    }
    let mut encoder = gz.write(output, Compression::new(compression_level));

    // Copy data from input to the encoder
    io::copy(&mut input.take(u64::MAX), &mut encoder)?;

    // Finish the compression
    encoder.finish()?;

    // Optionally handle other options here (like quiet, verbose, etc.)

    Ok(())
}

// just for writing to stdout
fn decompress<R: Read, W: Write>(
    input: R,
    mut output: W,
    _options: &GzipOptions,
) -> io::Result<()> {
    let mut decoder = GzDecoder::new(input);
    io::copy(&mut decoder, &mut output)?;
    Ok(())
}

fn open_file(filename: &str, options: &GzipOptions) -> File {
    File::open(filename).unwrap_or_else(|_| {
        if !options.quiet {
            eprintln!("{}: {}: No such file or directory", PACKAGE_NAME, filename);
        }
        utils::exit_error()
    })
}

fn open_output(output_filename: &str, options: &GzipOptions) -> Box<dyn Write> {
    if options.to_stdout {
        Box::new(io::stdout())
    } else {
        check_output_filename(&output_filename, options);
        Box::new(create_file(&output_filename, options))
    }
}

fn create_file(filename: &str, options: &GzipOptions) -> File {
    match File::create(filename) {
        Ok(f) => f,
        Err(e) => {
            if !options.quiet {
                eprintln!("{}: {}", filename, e);
            }
            utils::exit_error()
        }
    }
}

fn has_valid_suffix(filename: &str) -> bool {
    KNOWN_SUFFIXES
        .iter()
        .any(|suffix| filename.ends_with(suffix))
}

fn trim_suffix(filename: &str) -> &str {
    for suffix in KNOWN_SUFFIXES {
        if let Some(stripped) = filename.strip_suffix(suffix) {
            return stripped;
        }
    }
    filename
}

fn check_output_filename(filename: &str, options: &GzipOptions) {
    if Path::new(filename).exists() && !options.force {
        eprint!("{}: {} already exits;", PACKAGE_NAME, filename);
        eprint!(" do you wish to overwrite (y or n)? ");

        let mut buffer = [0; 1];
        match io::stdin().read_exact(&mut buffer) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("{}: {}", PACKAGE_NAME, e);
                utils::exit_error();
            }
        }

        let c = buffer[0] as char;

        if c == 'y' || c == 'Y' {
            return;
        } else {
            eprintln!("\tnot overwritten");
            utils::exit_warning();
        }
    }
}
