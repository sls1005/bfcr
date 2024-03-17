use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io::{Write, BufRead, BufReader, stdout, stderr};
use std::iter::FromIterator;
use std::process::Command;

const USAGE: &str = "
Usage: bfcr [options] <file>

options:
      -h, --help                  Show this help message and exit.

      -c, --compile               Compile into the target language (Rust). Don't do anything further.

      -o, --output <file>         The name of the output file(s). The name will be automatically suffixed with '.rs', so you don't need to add the suffix to it.

      -ic, --initial-cells        The number of cells at initial. (Default: the total number of '>' in the source file.)

      -b <flag>                   Pass a flag to the Rust compiler. (Can be flags split by whitespaces)

      -cmd                        Path to the Rust compiler, or the command to invoke it. (Default: 'rustc')

      -opt                        The optimization level. (Default: 2)

By default, this program generates a '.rs' file and then invokes 'rustc' to finish the compilation. It always overwrite the output file if it exists.
";

fn compile_bf(input_file_name: &str, output_file_name: &str, initial_number_of_cells: Option<usize>) -> bool {
    let code = HashMap::from([
        ('+', "cinc(&mut v, i,"),
        ('-', "cdec(&mut v, i,"),
        ('>', "pinc(&mut v, &mut i,"),
        ('<', "pdec(&mut i,"),
        (',', "if !rc(&mut v, i, &mut stdin_buf, &mut buf_p) {return};"),
        ('.', "wc(v[i]);"),
        ('[', "while v[i] != 0 {"),
        (']', "}"),
    ]);
    let mut functions = HashMap::from([
        ('+', (false, "
            fn cinc(v: &mut Vec<u8>, i: usize, x: u8) {
                v[i] = (v[i]).wrapping_add(x);
            }")
        ),
        ('-', (false, "
            fn cdec(v: &mut Vec<u8>, i: usize, x: u8) {
                v[i] = (v[i]).wrapping_sub(x);
            }")
        ),
        ('>', (false, "
            fn pinc(v: &mut Vec<u8>, i: &mut usize, x: usize) {
                if *i > usize::MAX - x {
                    panic!(\"Right bound reached.\");
                } else {
                    *i += x;
                    if *i >= v.len() {
                        v.resize(1 + *i, 0);
                    }
                }
            }")
        ),
        ('<', (false, "
            fn pdec(i: &mut usize, x: usize) {
                if *i < x {
                    panic!(\"Left bound reached.\");
                } else {
                    *i -= x;
                }
            }")
        ),
        (',', (false, "
            use std::io::stdin;
            fn rc(v: &mut Vec<u8>, i: usize, buf: &mut String, buf_p: &mut usize) -> bool {
                if *buf_p >= buf.len() {
                    buf.clear();
                    stdin().read_line(buf).expect(\"Couldn't read from stdin.\");
                    if buf.len() == 0 {
                        return false;
                    } else {
                        *buf_p = 0;
                    }
                }
                (*v)[i] = buf.as_bytes()[*buf_p];
                (*buf_p) += 1;
                return true;
            }")
        ),
        ('.', (false, "
            use std::io::{Write, stdout};
            fn wc(c: u8) {
                stdout().write_all(&[c]).expect(\"Couldn't write to stdout.\");
            }")
        ),
    ]);
    let input_file = File::open(input_file_name).expect(&*format!("[Error] Cannot open '{}'.", input_file_name));
    let mut output_file = File::create(output_file_name).expect(&*format!("[Error] Cannot create '{}'.", output_file_name));
    output_file.write_all(("
#[allow(unused_variables)]
#[allow(unused_mut)]
fn main() {
    let mut stdin_buf = String::new();
    let mut buf_p = 0usize;
    let mut v = init_cells();
    let mut i = 0usize;
    v.push(0);
").as_bytes()).expect("");
    let mut require_function = |c: char| {
        match functions.get_mut(&c) {
            Some(f) => {
                if !(f.0) {
                    f.0 = true;
                }
            },
            None => (),
        }
    };
    let mut current_cmd: Option<char> = None;
    let mut balance = 0;
    let mut n = 0 as u64;
    let mut x = match initial_number_of_cells {
        Some(ic) => ic,
        None => 0,
    };
    for line in BufReader::new(input_file).lines() {
        for c in line.unwrap().chars() {
            if code.keys().find(|x| c == **x).is_none() {
                continue;
            } else if c == '>' {
                if x < usize::MAX && initial_number_of_cells.is_none() {
                    x += 1;
                }
            }
            match current_cmd {
                Some(cmd) => {
                    if cmd == c && n < u64::MAX {
                        n += 1;
                        continue;
                    } else {
                        output_file.write_all((
                                (*code.get(&cmd).unwrap()).to_owned() + &*format!("{n}") + ");\n"
                            ).as_bytes()
                        ).expect("");
                        require_function(cmd);
                        current_cmd = None;
                        n = 0;
                    }
                },
                None => (),
            }
            match c {
                '+' | '-' | '<' | '>' => {
                    current_cmd = Some(c);
                    n += 1;
                },
                _ => {
                    if c == '[' {
                        balance += 1;
                    } else if c == ']' {
                        if balance == 0 {
                            stderr().write_all(b"Syntax error: Unmatched ']'.\n").expect("");
                            return false;
                        } else {
                            balance -= 1;
                        }
                    }
                    output_file.write_all((
                            (*code.get(&c).unwrap()).to_owned() + "\n"
                        ).as_bytes()
                    ).expect("");
                    require_function(c);
                },
            }
        }
    }
    match current_cmd {
        Some(cmd) => {
            output_file.write_all((
                    (*code.get(&cmd).unwrap()).to_owned() + &*format!("{n}") + ");\n"
                ).as_bytes()
            ).expect("");
            require_function(cmd);
        },
        None => (),
    }
    output_file.write_all(("}
        fn init_cells() -> Vec<u8> {
            Vec::<u8>::with_capacity(".to_owned() + &*format!("{x}")  + ")
        }").as_bytes()
    ).expect("");
    for k in functions.keys() {
        match functions.get(&k) {
            Some(f) => {
                if f.0 {
                    output_file.write_all(f.1.as_bytes()).expect("");
                }
            },
            _ => (),
        }
    }
    if balance != 0 {
        stderr().write_all(b"Syntax error: Unmatched '['.\n").expect("");
        return false;
    } else {
        return true;
    }
}

fn main() {
    let mut input_file_name = String::new();
    let mut output_file_name: Option<String> = None;
    let mut initial_number_of_cells: Option<usize> = None;
    let mut should_produce_executable = true;
    let mut cmd = String::from("rustc");
    let mut opt = String::from("2");
    let mut arguments = Vec::<&str>::new();
    let argv = Vec::from_iter(args());
    let argc = argv.len();
    let k = argc - 1;
    let mut i = 1 as usize;
    let mut a;
    if argc == 1 {
        stdout().write_all(USAGE.as_bytes()).expect("");
        return;
    }
    while i < argc {
        a = argv[i].as_str();
        match a {
            "-h" | "--help" => {
                stdout().write_all(USAGE.as_bytes()).expect("");
                return;
            },
            "-c" | "--compile" => {
                should_produce_executable = false;
            },
            "-o" | "--output" => {
                if i < k {
                    output_file_name = Some(argv[i+1].clone() + ".rs");
                    i += 2;
                    continue;
                } else {
                    stderr().write_all((
                            "Error: Expected a file name after '".to_owned() + &a + "'."
                        ).as_bytes()
                    ).expect("");
                    return;
                }
            },
            "-ic" | "--initial-cells" => {
                if i < k {
                    initial_number_of_cells = Some(argv[i+1].parse().unwrap());
                    i += 2;
                    continue;
                } else {
                    stderr().write_all((
                            "Error: Expected an integer after '".to_owned() + &a + "'."
                        ).as_bytes()
                    ).expect("");
                    return;
                }
            },
            "-b" => {
                if i < k {
                    arguments.append(
                        &mut Vec::from_iter(argv[i+1].split(' ').filter(|s| !s.is_empty()))
                    );
                    i += 2;
                    continue;
                } else {
                    stderr().write_all((
                            "Error: Expected a flag after '".to_owned() + &a + "'."
                        ).as_bytes()
                    ).expect("");
                    return;
                }
            },
            "-cmd" => {
                if i < k {
                    cmd = argv[i+1].clone();
                    i += 2;
                    continue;
                } else {
                    stderr().write_all((
                            "Error: Expected a path after '".to_owned() + &a + "'."
                        ).as_bytes()
                    ).expect("");
                    return;
                }
            },
            "-opt" => {
                if i < k {
                    opt = argv[i+1].clone();
                    i += 2;
                    continue;
                } else {
                    stderr().write_all((
                            "Error: Expected an integer or string after '".to_owned() + &a + "'."
                        ).as_bytes()
                    ).expect("");
                    return;
                }
            },
            _ => {
                input_file_name = String::from(a);
            },
        }
        i += 1;
    }
    let output_file_name1 = match output_file_name {
        Some(name) => name,
        None => match input_file_name.strip_suffix(".bf") {
            Some(s) => s.to_owned() + ".rs",
            None => input_file_name.clone() + ".rs",
        },
    };
    if ! compile_bf(&input_file_name, &output_file_name1, initial_number_of_cells) {
        return;
    }
    if should_produce_executable {
        opt = "opt-level=".to_owned() + &opt;
        arguments.push("-C");
        arguments.push(&opt);
        arguments.push(&output_file_name1);
        let status = Command::new(cmd.as_str()).args(arguments).status().expect((
                "Failed to execute the command '".to_owned() + &cmd + "'."
            ).as_str()
        );
        if ! status.success() {
            stderr().write_all((
                    "Error: Failed to compile '".to_owned() + &output_file_name1 + "'."
                ).as_bytes()
            ).expect("");
            return
        }
    }
}
