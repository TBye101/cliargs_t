/*
Parts todo:
help command
command aliases
strong static assertions
debug runtime contracts
tests
better documentation
async command execution support
optional: log support
publish crate

Pattern:
global_prefix command_name [OptionalArgs(<flag, value>)]

https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-path-dependencies
*/

use std::borrow::Borrow;
use contracts::*;
use static_assertions::*;
use std::collections::HashMap;

const FLAG_PREFIX: char = '-';

/// Represents a flag for a command.
pub struct Flag {
    
    ///This identifier should be the letter or phrase that signifies the flag. This should not include '-'.
    ///This should also not include '-h', as that is reserved for displaying help information.
    pub identifier: &'static str,
    
    ///Help text for this flag.
    pub flag_help: &'static str,

    ///Whether or not this flag is required for the command to be used.
    pub required: bool
}

///Implementors of this trait handle a specific command's execution.
pub trait Command {
    
    ///The implementation of this function should execute the command with the given flag information.
    fn execute_command(&self, flags: HashMap<String, String>);

    ///Returns general help text about the command.
    fn get_help(&self) -> &'static str;

    ///Returns the flags that are valid for the command.
    fn get_flags(&self) -> std::vec::Vec<Flag>;
    
    ///Returns the name of the command.
    fn get_name(&self) -> &'static str;
}
assert_obj_safe!(Command);

pub struct HelpCommand {
    /// <command_name<flag_identifier, flag_data>>
    pub known_commands: HashMap<String, HashMap<String, Flag>>
}

impl HelpCommand {

    pub fn new(commands: &std::vec::Vec<Box<dyn Command>>) -> HelpCommand {
        let mut known: HashMap<String, HashMap<String, Flag>> = HashMap::new();
        for command in commands {
            //Parse each command into <name, flags>
            let command_name = command.get_name();
            let flag_data = HelpCommand::parse_flags(command);
            known.insert(command_name.to_string(), flag_data);
        }

        return HelpCommand { 
            known_commands: known
        };
    }

    ///Parses a command's flags into the HashMap format <flag_name, flag_data>
    fn parse_flags(command: &Box<dyn Command>) -> HashMap<String, Flag> {
        let mut flag_data: HashMap<String, Flag> = HashMap::new();

        for flag in command.get_flags() {
            flag_data.insert(flag.identifier.to_string(), flag);
        }

        return flag_data;
    }

    ///Displays help information on all available commands.
    fn display_all_commands_help(&self) {
        
    }

    ///Displays help information about a specific command and lists its flags and their help information.
    fn display_command_help(&self) {

    }

    ///Displays help information about a specific command's flag.
    fn display_flag_help(&self) {

    }
}

impl Command for HelpCommand {

    fn execute_command(&self, flags: std::collections::HashMap<std::string::String, std::string::String>) { 
        let command = flags.get("c");
        if command.is_some() {
            let flag = flags.get("f");
            if flag.is_some() {
                //Display help for a specific command's flag
                self.display_flag_help();
            }
            else {
                //Display help about a specific command and list its flags and their help
                self.display_command_help();
            }
        }
        else {
            //Display help for all commands
            self.display_all_commands_help();
        }
    }

    fn get_help(&self) -> &'static str { 
        return "Displays help information about commands and their flags.";
    }

    fn get_flags(&self) -> std::vec::Vec<Flag> { 
        return vec![
            Flag {
                identifier: "c",
                flag_help: "Displays information about the specified command and its flags",
                required: false
            },
            Flag {
                identifier: "f",
                flag_help: "Displays information about a flag specific to the specified command",
                required: false
            }
        ];
    }

    fn get_name(&self) -> &'static str { 
        return "help";
    }
}


pub struct Commander<'a> {
    ///A map of <command_name, command>. Command names should be lowercase.
    pub known_commands: HashMap<String, &'a Box<dyn Command>>
}

impl<'a> Commander<'a> {

    pub fn new(commands: &'a mut std::vec::Vec<Box<dyn Command>>) -> Commander<'a> {
        //Construct the help command and register it
        let help = HelpCommand::new(&commands);
        commands.insert(0, Box::new(help));

        //Register the rest of the commands
        let mut known: HashMap<String, &Box<dyn Command>> = HashMap::with_capacity(commands.len());
        for command in commands {
            known.insert(command.get_name().to_string(), command);
        }
        
        return Commander {
            known_commands: known
        };
    }

    /// Parses the specified tokens for flags and their values.
    /// Returns the flags as a HashMap<String, String>
    #[debug_requires(tokens.clone().next().is_some())]
    fn parse_flags(&self, tokens: std::str::SplitWhitespace) -> Option<HashMap<String, String>> {
        let mut parsed_flags = HashMap::new();
        let mut flag = String::new();
        let mut flag_value;

        for token in tokens {
            if token.starts_with(FLAG_PREFIX) {
                flag = token.to_string();
                if parsed_flags.contains_key(&flag) {
                    //We shouldn't have a flag twice
                    println!("Flag {} has been discovered twice", flag);
                    return None;
                } 
                else {
                    //Add the discovered flag
                    parsed_flags.insert(flag.clone(), String::default());
                }
            }
            else {
                flag_value = token.to_string();
                let stored_flag_value = parsed_flags.get_key_value(&flag).unwrap().1;
                if stored_flag_value == &String::new() { //This probably won't work as they are both references?? Unless we did string aliasing...
                    //Set the value for the flag
                    parsed_flags.remove_entry(&flag);
                    parsed_flags.insert(flag.clone(), flag_value);
                }
                else {
                    //Flags shouldn't have two values
                    println!("Flag {} already has a value", flag);
                    return None;
                }
            }
        }
        return Some(parsed_flags);
    }

    ///Determines whether the provided flags meet a command's required flags and are valid.
    fn verify_flags(&self, parsed_flags: &HashMap<String, String>, required_flags: std::vec::Vec<Flag>) -> bool {
        for required_flag in required_flags {
            if required_flag.required {
                let had_flag = parsed_flags.contains_key(required_flag.identifier);
                if !had_flag {
                    println!("Missing a required flag: {}", required_flag.identifier);
                    return false;
                }
            }
        }
        return true;
    }

    ///Takes in a user's command input and parses and executes the command if everything is in order.
    pub fn handle_input(&self, input: String) {
        let mut tokens = input.trim().split_whitespace();
        let command_name_wrapped = tokens.nth(0);

        if command_name_wrapped.is_some() {
            //Parse the command's name
            let command_name = command_name_wrapped.unwrap().trim().to_lowercase();
            //Get the command if it is a known command
            let target_command = self.known_commands.get(&command_name);

            if target_command.is_some() {
                let command = target_command.unwrap();
                let found_flags: Option<HashMap<String, String>> = self.parse_flags(tokens);
                if found_flags.is_some() {
                    //We have our flags parsed, the command has been found and are ready to execute the command
                    let flags = found_flags.unwrap();
                    if self.verify_flags(&flags, command.get_flags()) {
                        command.execute_command(flags);
                    }
                }
            }
            else {
                println!("Failed to find the target command: {}", command_name);
            }
        }
        else {
            println!("Expected a command name");
        }
    }
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }
