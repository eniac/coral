use cfg_core::Outputs;
use csv::Writer;
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::process;
use std::time::SystemTime;
use validate_input::validate_input;

#[cfg(feature = "metrics")]
use metrics::metrics::{log, log::Component};

fn main() {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input_file> <grammar_file>", args[0]);
        process::exit(1);
    }

    let grammar_file = &args[2];
    let input_file = &args[1];

    println!("Reading grammar file: {}", grammar_file);
    let grammar_content = match fs::read_to_string(grammar_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading grammar file: {}", e);
            process::exit(1);
        }
    };

    println!("Reading input file: {}", input_file);
    let input_data = match fs::read_to_string(input_file) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading input file: {}", e);
            process::exit(1);
        }
    };

    #[cfg(feature = "metrics")]
    {
        assert!(args.len() == 4, "Expected metrics file");
        let metrics = PathBuf::from(args[3].clone());
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&metrics)
            .unwrap();
        let mut wtr = Writer::from_writer(file);
        let _ = wtr.write_record(&[
            input_file.to_string(),
            grammar_file.to_string(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string(),
            input_data.len().to_string(),
        ]);
        let spacer = "---------";
        let _ = wtr.write_record(&[spacer, spacer, spacer, spacer, "\n"]);
        let _ = wtr.flush();
        #[cfg(feature = "metrics")]
        log::write_csv(metrics.to_str().unwrap()).unwrap();
    }

    println!("Starting validation...");
    let (_, is_valid) = validate_input(&grammar_content, &input_data);

    #[cfg(feature = "metrics")]
    log::write_csv(
        &PathBuf::from(args[3].clone())
            .as_path()
            .display()
            .to_string(),
    )
    .unwrap();
}
