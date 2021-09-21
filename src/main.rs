mod cli;
mod data_records;

fn main() {
    let opt = cli::Opt::get_input_file();
    println!("{:?}", opt);
}
