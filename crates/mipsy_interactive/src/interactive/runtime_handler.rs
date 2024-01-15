use crate::interactive::TargetAction;
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    rc::Rc,
    str::FromStr,
};

use super::{prompt, TargetWatch};
use colored::*;
#[cfg(feature = "raw_io")]
use mipsy_lib::runtime::{CloseArgs, OpenArgs, ReadArgs, WriteArgs};
use std::io::Write;
use text_io::try_read;

fn get_input<T>(name: &str, verbose: bool, line: bool) -> T
where
    T: FromStr + Display,
    <T as FromStr>::Err: Debug,
{
    let prompt: Box<dyn Fn()> = if verbose {
        Box::new(|| prompt::error_nonl(format!("bad input (expected {}), try again: ", name)))
    } else {
        Box::new(|| print!("[mipsy] bad input (expected {}), try again: ", name))
    };

    loop {
        let result: Result<T, _> = if line {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            input.parse().map_err(|_| ())
        } else {
            try_read!().map_err(|_| ())
        };

        match result {
            Ok(n) => return n,
            Err(_) => {
                (prompt)();
                std::io::stdout().flush().unwrap();
                continue;
            }
        };
    }
}

fn get_input_eof<T>(name: &str, verbose: bool) -> Option<T>
where
    T: FromStr + Display,
    <T as FromStr>::Err: Debug,
{
    let prompt: Box<dyn Fn()> = if verbose {
        Box::new(|| prompt::error_nonl(format!("bad input (expected {}), try again: ", name)))
    } else {
        Box::new(|| print!("[mipsy] bad input (expected {}), try again: ", name))
    };

    loop {
        let result: Result<T, _> = try_read!();

        match result {
            Ok(n) => return Some(n),
            Err(text_io::Error::Parse(leftover, _)) => {
                if leftover.is_empty() {
                    return None;
                }

                (prompt)();
                std::io::stdout().flush().unwrap();
                continue;
            }
            Err(_) => {
                (prompt)();
                std::io::stdout().flush().unwrap();
                continue;
            }
        };
    }
}

fn get_input_int(name: &str, verbose: bool) -> Option<i32> {
    let bad_input_prompt: &dyn Fn() = &|| {
        if verbose {
            prompt::error_nonl(format!("bad input (expected {}), try again: ", name))
        } else {
            print!("[mipsy] bad input (expected {}), try again: ", name)
        };
    };

    let too_big_prompt: &dyn Fn() = &|| {
        if verbose {
            prompt::error("bad input (too big to fit in 32 bits)")
        } else {
            println!("[mipsy] bad input (too big to fit in 32 bits)")
        }
    };

    loop {
        let result: Result<i128, _> = try_read!();

        match result {
            Ok(n) => match i32::try_from(n) {
                Ok(n) => return Some(n),
                Err(_) => {
                    (too_big_prompt)();
                    println!(
                        "[mipsy] if you want the value to be truncated to 32 bits, try {}",
                        n as i32
                    );
                    print!("[mipsy] try again: ");
                    std::io::stdout().flush().unwrap();
                    continue;
                }
            },
            Err(text_io::Error::Parse(leftover, _)) => {
                if leftover.is_empty() {
                    return None;
                }

                (bad_input_prompt)();
                std::io::stdout().flush().unwrap();
                continue;
            }
            Err(_) => {
                (bad_input_prompt)();
                std::io::stdout().flush().unwrap();
                continue;
            }
        };
    }
}

pub(crate) fn sys1_print_int(verbose: bool, val: i32) {
    if verbose {
        prompt::syscall_nl(1, format!("print_int: {}", val.to_string().green()));
    } else {
        print!("{}", val);
    }

    std::io::stdout().flush().unwrap();
}

pub(crate) fn sys2_print_float(verbose: bool, val: f32) {
    if verbose {
        prompt::syscall_nl(2, format!("print_float: {}", val.to_string().green()));
    } else {
        print!("{}", val);
    }

    std::io::stdout().flush().unwrap();
}

