///
/// example is from wikipedia: https://en.wikipedia.org/wiki/Builder_pattern
///

#[derive(Debug, Eq, PartialEq)]
struct Car {
    maker: String,
    seats: u8,
    color: String,
}

struct CarBuilder {
    color: String,
    seats: u8,
    maker: String
}

impl CarBuilder {
    fn new() -> CarBuilder {
        CarBuilder {
            color: String::from(""),
            seats: 4,
            maker: String::from("")
        }
    }

    fn with_color(mut self, color: &str) -> Self {
        self.color = String::from(color);
        self
    }

    fn with_seats(mut self, seats: u8) -> Self {
        self.seats = seats;
        self
    }

    fn with_maker(mut self, maker: &str) -> Self {
        self.maker = String::from(maker);
        self
    }

    fn build(self) -> Car {
        Car {
            color: self.color,
            maker: self.maker,
            seats: self.seats
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test() {
        let builder = CarBuilder::new();

        let car = builder
            .with_color("black")
            .with_maker("lamborghini")
            .with_seats(4)
            .build();

        let expected = Car {
            color: "black".to_string(),
            maker: "lamborghini".to_string(),
            seats: 4,
        };

        assert_eq!(car, expected);
    }
}
