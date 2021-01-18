# cliargs_t
A simple library for quickly creating a CLI application without dealing with user input parsing.

Pros | Cons |
| - | - |
Simple to Use | Missing Features More Complicated Libraries Have
Provides an Automatic Help Command | 

## User Experience
Users would utilize your commands like so if they were to utilize the example code below:

No flags: ```hello```

Flag: ```hello -a flag_value```

### Help Command
This library provides a already implemented help command!

```help```: Will provide list of commands, their help description, and the number of their flags.

```help -c command_name```: Will show command help description as well as detailed flag help

```help -c command_name -f flag_name```: Will show help information for the specific flag of the command specified.

## Create a Command
The example below shows how to create an instance of a command by implementing the Command trait.
```rust
struct HelloCommand {}

impl cliargs_t::Command for HelloCommand {
    
    fn execute_command(&self, flags: std::collections::HashMap<std::string::String, std::string::String>) {
        println!("Hello from the command!");
    }

    fn get_information(&self) -> cliargs_t::CommandInformation { 
        return cliargs_t::CommandInformation {
            command_name: "hello",
            command_help: "says hello",
            flags: vec![
                cliargs_t::Flag {
                    identifier: "a",
                    flag_help: "some example flag -a",
                    required: false
                }
            ]
        }
    }
}
```

## The Main Loop
This is an example of a main method that might funnel input through this library. In this example I used the [rustyline](https://crates.io/crates/rustyline) library to simplify my user input reading, but any user input method would work as long as it is passed in the appropriate format to the handle_input() method.

```rust
use std::io::Read;
use rustyline::error::ReadlineError;
use rustyline::Editor;

fn main() {
    let hello_command: Box<cliargs_t::Command> = Box::new(HelloCommand {});
    let mut commands = vec![
        hello_command
    ];
    let commander = cliargs_t::Commander::new(&mut commands);

    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                commander.handle_input(line);
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}
```