pub(crate) fn sys3_print_double(verbose: bool, val: f64) {
    if verbose {
        prompt::syscall_nl(3, format!("print_double: {}", val.to_string().green()));
    } else {
        print!("{}", val);
    }

    std::io::stdout().flush().unwrap();
}

pub(crate) fn sys4_print_string(verbose: bool, val: &[u8]) {
    let val = String::from_utf8_lossy(val);

    if verbose {
        prompt::syscall_nl(
            4,
            format!(
                "print_string: \"{}\"",
                val.escape_default().to_string().green()
            ),
        );
    } else {
        print!("{}", val);
    }

    std::io::stdout().flush().unwrap();
}

pub(crate) fn sys5_read_int(verbose: bool) -> i32 {
    if verbose {
        prompt::syscall(5, "read_int: ");
        std::io::stdout().flush().unwrap();
    }

    get_input_int("int", verbose).unwrap_or(0)
}

pub(crate) fn sys6_read_float(verbose: bool) -> f32 {
    if verbose {
        prompt::syscall(6, "read_float: ");
        std::io::stdout().flush().unwrap();
    }

    get_input_eof("float", verbose).unwrap_or(0.0)
}

pub(crate) fn sys7_read_double(verbose: bool) -> f64 {
    if verbose {
        prompt::syscall(7, "read_double: ");
        std::io::stdout().flush().unwrap();
    }

    get_input_eof("double", verbose).unwrap_or(0.0)
}

pub(crate) fn sys8_read_string(verbose: bool, max_len: u32) -> Vec<u8> {
    if verbose {
        prompt::syscall(5, format!("read_string [size={}]: ", max_len));
        std::io::stdout().flush().unwrap();
    }

    let input: String = get_input("string", verbose, true);

    // if input.len() > max_len as usize {
    //     prompt::error(format!("bad input (max string length specified as {}, given string is {} bytes), try again: ", max_len, input.len()));
    //     prompt::error_nonl("please try again: ");
    //     std::io::stdout().flush().unwrap();
    //     continue;
    // }

    // if input.len() == max_len as usize {
    //     prompt::error(format!("bad input (max string length specified as {}, given string is {} bytes -- must be at least one byte fewer, for NULL character), try again: ", max_len, input.len()));
    //     prompt::error_nonl("please try again: ");
    //     std::io::stdout().flush().unwrap();
    //     continue;
    // }

    input.into_bytes()
}

pub(crate) fn sys9_sbrk(verbose: bool, val: i32) {
    if verbose {
        prompt::syscall_nl(1, format!("sbrk: {}", val.to_string().green()));
    }
}

pub(crate) fn sys10_exit(verbose: bool) {
    if verbose {
        prompt::syscall_nl(10, "exit");
    }
}

pub(crate) fn sys11_print_char(verbose: bool, val: u8) {
    let val = val as char;

    if verbose {
        prompt::syscall_nl(
            11,
            format!("print_char: '{}'", val.escape_default().to_string().green()),
        );
    } else {
        print!("{}", val);
    }

    std::io::stdout().flush().unwrap();
}

pub(crate) fn sys12_read_char(verbose: bool) -> u8 {
    if verbose {
        prompt::syscall(5, "read_character: ");
        std::io::stdout().flush().unwrap();
    }

    let character: char = get_input_eof("character", verbose).unwrap_or('\0');
    character as u8
}

#[cfg(feature = "raw_io")]
pub(crate) fn sys13_open(verbose: bool, args: OpenArgs) -> i32 {
    use std::ffi::CString;

    let name = CString::new(args.path).expect("runtime should have removed \\0");

    if verbose {
        prompt::syscall_nl(
            13,
            format!("open({:?}, {}, {})", name, args.flags, args.mode),
        );
    }

    unsafe { libc::open(name.as_ptr(), args.flags as i32, args.mode) }
}

