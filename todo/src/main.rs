extern crate todo;
use console::style;
use dialoguer::Confirm;
use rusqlite::Result;
use std::env;

use todo::*;

#[allow(unused)] // Remove later
fn main() -> Result<()> {
    // env::args() returns an iterator over the command-line arguments
    // collect() consumes said iterator to collect its elements into a Vec<String> / collection
    let args: Vec<String> = env::args().collect();
    
    // Get a connection to the DB
    // If it fails it returns the error, but if it works conn will be assigned properly
    let conn = get_connection()?;
    
    // If no arguments have been passed in (args only contains the program path)
    if args.len() == 1 {
        println!("No arguments were passed!");
        help();
        // Exits with a non-zero status to indicate abnormal termination
        std::process::exit(1);
    }

    // Parse command line arguments into commands and suffixes
    // For example, "add Write a Tutorial" -> command is add, rest is suffix

    // The first argument is the program path, so the command is the second argument
    let command = &args[1];
    
    // The third argument onwards is the suffix
    // & creates a reference to the resulting string, allowing suffix to be a reference to the joined string and avoids unnecessary copying
    // args[2..] accesses a slice of the args vector, beginning from index 2
    // iter() converts the slice into an iterator
    // cloned() creates a new iterator where every element is cloned in order to create owned values (because join requres owned strings)
    // collect::<Vec<_>> collects the iterator into a vector (Vec) where the type of each element is inferred (_)
    let suffix = &args[2..].iter().cloned().collect::<Vec<_>>().join(" ");

    // Commands are inferred to be type &String so use as_str to convert to &str
    // Match will match against string literals &str for efficiency
    match command.as_str() {
        // If the command is "add" and the suffix is empty, print help and exit
        // Otherwise, add the task to the DB
        "add" => {
            // If no task name is provided, print help and exit
            if suffix.as_str().is_empty() {
                help()?;
                std::process::exit(1);
            }
            // Otherwise, add the task to the DB
            else {
                Task::add(&conn, suffix.as_str())?;
            }
            Ok(())
        }
        "list" => {
            // retrieve a list of tasks from the database in the form of a Vec<Task>
            let tasks = Task::list(&conn, false)?;
            if tasks.is_empty() {
                println!("No tasks found.");
            }
            else {
                // print the list of tasks
                println!("To-Do List (sorted by id):");
                Task::print_list(tasks)?;
            }
            Ok(())
        }
        "mark" => {
            // If no task id is provided or the task id is not a number, print help and exit
            if args.len() < 3 || !args[2].parse::<i32>().is_ok() {
                help()?;
                std::process::exit(1);
            } else {
                // Otherwise, toggle the task status
                // unwrap() will try to return the parsed integer
                // If it fails, it will panic and print an error message
                Task::toggle(&conn, args[2].parse::<i32>().unwrap())?;
            }
            Ok(())
        }
        "reset" => {
            let confirmation = Confirm::new()
                .with_prompt(
                    style("Are you sure you want to delete all tasks?")
                        .bright()
                        .red()
                        .bold()
                        .to_string()
                )
                .interact();

            match confirmation {
                Ok(true) => {
                    Task::reset(&conn)?;
                    println!("Database reset. All tasks have been deleted.");
                }
                Ok(false) => println!("Reset aborted."),
                Err(_) => println!("Error processing input."),
            }
            Ok(())
        }
        "rm" => {
            // If no task id is provided or the task id is not a number, print help and exit
            if args.len() < 3 || !args[2].parse::<i32>().is_ok() {
                help()?;
                std::process::exit(1);
            } else {
                // Otherwise, remove the task
                Task::rm(&conn, args[2].parse::<i32>().unwrap())?;
            }
            Ok(())
        }
        "sort" => {
            Ok(())
        }
        "help" | "--help" | "-h" | _ => help(),
    }?;

    Ok(())
}
