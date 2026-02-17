

pub fn key_to_number(press: &str) -> Option<i32> {
    match press.to_ascii_lowercase().as_str() {
        "a" => Some(0),
        "b" => Some(1),
        "c" => Some(2),
        "d" => Some(3),
        "e" => Some(4),
        "f" => Some(5),
        "g" => Some(6),
        "h" => Some(7),
        "i" => Some(8),
        "j" => Some(9),
        "k" => Some(10),
        "l" => Some(11),
        "m" => Some(12),
        "n" => Some(13),
        "o" => Some(14),
        "p" => Some(15),
        "q" => Some(16),
        "r" => Some(17),
        "s" => Some(18),
        "t" => Some(19),
        "u" => Some(20),
        "v" => Some(21),
        "w" => Some(22),
        "x" => Some(23),
        "y" => Some(24),
        "z" => Some(25),

        // numbers 0–9
        "0" => Some(26),
        "1" => Some(27),
        "2" => Some(28),
        "3" => Some(29),
        "4" => Some(30),
        "5" => Some(31),
        "6" => Some(32),
        "7" => Some(33),
        "8" => Some(34),
        "9" => Some(35),

        // space and enter
        " " => Some(36),
        "space" => Some(36),
        "enter" => Some(37),

        // arrow keys
        "arrowup" => Some(38),
        "arrowdown" => Some(39),
        "arrowleft" => Some(40),
        "arrowright" => Some(41),
        _ => None
    }
}