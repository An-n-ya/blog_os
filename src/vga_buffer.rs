use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]     // each enum variant is stored as u8
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]   // to ensure the ColorCode has the same layout as u8
struct ColorCode(u8);

impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]      // it guarantees the struct's fields ordering like the order below and do not change
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    // compiler knows nothing about this memory, the write maybe optimized by compiler
    // use volatile to make sure that the write to the buffer will not be ignored
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// the Writer struct specifies where to write, what the color is, and which memory to write
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,        // the buffer's lifetime is valid for the whole program
}

impl Writer{
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.new_line();
            },
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1; // write in the last row
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(
                ScreenChar{
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1; // move to the next position
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        // iterates each char
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // the other part is not printable, we print ■ instead
                _ => self.write_byte(0xfe)
            }

        }
    }

    fn new_line(&mut self) {
        // DONE: chang the position of the cursor to a newline
        for row in 1..BUFFER_HEIGHT {
            // the first row, i.e. the 0 row is hidden
            for col in 0..BUFFER_WIDTH {
                // move the current row to the previous row
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row-1][col].write(character)
            }
        }

        // remove the last row, and set the current cursor to the beginning
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}


impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        // always OK
        Ok(())
    }
}



pub fn print_something() {
    // to use write! macro, you need this
    use core::fmt::Write;
    let mut writer = Writer{
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe{&mut *(0xb8000 as *mut Buffer)},
    };

    writer.write_byte(b'H');
    writer.write_string("ello world\n");
    write!(writer, "the numbers are {} and {}", 42, 1.0/7.0).unwrap()
}


lazy_static!{
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe{&mut *(0xb8000 as *mut Buffer)}
    });
}


// println

#[macro_export]
macro_rules! print {
    ($($args:tt)*) => ($crate::vga_buffer::_print(format_args!($($args)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]  // hide the function in generated doc
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    // call WRITER
    WRITER.lock().write_fmt(args).unwrap();
}


// test
#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_massive_output() {
    for _ in 0..200 {
        println!("test_println_massive_output output");
    }
}

#[test_case]
fn test_println_output() {
    let s = "SOme test string that fits on a single line";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        // read the printed line
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.ascii_character), c);
    }
}