#[cfg(feature = "raw_io")]
pub(crate) fn sys14_read(verbose: bool, args: ReadArgs) -> (i32, Vec<u8>) {
    if verbose {
        prompt::syscall_nl(14, format!("read({}, {})", args.fd, args.len));
    }

    let mut vec = vec![0u8; args.len as usize];
    let read = unsafe { libc::read(args.fd as i32, vec.as_mut_ptr().cast(), args.len as usize) };
    (read as i32, vec)
}

#[cfg(feature = "raw_io")]
pub(crate) fn sys15_write(verbose: bool, args: WriteArgs) -> i32 {
    if verbose {
        prompt::syscall_nl(15, format!("write({}, {:?})", args.fd, args.buf));
    }

    unsafe { libc::write(args.fd as i32, args.buf.as_ptr().cast(), args.buf.len()) as i32 }
}

#[cfg(feature = "raw_io")]
pub(crate) fn sys16_close(verbose: bool, args: CloseArgs) -> i32 {
    if verbose {
        prompt::syscall_nl(16, format!("close({})", args.fd));
    }

    unsafe { libc::close(args.fd as i32) as i32 }
}

pub(crate) fn sys17_exit_status(verbose: bool, val: i32) {
    if verbose {
        prompt::syscall_nl(
            17,
            format!(
                "exit_status: {}",
                if val == 0 {
                    val.to_string().green()
                } else {
                    val.to_string().red()
                }
            ),
        );
    }
}

pub(crate) fn trap(_verbose: bool) {
    // TODO(zkol): This should provide actual diagnostics
    println!("{}\n", "[TRAP]".bright_red().bold());
}

pub(crate) fn breakpoint(
    label: Option<&str>,
    pc: u32,
    line_numbers: &HashMap<u32, (Rc<str>, u32)>,
) {
    let (filename, line_num, addr) = get_line_info(line_numbers, pc);

    println!(
        "{}{}{}\n",
        "\n[BREAKPOINT ".cyan().bold(),
        label.unwrap_or(&format!("{}:{}{}", filename, line_num, addr)),
        "]".cyan().bold()
    );
}

pub(crate) fn watchpoint(
    watchpoint: &TargetWatch,
    pc: u32,
    line_numbers: &HashMap<u32, (Rc<str>, u32)>,
) {
    let (filename, line_num, addr) = get_line_info(line_numbers, pc);
    println!(
        "{} {}:{}{}{} - {} was {}\n",
        "\n[WATCHPOINT".cyan().bold(),
        filename,
        line_num,
        addr,
        "]".cyan().bold(),
        watchpoint.target,
        match watchpoint.action {
            TargetAction::ReadOnly => "read from",
            TargetAction::WriteOnly | TargetAction::ReadWrite => "written to",
        }
    );
}

fn get_line_info(line_numbers: &HashMap<u32, (Rc<str>, u32)>, pc: u32) -> (&str, u32, String) {
    // get closest line number (pc may be in the middle of a pseudoinstruction)
    let mut lines = line_numbers
        .iter()
        .filter(|&(&addr, _)| addr <= pc)
        .collect::<Vec<_>>();
    lines.sort_unstable_by_key(|&(&addr, _)| addr);
    let line_num = lines.last().expect("there should be at least one line");
    let addr = if line_num.0 != &pc {
        // we're in the middle of a pseudoinstruction
        format!(" ({}{:08x})", "0x".yellow(), pc)
    } else {
        "".into()
    };

    // don't show filename when only 1 file is loaded
    let mut filenames = line_numbers.values().map(|(filename, _)| filename);
    let file = filenames.next().unwrap().as_ref();
    let filename = if !filenames.all(|f| f.as_ref() == file) {
        line_num.1 .0.as_ref()
    } else {
        ""
    };

    (filename, line_num.1 .1, addr)
}
