extern crate ncurses;
use ncurses::*;

/* Constants. */
const REGULAR_PAIR: i16 = 0;
const HIGHLIGHT_PAIR: i16 = 1;

fn main () {

    /* Init ncurses. */
    initscr();

    /* Init colors. */
    start_color();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(HIGHLIGHT_PAIR, COLOR_BLACK, COLOR_WHITE);

    /* Init application state. */
    let mut quit= false;
    let mut todos =vec![
        "Buy a bread",
        "Write the todo app",
        "Make a cup of tea"
    ];
    let mut todo_curr: usize =  0;

    /* Event loop. */
    while !quit {
        for (index, todo) in todos.iter().enumerate() {

            /* Colors. */
            let pair = {
                if todo_curr == index {
                    /* Render in a different style. */
                    HIGHLIGHT_PAIR
                } else {
                    REGULAR_PAIR
                }
            };

            attr_on(COLOR_PAIR(pair));
            mv(index as i32, 0);
            addstr(*todo);
            attr_off(COLOR_PAIR(pair));
        }
 
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
                if todo_curr != todos.len() - 1 {
                    todo_curr += 1;
                }
            }
            _ => {}
        }
    }

    /* Terminate ncurses. */
    endwin();
}