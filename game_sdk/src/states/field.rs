use states::FieldType;
use std;
#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct Field {
    pub fieldtype: FieldType,
    pub x: u8,
    pub y: u8,
}

impl Field {
    pub fn get_xml(&self) -> String {
        return format!(
            "<field x=\"{}\" y=\"{}\" state=\"{}\"/>",
            self.x, self.y, self.fieldtype
        );
    }

    pub fn get_bit_index(&self) -> u128 {
        return 0b1 << (self.x + self.y * 10);
    }
}

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}|{}:{}", self.x, self.y, self.fieldtype)
    }
}
impl std::fmt::Debug for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}|{}:{}", self.x, self.y, self.fieldtype)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_bit_index() {
        let field = Field {
            fieldtype: FieldType::Free,
            x: 2,
            y: 5,
        };
        assert_eq!(field.get_bit_index(), 0b1<<52);
    }
}
