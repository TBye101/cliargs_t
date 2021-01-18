/*
Parts todo:
command aliases
strong static assertions
debug runtime contracts
tests
better documentation
async command execution support
optional: log support
*/

use static_assertions::*;
use std::collections::HashMap;

const FLAG_PREFIX: char = '-';

/// Represents a flag for a command.
#[derive(Clone)]
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

    ///Returns general information about the command such as its name, help text, flags, and the flag's help information.
    fn get_information(&self) -> CommandInformation;
}
assert_obj_safe!(Command);

///Holds various information that is mainly utilized by the help command.
#[derive(Clone)]
pub struct CommandInformation {

    ///The name of the command.
    pub command_name: &'static str,

    ///The help description of the command.
    pub command_help: &'static str,

    ///The flags that the command supports or requires
    pub flags: std::vec::Vec<Flag>,
}
pub struct HelpCommand {
    ///Some general information about a command
    pub known_commands: std::vec::Vec<CommandInformation>
}

impl HelpCommand {

    pub fn new(commands: &std::vec::Vec<Box<dyn Command>>) -> HelpCommand {
        let mut known: std::vec::Vec<CommandInformation> = std::vec::Vec::with_capacity(commands.len() + 1);
        known.push(HelpCommand::get_info());

        //Grab the information about each command
        for command in commands {
            known.push(command.get_information());
        }

        return HelpCommand { 
            known_commands: known
        };
    }

    fn get_info() -> CommandInformation {
        return CommandInformation {
            command_name: "help",
            command_help: "Displays help information about commands and their flags.",
            flags:
                vec![
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
                ]
        }
    }

    ///Displays help information on all available commands.
    fn display_all_commands_help(&self) {
        for info in &self.known_commands {
            println!("{}, {}, # Flags: {}", info.command_name, info.command_help, info.flags.len());
        }
    }

    ///Displays help information about a specific command and lists its flags and their help information.
    fn display_command_help(&self, command: CommandInformation) {
        //Print a little header
        println!("'{}' help", command.command_name);
        println!("{}", command.command_help);
        println!("");
        
        //Print the available flags
        for flag in command.flags {
            println!("-{}, {}, required: {}", flag.identifier, flag.flag_help, flag.required);
        }
    }

    ///Displays help information about a specific command's flag.
    fn display_flag_help(&self, command: CommandInformation, flag_identifier: &String) {
        
        let flag_help = self.search_flag_help(command.flags, flag_identifier);
        if flag_help.is_some() {
            println!("{} -{}", command.command_name, flag_identifier);
            println!("{}", flag_help.unwrap());
        }
        else {
            println!("{} does not have a flag -{}", command.command_name, flag_identifier);
        }
    }

    fn search_flag_help(&self, flags: std::vec::Vec<Flag>, target_flag_identifier: &String) -> Option<&'static str> {
        for flag in flags {
            if flag.identifier == target_flag_identifier {
                return Some(flag.flag_help);
            }
        }
        return None;
    }

    fn get_command_info(&self, command_name: &String) -> Option<CommandInformation> {
        for command_info in self.known_commands.clone() {
            if command_info.command_name == command_name {
                return Some(command_info);
            }
        }
        return None;
    }
}

impl Command for HelpCommand {

    fn execute_command(&self, flags: std::collections::HashMap<std::string::String, std::string::String>) { 
        let command = flags.get("c");
        if command.is_some() {
            let command_info = self.get_command_info(command.unwrap());

            if command_info.is_some() {
                let flag = flags.get("f");
                if flag.is_some() {
                    //Display help for a specific command's flag
                    self.display_flag_help(command_info.unwrap(), flag.unwrap());
                }
                else {
                    //Display help about a specific command and list its flags and their help
                    self.display_command_help(command_info.unwrap());
                }
            } 
            else {
                println!("{} is not a registered command", command.unwrap());
            }
        }
        else {
            //Display help for all commands
            self.display_all_commands_help();
        }
    }

    fn get_information(&self) -> CommandInformation { 
        return HelpCommand::get_info();
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
            known.insert(command.get_information().command_name.to_string(), command);
        }
        
        return Commander {
            known_commands: known
        };
    }

    /// Parses the specified tokens for flags and their values.
    /// Returns the flags as a HashMap<String, String>
    fn parse_flags(&self, tokens: std::str::SplitWhitespace) -> Option<HashMap<String, String>> {
        let mut parsed_flags = HashMap::new();
        let mut flag = String::new();
        let mut flag_value;

        for token in tokens {
            if token.starts_with(FLAG_PREFIX) {
                flag = token.to_string().replace("-", "");
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
                let wrapped_stored_flag_value = parsed_flags.get_key_value(&flag);
                if wrapped_stored_flag_value.is_some() {
                    if wrapped_stored_flag_value.unwrap().1 == &String::new() {
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
                else {
                    println!("Expected a flag, instead found: {}", flag_value);
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
                    if self.verify_flags(&flags, command.get_information().flags) {
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