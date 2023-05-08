use std::fs::File;
use std::io::{self, Write, BufRead};
use std::env::args;
use std::process;

use ncurses::*;
extern crate ncurses;

/* Constants. */
const REGULAR_PAIR: i16 = 0;
const HIGHLIGHT_PAIR: i16 = 1;

/* Types. */
type Id = usize;

#[derive(Default)]
struct Ui {
    list_curr: Option<Id>,
    row: usize,
    col: usize
}

impl Ui {
    fn begin(&mut self, row: usize, col: usize) {
        self.row = row;
        self.col = col;
    }
    fn begin_list(&mut self, id: Id) {
        /* Prevent from creating  nested lists. */
        assert!(self.list_curr.is_none(), "Nested lists are not allowed.");
        self.list_curr = Some(id);
    }
    fn list_element(&mut self, label: &str, id: Id) -> bool {
        /* Prevent running list_element outside of list. */
        let id_curr = self.list_curr.expect("Not allowed to create list elements outside of lists.");
        let pair = {
            if id_curr == id {
                HIGHLIGHT_PAIR
            } else {
                REGULAR_PAIR
            }
        };
        self.label(label, pair);
        return false;
    }
    fn label(&mut self, text: &str, pair: i16) {
        mv(self.row as i32, self.col as i32);
        attron(COLOR_PAIR(pair));
        addstr(text);
        attroff(COLOR_PAIR(pair));
        self.row += 1;
    }
    fn end_list(&mut self) {
        self.list_curr = None;
    }
    fn end(&mut self) {
    }
}

/* Status of view mode. */
#[derive(Debug)]
enum Status {
    Todo,
    Done
}

impl Status {
    fn toggle(&self) -> Self {
        match self {
            Status::Todo => Status::Done,
            Status::Done => Status::Todo
        }
    }
}

/* Parse item from text file. */
fn parse_item(line: &str) -> Option<(Status, &str)> {
    let todo_prefix = "TODO: ";
    let done_prefix = "DONE: ";
    if line.starts_with(todo_prefix) {
        return Some((Status::Todo, &line[todo_prefix.len()..]))
    }
    if line.starts_with(done_prefix) {
        return Some((Status::Done, &line[done_prefix.len()..]))
    }
    return None;
}

/* Movements helpers. */
fn list_up(list_curr: &mut usize) {
    if *list_curr > 0 {
        *list_curr -= 1;
    }
}

fn list_down(list: &Vec<String>, list_curr: &mut usize) {
    if *list_curr + 1 < list.len() {
        *list_curr += 1;
    }
}

fn list_transfer(list_dst: &mut Vec<String>, list_src: &mut Vec<String>, list_src_curr: &mut usize) {
    if *list_src_curr < list_src.len() {
        /* Remove todo from list_src, and add to list_dst. */
        list_dst.push(list_src.remove(*list_src_curr));
        /* Prevent from selecting invisible todo */
        if *list_src_curr >= list_src.len() && list_src.len() > 0 {
            *list_src_curr = list_src.len() - 1;
        }
    }
}

fn load_state(todos: &mut Vec<String>, dones: &mut Vec<String>, file_path: &str) {
    let file = File::open(file_path).unwrap();
    for (index, line) in io::BufReader::new(file).lines().enumerate() {
        match parse_item(&line.unwrap()) {
            Some((Status::Todo, title)) => todos.push(title.to_string()),
            Some((Status::Done, title)) => dones.push(title.to_string()),
            None => {
                eprintln!("{}:{}: ERROR: ill-formed item line", file_path, index + 1);
                process::exit(1);
            }
        }
    }
}

fn save_state(todos: &Vec<String>, dones: &Vec<String>, file_path: &str) {
    let mut file = File::create(file_path).unwrap();
    for todo in todos.iter() {
        writeln!(file, "TODO: {}", todo).unwrap();
    }
    for done in dones.iter() {
        writeln!(file, "DONE: {}", done).unwrap();
    }
}

fn main () {

    let mut args = args();
    args.next().unwrap();
    let file_path = match args.next() {
        Some(file_path) => file_path,
        None => {
            eprintln!("Usage: rust-todo-cli <file-path>");
            eprintln!("ERROR: file path is not provided");
            process::exit(1);
        }
    };

    /* Application state. */
    let mut quit= false;
    let mut todos = Vec::<String>::new();
    let mut dones = Vec::<String>::new();
    let mut todo_curr: usize = 0;
    let mut done_curr: usize = 0;     

    /* Load state. */
    load_state(&mut todos, &mut dones, &file_path);

    /* Init ncurses. */
    initscr();

    /* Disable cursor and keys echo. */
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    /* Init colors. */
    start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(HIGHLIGHT_PAIR, COLOR_BLACK, COLOR_WHITE);

    /* Create default interface. */
    let mut tab = Status::Todo;
    let mut ui = Ui::default();

    /* Event loop. */
    while !quit {
        erase();

        /* Start the interface. */
        ui.begin(0, 0);

        {
            /* Match view tab. */
            match tab {
                Status::Todo => {
                    ui.label("[TODO] DONE ",REGULAR_PAIR);
                    ui.label("------------",REGULAR_PAIR);

                    ui.begin_list(todo_curr);
        
                    for (index, todo) in todos.iter().enumerate() {
                       ui.list_element(&format!("- [] {}", todo), index);
                    }
            
                    ui.end_list();
                }
                Status::Done => {
                    ui.label(" TODO [DONE]",REGULAR_PAIR);
                    ui.label("------------",REGULAR_PAIR);

                    ui.begin_list(done_curr);

                    for (index, done) in dones.iter().enumerate() {
                        ui.list_element(&format!("- [x] {}", done), index);
                    }
                    
                    ui.end_list();
                }
            }
    
            ui.end();
     
            /* Update the screen. */
            refresh();
    
            /* Wait for a key press. */
            let key = getch();
    
            /* Handle input from the user. */
            match key as u8 as char {
                'q' => quit = true,
                'w' => {
                    match tab {
                        Status::Todo => list_up(&mut todo_curr),
                        Status::Done => list_up(&mut done_curr)
                    }
                }
                's' => {
                    match tab {
                        Status::Todo => list_down(&todos, &mut todo_curr),
                        Status::Done => list_down(&dones, &mut done_curr)
                    }
                }
                '\n' => match tab {
                    Status::Todo => {
                        list_transfer(&mut dones, &mut todos, &mut todo_curr);
                    }
                    Status::Done => {
                        list_transfer(&mut todos, &mut dones, &mut done_curr);
                    }
                }
                '\t' => {
                    tab = tab.toggle();
                }
                _ => {}
            }
        }
    }

    /* Persist state on close or 'q'. */
    save_state(&todos, &dones, &file_path);

    /* Terminate ncurses. */
    endwin();
}