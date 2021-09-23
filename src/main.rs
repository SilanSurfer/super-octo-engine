mod cli;
mod data_records;
mod engine;
mod errors;
mod operations;

use engine::Engine;

use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_file_path = cli::Opt::get_input_file();
    let mut engine: Engine = Engine::new();
    let file = File::open(input_file_path)?;
    let mut reader = csv::Reader::from_reader(file);
    for result in reader.deserialize() {
        match result {
            Ok(record) => {
                println!("{:?}", record);
                if let Err(e) = engine.process_record(record) {
                    eprintln!("Error: {:?}", e);
                }
            }
            Err(e) => {
                eprintln!("Error while deserializing record! {}", e);
            }
        }
    }
    Ok(())
}
