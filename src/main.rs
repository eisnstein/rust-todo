use chrono::prelude::*;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::str::FromStr;

#[derive(Debug)]
struct Todo {
    id: u32,
    is_completed: bool,
    text: String,
    created_at: DateTime<Local>,
}

#[derive(Debug)]
struct Metadata {
    seq_id: u32,
}

#[derive(Debug, PartialEq, Eq)]
struct ParseTodoError;

#[derive(Debug, PartialEq, Eq)]
struct ParseMetadataError;

impl FromStr for Todo {
    type Err = ParseTodoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let elements: Vec<&str> = s.split(',').collect();

        if elements.len() != 4 {
            return Err(ParseTodoError);
        }

        let id = elements[0].parse::<u32>().unwrap();
        let created_at = elements[1].parse::<DateTime<Local>>().unwrap();
        let text = elements[2].to_string();
        let is_completed = elements[3].parse::<bool>().unwrap();

        Ok(Todo {
            id: id,
            created_at: created_at,
            text: text,
            is_completed: is_completed,
        })
    }
}

impl FromStr for Metadata {
    type Err = ParseMetadataError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("seq_id:") {
            return Err(ParseMetadataError);
        }

        let elements: Vec<&str> = s.split(':').collect();

        if elements.len() != 2 {
            return Err(ParseMetadataError);
        }

        let seq_id = elements[1].parse::<u32>().unwrap();

        Ok(Metadata { seq_id: seq_id })
    }
}

impl fmt::Display for Todo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{},{:?},{},{}",
            self.id, self.created_at, self.text, self.is_completed
        )
    }
}

impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "seq_id:{}", self.seq_id)
    }
}

fn main() -> Result<(), io::Error> {
    let mut stdout = io::stdout();
    let stdin = io::stdin();

    let mut todos: Vec<Todo> = Vec::new();
    let mut metadata = load_metadata();
    load_todos(&mut todos);

    loop {
        println!("What do you want to do?");
        println!("[1] Show all todos");
        println!("[2] Show all open todos");
        println!("[3] Create a new todo");
        println!("[4] Set a todo as complete");
        println!("[5] Delete a todo");
        println!("[6] Close");

        print!(">> ");
        stdout.flush()?;

        let mut input = String::new();
        stdin.read_line(&mut input)?;

        match input.trim() {
            "1" => show_all_todos(&todos),
            "2" => show_all_open_todos(&todos),
            "3" => {
                let new_todo = new_todo(&mut metadata);
                todos.push(new_todo);
            }
            "4" => set_todo_completed(&mut todos),
            "5" => delete_todo(&mut todos),
            _ => {
                save_todos(&metadata, &todos);
                break;
            }
        }
    }

    Ok(())
}

fn load_metadata() -> Metadata {
    let f = File::open("todos_db.txt").unwrap();
    let mut reader = BufReader::new(f);
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();

    line.trim().parse::<Metadata>().unwrap()
}

fn load_todos(todos: &mut Vec<Todo>) {
    // Read todos from db file
    let f = File::open("todos_db.txt").unwrap();
    let reader = BufReader::new(f);
    let mut count = 0;
    for line in reader.lines() {
        if count == 0 {
            count += 1;
            continue;
        }
        let t = line.unwrap().parse::<Todo>().unwrap();
        todos.push(t);
    }
}

fn save_todos(metadata: &Metadata, todos: &Vec<Todo>) {
    // Store todos in a file
    let mut f = File::create("todos_db.txt").unwrap();

    let todos_buf = todos
        .iter()
        .map(|t| t.to_string())
        .collect::<Vec<String>>()
        .join("\n");

    f.write(metadata.to_string().as_bytes()).unwrap();
    f.write(b"\n").unwrap();
    f.write(todos_buf.as_bytes()).unwrap();
}

fn show_all_todos(todos: &Vec<Todo>) {
    print_todos(&todos, false);
}

fn show_all_open_todos(todos: &Vec<Todo>) {
    print_todos(&todos, true);
}

fn new_todo(metadata: &mut Metadata) -> Todo {
    let mut input_todo = String::new();
    io::stdin().read_line(&mut input_todo).unwrap();

    metadata.seq_id += 1;

    Todo {
        id: metadata.seq_id,
        is_completed: false,
        text: input_todo.trim().into(),
        created_at: Local::now(),
    }
}

fn set_todo_completed(todos: &mut Vec<Todo>) {
    let mut input_todo_id = String::new();
    io::stdin().read_line(&mut input_todo_id).unwrap();

    let id = input_todo_id.trim().parse::<u32>().unwrap();

    let todo = todos.iter_mut().find(|t| t.id == id);

    match todo {
        Some(t) => t.is_completed = true,
        None => println!("Could not find Todo by that id"),
    }
}

fn delete_todo(todos: &mut Vec<Todo>) {
    let mut input_todo_id = String::new();
    io::stdin().read_line(&mut input_todo_id).unwrap();

    let id = input_todo_id.trim().parse::<u32>().unwrap();

    let todo_index = todos.iter().position(|t| t.id == id);

    match todo_index {
        Some(index) => {
            todos.remove(index);
        }
        None => println!("Could not find Todo by that id"),
    }
}

fn print_todos(todos: &Vec<Todo>, only_open_todos: bool) {
    let column_sizes = get_size_for_columns(&todos);

    println!("");
    for todo in todos {
        if only_open_todos && todo.is_completed == true {
            continue;
        }

        let created_at = todo.created_at.format("%d.%m.%Y");
        print!("{:>width$}", todo.id, width = column_sizes[0]);
        print!(" {:>width$}", created_at, width = column_sizes[1]);
        print!(" {:<width$}", todo.text, width = column_sizes[2]);
        print!(" {:>width$}", todo.is_completed, width = column_sizes[3]);
        println!();
    }
    println!("");
}

fn get_size_for_columns(todos: &Vec<Todo>) -> Vec<usize> {
    let mut column_sizes: Vec<usize> = Vec::new();
    let mut id_column_size = 0;
    let mut text_column_size = 0;

    for todo in todos {
        let id_str = todo.id.to_string();
        if id_str.len() > id_column_size {
            id_column_size = id_str.len();
        }

        if todo.text.len() > text_column_size {
            text_column_size = todo.text.len();
        }
    }

    column_sizes.push(id_column_size);
    column_sizes.push(10);
    column_sizes.push(text_column_size);
    column_sizes.push(4);

    column_sizes
}
