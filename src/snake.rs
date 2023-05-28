use crate::print;
use crate::vga_buffer::BUFFER_HEIGHT;
use core::mem::MaybeUninit;
use fixed_slice_vec::FixedSliceVec;

pub fn run() {
    // create game consts
    // const TIME_PER_TILE: time::Duration = time::Duration::from_millis(200);
    const GAME_LENGTH: i32 = 20;
    const GAME_HEIGHT: i32 = 10;
    const STARTING_SNAKE: i32 = 4;

    let mut bytes: [MaybeUninit<u8>; 200] = unsafe { MaybeUninit::uninit().assume_init() };
    let byte_slice = &mut bytes[..200];
    let mut snake_vec: FixedSliceVec<SnakeNode> = FixedSliceVec::from_uninit_bytes(byte_slice);

    // creates vector of snake nodes and pushes starting snake nodes
    for num in (1..=STARTING_SNAKE).rev() {
        snake_vec.push(SnakeNode {
            x: num,
            y: GAME_HEIGHT / 2,
        });
    }

    // variables :0
    let display_arr: [[char; GAME_LENGTH as usize]; GAME_HEIGHT as usize] =
        [['~'; GAME_LENGTH as usize]; GAME_HEIGHT as usize];
    let mut money = Money {
        x: GAME_LENGTH - 1,
        y: GAME_HEIGHT / 2,
    };
    let mut direction = Direction::Right;
    let mut eating = false;
    let mut death = false;

    let mut bytes: [MaybeUninit<u8>; 50] = unsafe { MaybeUninit::uninit().assume_init() };
    let byte_slice = &mut bytes[..50];
    let mut input_vec: FixedSliceVec<Direction> = FixedSliceVec::from_uninit_bytes(byte_slice);

    // main game loop
    loop {
        // adds all the inputs to vector
        // this is to stop all the inputs from stacking and instead accepts the first input you
        // pressed in the 'time per tile'

        if !input_vec.is_empty() {
            let new_direction = input_vec[0].clone();
            if !(new_direction == direction.get_opposite()) && !(direction == new_direction) {
                direction = new_direction;
            }
            input_vec.remove(0);
            if input_vec.len() > 2 {
                input_vec.pop();
            }
        }

        // removes from the tail and adds to the head of the snake vector
        let new_snake_node = match direction {
            Direction::Up => SnakeNode {
                x: snake_vec[0].x,
                y: snake_vec[0].y - 1,
            },
            Direction::Down => SnakeNode {
                x: snake_vec[0].x,
                y: snake_vec[0].y + 1,
            },
            Direction::Left => SnakeNode {
                x: snake_vec[0].x - 1,
                y: snake_vec[0].y,
            },
            Direction::Right => SnakeNode {
                x: snake_vec[0].x + 1,
                y: snake_vec[0].y,
            },
            _ => {
                panic!("unexpected direction")
            }
        };

        // if snake is exiting the borders or collides with itself then the snake dies
        if new_snake_node.x > GAME_LENGTH
            || new_snake_node.x <= 0
            || new_snake_node.y > GAME_HEIGHT
            || new_snake_node.y <= 0
            || does_snake_die(&snake_vec)
        {
            death = true;
        } else {
            // if the snake ate the money then the snake node is added to the front but none is
            // removed from the back
            if eating {
                eating = false;
            } else {
                snake_vec.pop();
            }
            snake_vec.insert(0, new_snake_node);

            // if snake eats then generate new food and eaiting = true
            if yum_yum(&snake_vec, &money) {
                eating = true;
                let mut new_x = 3;
                let mut new_y = 10;

                let mut bytes: [MaybeUninit<u8>; 250] =
                    unsafe { MaybeUninit::uninit().assume_init() };
                let byte_slice = &mut bytes[..250];
                let mut values_vec: FixedSliceVec<(i32, i32)> =
                    FixedSliceVec::from_uninit_bytes(byte_slice);
                snake_vec
                    .iter()
                    .for_each(|snake| values_vec.push((snake.x, snake.y)));

                while is_in_vec(&(new_x, new_y), &values_vec) {
                    new_x = 9;
                    new_y = 20;
                }
                money = Money { x: new_x, y: new_y };
            }
        }
        // displays score and direction
        print!("Score: {}\n", snake_vec.len() - 4);
        // creates array using function and adds the food yum yum!!!
        let print_out = &mut snake_to_display(&display_arr, &snake_vec);
        print_out[(money.y - 1) as usize][(money.x - 1) as usize] = '$';

        // displays top border
        print!(
            "{empty:->width$}",
            empty = "",
            width = (GAME_LENGTH + 2) as usize
        );

        // loops through the 2d array and collects each 1d arrray into a string and displays it
        for number in 1..=GAME_HEIGHT {
            let num_1 = (number - 1) as usize;
            print!("|{:?}|\n", print_out[num_1]);
        }

        // displays bottom border
        print!(
            "{empty:->width$}",
            empty = "",
            width = (GAME_LENGTH + 2) as usize
        );

        // sleeps for the time per tile
        //thread::sleep(TIME_PER_TILE);

        // if the snake died then display "you died" then waits for one last character input before
        // breaking the loop
        if death {
            print!("You Died");
            break;
        }

        // clears the screen for the next iteration of the loop
        for _ in 0..=BUFFER_HEIGHT {
            print!("\n");
        }
    }
}

