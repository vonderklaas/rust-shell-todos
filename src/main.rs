use std::fs::File;
use std::io::{self, BufRead}; // Write
use std::env::args;
use std::process;
extern crate ncurses;
use ncurses::*;

/* Constants. */
const REGULAR_PAIR: i16 = 0;
const HIGHLIGHT_PAIR: i16 = 1;

/* Types. */
type Id = usize;

/* Ui interface. */
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
        /* Disallow nested lists. */
        assert!(self.list_curr.is_none(), "Nested lists are not allowed.");
        self.list_curr = Some(id);
    }
    fn list_element(&mut self, label: &str, id: Id) -> bool {
        /* Run list_element only inside list. */
        let id_curr = self.list_curr.expect("Not allowed to create list elements outside of lists.");
        let pair = {
            if id_curr == id {
                /* Render in a different style. */
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
        /* Rendering things downwards. */
        self.row += 1;
    }
    fn end_list(&mut self) {
        self.list_curr = None;
    }
    fn end(&mut self) {
    }
}

/* View focus. */
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

/* To which vector we have to push the item. */
fn parse_item(line: &str) -> Option<(Status, &str)> {
    let todo_prefix = "TODO: ";
    let done_prefix = "DONE: ";
    if line.starts_with(todo_prefix) {
        return Some((Status::Todo, &line[todo_prefix.len()..]));
    }
    if line.starts_with(done_prefix) {
        return Some((Status::Done, &line[done_prefix.len()..]));
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

// TODO:
// - persist the state of the application (text file?)
// - add new items to TODO
// - delete items
// - edit the items
// - keep track of date when the item was DONE
// - undo system

fn main () {

    /* Prints each argument on a separate line. */
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
    /* Init application state. */
    let mut quit= false;

    let mut todos = Vec::<String>::new();
    let mut dones = Vec::<String>::new();

    let mut todo_curr: usize = 0;
    let mut done_curr: usize = 0;

    {
        let file = File::open(file_path.clone()).unwrap();
        for (index, line_result) in io::BufReader::new(file).lines().enumerate() {
            let line = line_result.unwrap();
            match parse_item(&line) {
                Some((Status::Todo, title)) => {
                    todos.push(title.to_string())
                },
                Some((Status::Done, title)) => {
                    dones.push(title.to_string())
                }
                None => {
                    eprintln!("{}:{}: ERROR: bad formatted item line", file_path, index + 1);
                    process::exit(1);
                }
            }
            println!("{:?}", parse_item(&line));
        }
    }        

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
                // 'e' => {
                //     let mut file = File::create("TODO").unwrap();
                //     for todo in todos.iter() {
                //         writeln!(file, "TODO: {}", todo);
                //     }
                //     for done in dones.iter() {
                //         writeln!(file, "DONE: {}", done);
                //     }
                // }
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
                _ => {
                    // todos.push(format!("{}", key));
                }
            }
        }
    }

    /* Terminate ncurses. */
    endwin();
}