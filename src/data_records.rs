use csv::{Reader, ReaderBuilder, Trim};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct InputRecord {
    #[serde(rename = "type")]
    pub oper_type: String,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f32>,
}

pub fn build_reader<T>(input: T) -> Reader<T>
where
    T: std::io::Read,
{
    ReaderBuilder::new()
        .flexible(true)
        .trim(Trim::All)
        .from_reader(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialization_of_input() {
        let csv_input = "\
type,client,tx,amount
deposit,1,1,1.0
";
        let expected_input_record = InputRecord {
            oper_type: "deposit".to_string(),
            client: 1,
            tx: 1,
            amount: Some(1.0),
        };

        let mut reader = build_reader(csv_input.as_bytes());
        let actual_input_record: InputRecord = reader
            .deserialize()
            .next()
            .expect("It shouldn't be empty")
            .expect("Should be properly deserialized");
        assert_eq!(expected_input_record, actual_input_record);
    }

    #[test]
    fn test_deserialization_of_input_with_whitespaces_and_tabs() {
        let csv_input = "\
type, client, tx, amount
deposit, 1, 1,      1.0
";
        let expected_input_record = InputRecord {
            oper_type: "deposit".to_string(),
            client: 1,
            tx: 1,
            amount: Some(1.0),
        };

        let mut reader = build_reader(csv_input.as_bytes());
        let actual_input_record: InputRecord = reader
            .deserialize()
            .next()
            .expect("It shouldn't be empty")
            .expect("Should be properly deserialized");
        assert_eq!(expected_input_record, actual_input_record);
    }
}
