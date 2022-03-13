use std::process::{Child, Command, Stdio};
use std::io::{stdin, stdout, Write};
use configparser::ini::Ini;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::env;

#[derive(Debug)]
struct Prompt {
    prompt:String,
    home:String,
}

fn def_Config() -> Prompt {
    let home = std::env::var("HOME").unwrap();
    let path = PathBuf::from(&home).join(".tshrc"); 
    match std::fs::File::open(&path) {
        Ok(_file) => {
            let mut config = Ini::new();
            let map = config.load(path).unwrap();
            let prompt = config.get("Config", "PS1").unwrap();
            Prompt{prompt,home}
        }
        Err(_er) => {
            println!("[TSH] Config missing. Created at $HOME/.tshrc");
            let mut file = File::create(path).unwrap();
            file.write(b"[Config]\nPS1 = $");
            let prompt = def_Config().prompt;
            Prompt{prompt,home}
      } 
    }
}

fn main() {
    loop {
        let ps1 = def_Config();
        print!("{} ", ps1.prompt);
        stdout().flush();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        // Split input as whitespace and pass anything after as args.
        let mut commands = input.trim().split(" | ").peekable();
        let mut previous_command = None;

        while let Some(command) = commands.next() {

            let mut parts = command.trim().split_whitespace();
            let command = parts.next().unwrap();
            let args = parts;

            match command {
                "cd" => {
                    let home = std::env::var("HOME").unwrap();
                    let new_dir = args.peekable().peek().map_or(home, |x| (*x).to_string());
                    let root = Path::new(&new_dir);
                    if let Err(e) = env::set_current_dir(&root) {
                        eprintln!("{}", e);
                    }
                    previous_command = None;
                },
                "exit" => return,
                command => {
                    let stdin = previous_command
                        .map_or(
                            Stdio::inherit(),
                            |output: Child| Stdio::from(output.stdout.unwrap())
                        );
                    let stdout = if commands.peek().is_some() {
                        Stdio::piped()
                    } else {
                        Stdio::inherit()
                    };
                    let output = Command::new(command)
                    .args(args)
                    .stdin(stdin)
                    .stdout(stdout)
                    .spawn();

                    match output {
                        Ok(output) => { previous_command = Some(output); },
                        Err(e) => {
                            previous_command = None;
                            eprintln!("{}", e);
                        },
                    };
              }
        }
    }
    if let Some(mut final_command) = previous_command {
        final_command.wait();
    }
  }
}
