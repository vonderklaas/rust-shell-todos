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
    fn begin_list(&mut self, id: Id){
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
    fn end_list(&mut self ){
        self.list_curr = None;
    }
    fn end(&mut self) {
    }
}

fn main () {

    /* Init ncurses. */
    initscr();

    /* Disable cursor and keys echo. */
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    noecho();

    /* Init colors. */
    start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(HIGHLIGHT_PAIR, COLOR_BLACK, COLOR_WHITE);

    /* Init application state. */
    let mut quit= false;

    let mut todos: Vec<String> = vec![
        "Buy a bread".to_string(),
        "Write the todo app".to_string(),
        "Make a cup of tea".to_string()
    ];
    let mut todo_curr: usize = 0;

    let mut dones: Vec<String> = vec![
        "Start the stream".to_string(),
        "Have a breakfast".to_string(),
        "Make a cup of tea".to_string()
    ];
    let mut done_curr: usize = 0;

    /* Create default interface. */
    let mut ui = Ui::default();

    /* Event loop. */
    while !quit {
        erase();

        /* Start the interface. */
        ui.begin(0, 0);

        {
            ui.label("TODO:",REGULAR_PAIR);
            ui.begin_list(todo_curr);

            for (index, todo) in todos.iter().enumerate() {
               ui.list_element(&format!("- [] {}", todo), index);
            }
    
            ui.end_list();
    
            /* Separation. */
            ui.label("------------------------------", REGULAR_PAIR);
    
            /* 'Done' list */
            ui.label("DONE:",REGULAR_PAIR);
            ui.begin_list(0);
            for (index, done) in dones.iter().enumerate() {
                ui.list_element(&format!("- [x] {}", done), index + 1);
            }
            ui.end_list();
    
            ui.end();
     
            /* Update the screen. */
            refresh();
    
            /* Wait for a key press. */
            let key = getch();
    
            /* Handle input from the user. */
            match key as u8 as char {
                'q' => quit = true,
                'w' => {
                    if todo_curr > 0 {
                        todo_curr -= 1;
                    }
                }
                's' => {
                    if todo_curr + 1 < todos.len() {
                        todo_curr += 1;
                    }
                }
                '\n' => {
                    if todo_curr < todos.len() {
                        let removed_todo = todos.remove(todo_curr);
                        dones.push(removed_todo);
                    }
                }
                _ => {}
            }
        }
    }

    /* Terminate ncurses. */
    endwin();
}