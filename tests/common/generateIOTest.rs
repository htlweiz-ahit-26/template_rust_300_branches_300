use std::env;
use std::fs::{OpenOptions, create_dir_all, exists, remove_file};
use std::io::{self, Read, Write};
use std::path;
use std::process::{Command, Stdio};
use std::thread;

const ERR_MSG_NO_PARENT: &str = "No parent!";
const ERR_MSG_PARSING_FAILED: &str = "Parsing Failed!";
const ERR_MSG_FILE_CREATION_ERROR: &str = "File Creation Error!";
const ERR_MSG_FILE_WRITE_ERROR: &str = "File Write Error!";
const ERR_MSG_ENCODING_ERROR: &str = "Encoding Error!";

const BREAK_FILE: &str = ".running";

fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: generateIOTest program_number test_number <additional args for program>");
        return;
    }
    if !exists("./tests/").unwrap() {
        println!("Use this program in the project basedir!");
        return;
    }
    let mut running = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(BREAK_FILE)
        .unwrap();
    write!(running, "1").expect(ERR_MSG_FILE_WRITE_ERROR);

    let me = args.remove(0);
    let me = path::Path::new(&me);
    eprintln!("This is me {:?}", me);

    let program_number = args.remove(0);
    eprintln!("This is the program_number {program_number}");
    let program_number: u8 = program_number.parse().expect(ERR_MSG_PARSING_FAILED);

    let test_number = args.remove(0);
    eprintln!("This is the test_number {test_number}");
    let test_number: u8 = test_number.parse().expect(ERR_MSG_PARSING_FAILED);

    println!("Path: {}", me.display());
    println!("Directory: {:?}", me.parent());
    println!("File: {:?}", me.file_name());

    let test_target_file_name = format!("program{program_number:02}");
    let test_target = path::Path::new(me.parent().expect(ERR_MSG_NO_PARENT));
    let test_target = test_target.join(&test_target_file_name);

    println!("Test_target: {}", test_target.display());

    let mut test_target = Command::new(test_target)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let mut test_target_stdin = test_target.stdin.take().unwrap();
    let mut test_target_stdout = test_target.stdout.take().unwrap();
    let mut test_target_stderr = test_target.stderr.take().unwrap();

    let test_dir = path::Path::new("./tests/io_tests")
        .join(&test_target_file_name)
        .join(format!("{test_number:02}"));

    create_dir_all(&test_dir).unwrap();

    let test_stdin = path::Path::new(&test_dir);
    let test_stdin = test_stdin.join("stdin.txt");

    let test_stdout = path::Path::new(&test_dir);
    let test_stdout = test_stdout.join("stdout.txt");

    let test_stderr = path::Path::new(&test_dir);
    let test_stderr = test_stderr.join("stderr.txt");

    thread::spawn(move || {
        let mut buffer = String::new();
        println!("Trying to write to file {:?}", test_stdin);
        let mut test_stdin = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(test_stdin)
            .expect(ERR_MSG_FILE_CREATION_ERROR);
        loop {
            if !exists(BREAK_FILE).unwrap() {
                eprintln!("in handler bye!");
                break;
            }
            io::stdin().read_line(&mut buffer).unwrap();
            write!(test_target_stdin, "{}", buffer).unwrap();
            write!(test_stdin, "{}", buffer).expect(ERR_MSG_FILE_WRITE_ERROR);
            test_stdin.flush().unwrap();
            buffer.clear();
            eprintln!("Buffer is: {:?}", buffer);
        }
    });

    let out_handle = thread::spawn(move || {
        let mut buffer = [0; 1024];
        let mut test_stdout = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(test_stdout)
            .expect(ERR_MSG_FILE_CREATION_ERROR);
        loop {
            if !exists(BREAK_FILE).unwrap() {
                eprintln!("out handler bye!");
                break;
            }
            let n = test_target_stdout.read(&mut buffer).unwrap();
            if n > 0 {
                io::stdout().write_all(&buffer[..n]).unwrap();
                write!(
                    test_stdout,
                    "{}",
                    String::from_utf8(buffer[..n].to_vec()).expect(ERR_MSG_ENCODING_ERROR)
                )
                .expect(ERR_MSG_FILE_WRITE_ERROR);
            }
            test_stdout.flush().unwrap();
        }
    });

    let err_handle = thread::spawn(move || {
        let mut buffer = [0; 1024];
        let mut test_stderr = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(test_stderr)
            .expect(ERR_MSG_FILE_CREATION_ERROR);
        loop {
            if !exists(BREAK_FILE).unwrap() {
                eprintln!("err handler bye!");
                break;
            }
            let n = test_target_stderr.read(&mut buffer).unwrap();
            if n > 0 {
                io::stderr().write_all(&buffer[..n]).unwrap();
                write!(
                    test_stderr,
                    "{}",
                    String::from_utf8(buffer[..n].to_vec()).expect(ERR_MSG_ENCODING_ERROR)
                )
                .expect(ERR_MSG_FILE_WRITE_ERROR);
            }
            test_stderr.flush().unwrap();
        }
    });

    let result = test_target.wait();
    match result {
        Ok(test) => eprintln!("Result is OK {:?}", test),
        Err(msg) => eprintln!("Result is Err with msg: {:?}", msg),
    }
    remove_file(BREAK_FILE).expect(ERR_MSG_FILE_WRITE_ERROR);
    out_handle.join().unwrap();
    err_handle.join().unwrap();
    eprintln!("Done!");
}
