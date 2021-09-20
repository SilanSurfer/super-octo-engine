use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq)]
pub struct InputRecord {
    #[serde(rename = "type")]
    oper_type: String,
    client: u16,
    tx: u32,
    amount: f32,
}

#[derive(Debug, Serialize)]
pub struct OutputRecord {
    client: u16,
    available: f32,
    held: f32,
    total: f32,
    locked: bool,
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
            amount: 1.0,
        };

        let mut reader = csv::Reader::from_reader(csv_input.as_bytes());
        let actual_input_record: InputRecord = reader
            .deserialize()
            .next()
            .expect("It shouldn't be empty")
            .expect("Should be properly deserialized");
        assert_eq!(expected_input_record, actual_input_record);
    }

    #[test]
    fn test_serialization_of_output() {
        let output_record = OutputRecord {
            client: 1,
            available: 1.5,
            held: 0.0,
            total: 1.5,
            locked: false,
        };
        let expected_csv_output = "\
client,available,held,total,locked
1,1.5,0.0,1.5,false
";

        let mut writer = csv::Writer::from_writer(vec![]);
        writer
            .serialize(output_record)
            .expect("Should be properly serialized");
        let actual_csv_output = String::from_utf8(writer.into_inner().expect("Should convert to inner"))
            .expect("Should convert to UTF-8");

        assert_eq!(expected_csv_output, actual_csv_output);
    }
}
