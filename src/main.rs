use std::env;
use std::fs::{self, File};
use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

const HISTORY_NAME: &'static str = "er_history";
const TEMP_HISTORY_NAME: &'static str = "er_history_temp";

fn main() {
    print!("er - executable runner v{}\n\n", env!("CARGO_PKG_VERSION"));

    let mut history_path = match env::current_exe().map(|mut p| {
        p.pop();
        p.push(HISTORY_NAME);
        p
    }) {
        Ok(p) => p,
        Err(e) => {
            eprintln!(
                "Could locate executable path. Using working directory.\n{}",
                e
            );
            PathBuf::from(".")
        }
    };

    let mut should_not_save_history = false;

    let mut history = {
        let result = File::open(&history_path)
            .map(BufReader::new)
            .and_then(|mut f| {
                let mut v = Vec::with_capacity(f.by_ref().lines().count());

                f.seek(SeekFrom::Start(0))?;
                for line in f.lines() {
                    let line = line?;
                    v.push(line);
                }

                Ok(v)
            });

        match result {
            Ok(v) => v,
            Err(e) => {
                should_not_save_history = false;
                eprintln!(
                    "Could not read history. Saving history is disabled for this session.\n{}",
                    e
                );
                Vec::new()
            }
        }
    };

    command_loop(&mut history);

    if should_not_save_history {
        return;
    }

    history_path.pop();
    history_path.push(TEMP_HISTORY_NAME);

    let result = File::create(&history_path).and_then(|f| {
        let mut writer = BufWriter::new(f);

        for line in history.iter() {
            writer.write(line.as_bytes())?;
        }

        writer.flush()
    });

    match result {
        Ok(()) => {
            let mut target_path = history_path.clone();
            target_path.pop();
            target_path.push(HISTORY_NAME);

            if let Err(e) = fs::rename(history_path, target_path) {
                eprintln!("Could not rename history file.\n{}", e);
            }
        }
        Err(e) => {
            eprintln!("Could not save history.\n{}", e);
        }
    }
}

fn push_history(history: &mut Vec<String>, mut line: String) {
    //remove all but the last newline
    let mut index = None;
    let mut last_char = 'a';
    for (i, c) in line.chars().enumerate() {
        if (last_char != '\r' || last_char == '\n') && (c == '\r' || c == '\n') {
            index = Some(i);
        }
        last_char = c;
    }

    if last_char == '\r' || last_char == '\n' {
        if let Some(index) = index {
            line.truncate(index + 1);
            //in case it's a '\r'
            line.pop();
            line.push('\n');
        }
    }

    if line.len() > 0 {
        history.push(line);
    }
}

fn command_loop(history: &mut Vec<String>) {
    loop {
        let current_dir = env::current_dir().unwrap_or_default();

        print!("{}>", current_dir.display());
        // need to explicitly flush this to ensure it prints before read_line
        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        push_history(history, input.clone());

        // read_line leaves a trailing newline, which trim removes
        // this needs to be peekable so we can determine when we are on the last command
        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_command = None;

        while let Some(command) = commands.next() {
            // everything after the first whitespace character is interpreted as args to the command
            let mut parts = command.trim().split_whitespace();
            let command = if let Some(command) = parts.next() {
                command
            } else {
                previous_command = None;
                continue;
            };
            let args = parts;

            match command {
                "cd" => {
                    // default to '/' as new directory if one was not provided
                    let new_dir = args.peekable().peek().map_or("/", |x| *x);
                    let root = Path::new(new_dir);
                    if let Err(e) = env::set_current_dir(&root) {
                        eprintln!("{}", e);
                    }

                    previous_command = None;
                }
                "echo" => {
                    let mut sep = "";
                    for arg in args {
                        print!("{}{}", sep, arg);
                        sep = " ";
                    }
                    print!("\n");

                    // need to explicitly flush this to ensure it prints before the next iteration
                    stdout().flush().unwrap();
                }
                "exit" => return,
                command => {
                    let stdin = previous_command.map_or(Stdio::inherit(), |output: Child| {
                        Stdio::from(output.stdout.unwrap())
                    });

                    let stdout = if commands.peek().is_some() {
                        // there is another command piped behind this one
                        // prepare to send output to the next command
                        Stdio::piped()
                    } else {
                        // there are no more commands piped behind this one
                        // send output to shell stdout
                        Stdio::inherit()
                    };

                    let output = Command::new(command)
                        .args(args)
                        .stdin(stdin)
                        .stdout(stdout)
                        .spawn();

                    match output {
                        Ok(output) => {
                            previous_command = Some(output);
                        }
                        Err(e) => {
                            previous_command = None;
                            eprintln!("{}", e);
                        }
                    };
                }
            }
        }

        if let Some(mut final_command) = previous_command {
            // block until the final command has finished
            final_command.wait().unwrap();
        }
    }
}