// defo don't need these as two structs but i thought it would make it look nicer
struct SnakeNode {
    x: i32,
    y: i32,
}

struct Money {
    x: i32,
    y: i32,
}

// this functions is for making sure the food doesn't spawin in the snake
fn is_in_vec(values: &(i32, i32), vec: &FixedSliceVec<(i32, i32)>) -> bool {
    for vec_thing in vec.iter() {
        if values == vec_thing {
            return true;
        }
    }
    return false;
}

// clones display array then loops through the snake vector and replaces the respective '.' with '@' also the head of the snake values is saved so that at the end the head is a '&'
fn snake_to_display(
    display_arr: &[[char; 20]; 10],
    snake_vec: &FixedSliceVec<SnakeNode>,
) -> [[char; 20]; 10] {
    let mut new_arr = display_arr.clone();
    let mut first = true;
    let mut first_node: (usize, usize) = (69, 420);
    for block_thing in snake_vec.iter() {
        if first {
            first_node = ((block_thing.y - 1) as usize, (block_thing.x - 1) as usize);
            first = false;
        } else {
            new_arr[(block_thing.y - 1) as usize][(block_thing.x - 1) as usize] = '@';
        }
    }
    new_arr[first_node.0][first_node.1] = '&';
    new_arr
}

// this is probably a really over engineered way of making sure non of the values in the snake
// vector are the same
fn does_snake_die(snake: &FixedSliceVec<SnakeNode>) -> bool {
    let mut bytes: [MaybeUninit<u8>; 250] = unsafe { MaybeUninit::uninit().assume_init() };
    let bytes_silce = &mut bytes[..250];
    let mut done_ur_mum: FixedSliceVec<(i32, i32)> = FixedSliceVec::from_uninit_bytes(bytes_silce);
    for node in snake.iter() {
        for values in done_ur_mum.iter() {
            if &(node.x, node.y) == values {
                return true;
            }
        }
        done_ur_mum.push((node.x, node.y));
    }

    return false;
}

// if snake is colliding with food then YUM YUM!!!
fn yum_yum(snake: &FixedSliceVec<SnakeNode>, money: &Money) -> bool {
    for node in snake.iter() {
        if node.x == money.x && node.y == money.y {
            return true;
        }
    }
    return false;
}

// enum for direction (duh) and it derives debug in order to display the direction of the snake and
// it derives partialEq so that you can use it in an if statement
#[derive(Debug, PartialEq, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
    None,
}

impl Direction {
    fn get_opposite(self: &Self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            _ => {
                panic!("wtf")
            }
        }
    }
}
