use console::style;
use std::fs;
use std::path::Path;
use rusqlite::{Connection, Result};

// Define properties of a todo entry 
#[derive(Debug)]
// Use String for date_added and unsigned integer for is_done to match the available SQLite datatypes
pub struct Task {
    pub id: i32,
    pub name: String,
    pub date_added: String,
    pub is_done: u8,
}

// Use impl to define methods for the Task struct
impl Task {
    // Constructor for a new Task instance
    pub fn new(id: i32, name: String, date_added: String, is_done: u8) -> Task {
        Task { 
            id, 
            name, 
            date_added, 
            is_done 
        }
    }

    // Add a new Task to the database
    pub fn add(conn: &Connection, name: &str) -> Result<()> {
        // Insert a new row into the tasks table
        conn.execute(
            "INSERT INTO tasks (name) VALUES (?)",
            // The ? placeholder is used to avoid SQL injection attacks
            // The value of name will be inserted into the query in place of the ?
            // The values must be passed as a reference & slice
            &[name],
        )?;
        Ok(())
    }

    // List all tasks in the database
    pub fn list(conn: &Connection, sort_by_status: bool) -> Result<Vec<Task>> {
        // Set the sql query to sort by status if sort_by_status is true or by id if it is false
        let sql = if sort_by_status {
            "SELECT * FROM tasks ORDER BY is_done, id"
        } else {
            "SELECT * FROM tasks ORDER BY id"
        };
        // Takes a SQL query and prepares it for execution
        // stmt is a prepared statement - a precompiled SQL statement that can be executed multiple times with different parameters
        let mut stmt = conn.prepare(sql)?;
        // query_map executes the SQL query associated with the prepared statement
        // query_map returns an iterator over the rows returned by the query
        // query_map takes a closure that will be called for each row returned by the query
        // the closure creates a Task object from the values in the row
        // this is done for each row returned by the query
        // it is done lazily, which means that the rows are fetched and mapped one by one
        // as needed, outside of query_map, which is why no loop is explicitly needed in query_map
        let task_iter = stmt.query_map((), |row| {
            Ok(Task::new(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })?;
        // Collect the results of the query_map iterator into a Vec<Task>
        let mut tasks = Vec::new();
        // consume the iterator: fetch the row, map it to a Task, and add it to the tasks Vec
        for task in task_iter {
            tasks.push(task?);
        }
        Ok(tasks)
    }

    // Prints a list of task objects
    pub fn print_list(tasks: Vec<Task>) -> Result<()> {
        for task in tasks {
            // mark and color-code the status of the task
            let status = if task.is_done == 1 {
                style("DONE").green()
            } else {
                style("PENDING").red()
            };
            // expected that the task id does not exceed 4 characters
            // > aligns the text to the right, < aligns the text to the left
            println!(
                "{:>4} | {:<44} | {:<8} {}",
                style(task.id).cyan().bright(),
                style(truncate(&task.name, 44)).bright(),
                status,
                style(task.date_added).dim(),
            );
        }
        Ok(())
    }

    // Toggle the status of a task
    pub fn toggle(conn: &Connection, id: i32) -> Result<()> {
        // Prepare a statement to update the is_done column of a specific task
        let sql = "UPDATE tasks SET is_done = 1 - is_done WHERE id = ?";
        let rows_affected = conn.execute(sql, [id])?;
        // If no rows were affected, print the task with the given id was not found
        // Otherwise, print that the task was toggled
        if rows_affected == 0 {
            println!("No task found with id: {}", id);
        } else {
            println!("Toggled task with id: {}", id);
        }
        Ok(())
    }

    // Reset the database
    pub fn reset(conn: &Connection) -> Result<()> {
        // Delete all rows from the tasks table - no parameters are needed
        conn.execute("DELETE FROM tasks", [])?;
        // reset the autoincrement counter for the tasks table
        conn.execute("DELETE FROM sqlite_sequence WHERE name='tasks'", [])?;
        Ok(())
    }

    // Remove a task from the database
    pub fn rm(conn: &Connection, id: i32) -> Result<()> {
        // Prepare a statement to remove a task with a specific id
        let sql = "DELETE FROM tasks WHERE id = ?";
        let rows_affected = conn.execute(sql, [id])?;
        if rows_affected == 0 {
            println!("No task found with id: {}", id);
        } else {
            println!("Removed task with id: {}", id);
        }
        Ok(())
    }
}

// Truncate a str and adds an ellipsis if needed
// Takes in an input string and a maximum length
pub fn truncate(input: &str, max: i32) -> String {
    // type conversion from i32 to usize
    let max_len: usize = max as usize;
    if input.len() > max_len {
        // slice the input string to the maximum length - 3 to make room for the ellipsis
        let truncated = &input[..(max_len - 3)];
        // format the truncated string with an ellipsis
        return format!("{}...", truncated);
    };
    // return the original string if it is shorter than or equal to the maximum length
    input.to_string()
}

pub fn help() -> Result<()> {
    let help_title = "\nAvailable commands:";
    // r#"..."#; used to create a raw string literal
    // Maintains formatting, indentation, and line breaks
    let help_text = r#"
        - add [TASK]
            Adds new task(s)
            Example: todo add "Build a tree"

        - list
            Lists all tasks
            Example: todo list

        - toggle [ID]
            Toggles the status of a task (Done/Pending)
            Exampple: todo toggle 2
            
        - rm [ID]
            Removes a task
            Example: todo rm 4

        - sort
            Sorts completed and uncompleted tasks

        - reset
            Deletes all tasks
    "#;

    println!("{}", style(help_title).magenta().bright());
    println!("{}", style(help_text).cyan().bright());

    Ok(())
}

// Returns the user's home directory as a string
fn get_home() -> String {
    dirs::home_dir().expect("Could not determine the user's home directory.")
                    .to_str().expect("Could not convert the home directory to a string.")
                    .to_string()   
}

// Creates the folder where the DB should be stored if it doesn't exist
pub fn verify_db_path(db_folder: &str) -> Result<()> {
    // Tries to create a Path object from the db_folder string
    if !Path::new(&db_folder).exists() {
        // If the folder does not exist, try to create it
        match fs::create_dir_all(&db_folder) {
            Ok(_) => println!("Created the database folder at: {}", db_folder),
            Err(e) => panic!("Could not create the database folder: {}", e),
        }
    }

    Ok(())
}

// Creates tables if they do not exist
pub fn verify_db(conn: &Connection) -> Result<()> {
    // Create the table if it does not exist
    // AUTOINCREMENT will set the id, or primary key, to be 1 if it is the first row inserted
    // Otherwise, it will increment the id of the last inserted row by 1
    // The is_done column will be set to 0 by default, as the task is not done when it is added
    // The date_added column will be set to the current timestamp by default
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER NOT NULL,
            name TEXT NOT NULL,
            date_added TEXT NOT NULL DEFAULT current_timestamp,
            is_done INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (id AUTOINCREMENT)
        )",
        [], // no parameters for this query
    )?;

    Ok(())
}

// Returns a connection, creating the database if needed
pub fn get_connection() -> Result<Connection> {
    // Get the db folder
    let db_folder = get_home() + "/" + "tasks_db/";
    // Since db_folder is on the left side of the + operator, it will be moved and no longer be available
    // To avoid this, clone db_folder
    let db_file_path = db_folder.clone() + "tasks.sqlite";
    // Verify that the path to the database folder exists
    verify_db_path(&db_folder)?;
    // Open a connection to the database
    let conn = Connection::open(&db_file_path)?;
    // Verify that the database contains the expected table
    verify_db(&conn)?;
    Ok(conn)
}