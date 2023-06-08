use crate::interrupts::{LASTPRESSED, STOPWATCH};
use crate::program::Program;
use crate::vga_buffer::WRITER;
use crate::{print, println};
use core::mem::MaybeUninit;
use fixed_slice_vec::FixedSliceVec;
use pc_keyboard::DecodedKey;
use rand::rngs::SmallRng;
use rand::RngCore;
use rand::SeedableRng;

// create game consts
const TIME_PER_TILE: u8 = 5;
const GAME_LENGTH: u8 = 20;
const GAME_HEIGHT: u8 = 10;
const STARTING_SNAKE: u8 = 4;

pub struct SnakeGame;

impl Program for SnakeGame {
    fn run(&mut self) -> Result<(), &'static str> {
        let mut bytes: [MaybeUninit<u8>; 250] = unsafe { MaybeUninit::uninit().assume_init() };
        let food_bytes_slice = &mut bytes[..250];
        let mut food_vec: FixedSliceVec<(u8, u8)> =
            FixedSliceVec::from_uninit_bytes(food_bytes_slice);

        let mut small_rng = SmallRng::seed_from_u64(23625234);
        while !food_vec.is_full() {
            println!("calculating...");
            let mut new_x = (small_rng.next_u32() >> 24) as u8;
            while new_x == 0 || new_x >= GAME_LENGTH {
                new_x = (small_rng.next_u32() >> 24) as u8;
            }
            let mut new_y = (small_rng.next_u32() >> 24) as u8;
            while new_y == 0 || new_y >= GAME_HEIGHT {
                new_y = (small_rng.next_u32() >> 24) as u8;
            }
            food_vec.push((new_x, new_y));
        }
        let mut bytes: [MaybeUninit<u8>; 450] = unsafe { MaybeUninit::uninit().assume_init() };
        let byte_slice = &mut bytes[..450];
        let mut snake_vec: FixedSliceVec<Point> = FixedSliceVec::from_uninit_bytes(byte_slice);

        // creates vector of snake nodes and pushes starting snake nodes
        for num in (1..=STARTING_SNAKE).rev() {
            snake_vec.push(Point {
                x: num,
                y: GAME_HEIGHT / 2,
            });
        }

        // variables :0
        let display_arr: [[char; GAME_LENGTH as usize]; GAME_HEIGHT as usize] =
            [['.'; GAME_LENGTH as usize]; GAME_HEIGHT as usize];
        let mut money = Point {
            x: GAME_LENGTH - 1,
            y: GAME_HEIGHT / 2,
        };
        let mut direction = Direction::Right;
        let mut eating = false;
        let mut death = false;

        // main game loop
        loop {
            if let DecodedKey::Unicode(val) = *LASTPRESSED.lock() {
                let new_direction = match val {
                    'w' => Direction::Up,
                    'a' => Direction::Left,
                    's' => Direction::Down,
                    'd' => Direction::Right,
                    _ => direction.clone(),
                };
                if new_direction != direction.get_opposite() {
                    direction = new_direction;
                }
            }

            // removes from the tail and adds to the head of the snake vector
            let new_snake_node = match direction {
                Direction::Up => Point {
                    x: snake_vec[0].x,
                    y: snake_vec[0].y - 1,
                },
                Direction::Down => Point {
                    x: snake_vec[0].x,
                    y: snake_vec[0].y + 1,
                },
                Direction::Left => Point {
                    x: snake_vec[0].x - 1,
                    y: snake_vec[0].y,
                },
                Direction::Right => Point {
                    x: snake_vec[0].x + 1,
                    y: snake_vec[0].y,
                },
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
                snake_vec
                    .try_insert(0, new_snake_node)
                    .expect("snake vec full");

                // if snake eats then generate new food and eaiting = true
                if yum_yum(&snake_vec, &money) {
                    eating = true;

                    let (mut x, mut y) = food_vec.pop().expect("ran out of food");
                    while is_in_vec(&Point { x, y }, &snake_vec) {
                        (x, y) = food_vec.pop().expect("ran out of food");
                    }
                    money = Point { x, y };
                }
            }
            // displays score and direction
            println!("Score: {}", snake_vec.len() - 4);
            // creates array using function and adds the food yum yum!!!
            let print_out = &mut snake_to_display(&display_arr, &snake_vec);
            print_out[(money.y - 1) as usize][(money.x - 1) as usize] = '$';

            // displays top border
            println!("{empty:->width$}", empty = "", width = GAME_LENGTH as usize);

            // loops through the 2d array and collects each 1d arrray into a string and displays it
            for number in 0..GAME_HEIGHT {
                print_out[(number) as usize]
                    .iter()
                    .for_each(|c| print!("{}", c));
                println!("");
            }

            // displays bottom border
            println!("{empty:->width$}", empty = "", width = GAME_LENGTH as usize);

            // sleeps for the time per tile
            let start_time = *STOPWATCH.lock();
            loop {
                if let Some(val) = STOPWATCH.try_lock() {
                    if *val - start_time >= TIME_PER_TILE as u128 {
                        break;
                    }
                } else {
                    println!("lock is locked");
                }
                for _ in 0..100000 {}
            }

            // if the snake died then display "you died" then waits for one last character input before
            // breaking the loop
            if death {
                println!("You Died");
                break;
            }

            // clears the screen for the next iteration of the loop
            WRITER.lock().clear();
        }
        Ok(())
    }
}

#[derive(PartialEq)]
struct Point {
    x: u8,
    y: u8,
}

// this functions is for making sure the food doesn't spawin in the snake
fn is_in_vec(point: &Point, vec: &FixedSliceVec<Point>) -> bool {
    for vec_thing in vec.iter() {
        if point == vec_thing {
            return true;
        }
    }
    return false;
}

// clones display array then loops through the snake vector and replaces the respective '.' with '@' also the head of the snake values is saved so that at the end the head is a '&'
fn snake_to_display(
    display_arr: &[[char; GAME_LENGTH as usize]; GAME_HEIGHT as usize],
    snake_vec: &FixedSliceVec<Point>,
) -> [[char; GAME_LENGTH as usize]; GAME_HEIGHT as usize] {
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

// a really over engineered way of making sure non of the values in the snake
// vector are the same
fn does_snake_die(snake: &FixedSliceVec<Point>) -> bool {
    let mut bytes: [MaybeUninit<u8>; 1000] = unsafe { MaybeUninit::uninit().assume_init() };
    let bytes_silce = &mut bytes[..1000];
    let mut done_ur_mum: FixedSliceVec<&Point> = FixedSliceVec::from_uninit_bytes(bytes_silce);
    for node in snake.iter() {
        for value in done_ur_mum.iter() {
            if &node == value {
                return true;
            }
        }
        done_ur_mum
            .try_push(node)
            .expect("the done_ur_mum vec is full");
    }

    return false;
}

// if snake is colliding with food then YUM YUM!!!
fn yum_yum(snake: &FixedSliceVec<Point>, money: &Point) -> bool {
    for node in snake.iter() {
        if node == money {
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
}

impl Direction {
    fn get_opposite(self: &Self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}
