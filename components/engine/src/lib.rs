extern crate core;

use std::{fs::File, io::Write};

use csv::{Reader, Trim};

use crate::{engine::Engine, error::EngineError, record::Record};

mod engine;
mod error;
mod record;

#[tokio::main]
pub async fn process_file(input_file: &str, out: &mut impl Write) -> Result<(), EngineError> {
    let mut reader = read_csv(input_file)?;

    let mut engine = Engine::new();

    for result in reader.deserialize() {
        let record: Record = result?;
        engine.process_record(record).await?;
    }
    engine.print_wallets(out).await?;

    Ok(())
}

fn read_csv(input_file: &str) -> Result<Reader<File>, EngineError> {
    // check if file extension is ".csv"
    if !input_file.ends_with(".csv") {
        return Err(EngineError::InputFileError(
            "Incorrect file extension. Extension must be \".csv\"".to_string(),
        ));
    }

    let reader = csv::ReaderBuilder::new()
        .trim(Trim::All)
        .comment(Some(b'#'))
        .from_reader(std::fs::File::open(input_file)?);

    Ok(reader)
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, ErrorKind};

    use super::*;
    use crate::EngineError::{CsvError, RecordError};

    #[tokio::main]
    pub async fn test_process_file(input_file: &str) -> Result<String, EngineError> {
        let output_str = test_process_reader(std::fs::File::open(input_file)?).await?;

        Ok(output_str)
    }

    #[tokio::main]
    pub async fn test_process_string(input_string: &str) -> Result<String, EngineError> {
        let output_str = test_process_reader(input_string.as_bytes()).await?;

        Ok(output_str)
    }

    pub async fn test_process_reader<R: std::io::Read>(
        io_reader: R,
    ) -> Result<String, EngineError> {
        let mut rdr =
            csv::ReaderBuilder::new().trim(Trim::All).comment(Some(b'#')).from_reader(io_reader);

        let mut engine = Engine::new();

        for result in rdr.deserialize() {
            let record: Record = result?;
            engine.process_record(record).await?;
        }

        let mut output_str = Cursor::new(Vec::<u8>::new());
        engine.print_sorted_wallets(&mut output_str).await?;

        Ok(String::from_utf8(output_str.into_inner()).unwrap())
    }

    #[test]
    fn input_file_works() {
        let expected_str = r#"client,available,held,total
1,1.5,0,1.5,false
2,2,0,2,false"#;
        let str = test_process_file("..\\..\\transaction.csv").unwrap();
        assert_eq!(str.as_str(), expected_str);
    }

    #[test]
    fn read_csv_fails_incorrect_path() {
        let res = test_process_file("..\\transaction.csv");
        let Some(EngineError::IoError(io_error)) = res.err() else { panic!() };

        assert_eq!(io_error.kind(), ErrorKind::NotFound);
        assert_eq!(io_error.to_string(), "The system cannot find the file specified. (os error 2)");
    }

    #[test]
    fn read_csv_fails_incorrect_file_extension() {
        let res = read_csv("..\\..\\Cargo.toml");

        let Some(EngineError::InputFileError(io_error)) = res.err() else { panic!() };

        assert_eq!(io_error, "Incorrect file extension. Extension must be \".csv\"");
    }

    #[test]
    fn incorrect_csv_format() {
        let input_str = r#"type, client, tx, amount
withdrawal, 2, ads, 3.0, asdf, asdf"#;

        let Err(CsvError(error)) = test_process_string(input_str) else {
            panic!();
        };
        assert_eq!(
            error.to_string(),
            "CSV error: record 1 (line: 2, byte: 25): found record with 6 fields, but the \
             previous record has 4 fields"
        );
    }

    #[test]
    fn incorrect_csv_format_2() {
        let input_str = r#"type, client, tx, amount
withdrawal, 1, 3.0,"#;

        let Err(CsvError(error)) = test_process_string(input_str) else {
            panic!();
        };
        assert_eq!(
            error.to_string(),
            "CSV deserialize error: record 1 (line: 2, byte: 25): field 2: invalid digit found in \
             string"
        );
    }

    #[test]
    fn incorrect_csv_format_3() {
        let input_str = r#"type, client, tx, amount
deposit, 1,1,"#;

        let Err(RecordError(error)) = test_process_string(input_str) else {
            panic!();
        };
        assert_eq!(error.to_string(), "The amount field is missing for deposit transaction in csv");
    }

    #[test]
    fn incorrect_csv_format_4() {
        let input_str = r#"type, client, tx, amount
dispute, 1,1,1"#;

        let Err(RecordError(error)) = test_process_string(input_str) else {
            panic!();
        };
        assert_eq!(error.to_string(), "The amount should be empty for dispute in csv");
    }

    #[test]
    fn incorrect_csv_format_5() {
        let input_str = r#"type, client, tx, amount
unknown123, 1,1,1"#;

        let Err(RecordError(error)) = test_process_string(input_str) else {
            panic!();
        };
        assert_eq!(error.to_string(), "Unknown transaction type");
    }

    #[test]
    fn test_case_1() {
        let input_str = r#"type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
dispute, 1,1,
withdrawal, 2, 5, 3.0"#;

        let expected_str = r#"client,available,held,total
1,0.5,1,1.5,false
2,2,0,2,false"#;

        let output_str = test_process_string(input_str).unwrap();
        assert_eq!(output_str.as_str(), expected_str);
    }

    #[test]
    fn test_case_2() {
        let input_str = r#"type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
dispute, 1,1,
resolve, 1,1,
withdrawal, 2, 5, 3.0"#;

        let expected_str = r#"client,available,held,total
1,1.5,0,1.5,false
2,2,0,2,false"#;

        let output_str = test_process_string(input_str).unwrap();
        assert_eq!(output_str.as_str(), expected_str);
    }

    #[test]
    fn test_case_3() {
        let input_str = r#"type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
dispute, 1,1,
chargeback, 1,1,
withdrawal, 2, 5, 3.0"#;

        let expected_str = r#"client,available,held,total
1,0.5,0,0.5,true
2,2,0,2,false"#;

        let output_str = test_process_string(input_str).unwrap();
        assert_eq!(output_str.as_str(), expected_str);
    }

    #[test]
    fn test_case_4() {
        let input_str = r#"type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
withdrawal, 1, 8, 1.5
dispute, 1,1,
chargeback, 1,1,
withdrawal, 2, 5, 3.0"#;

        let expected_str = r#"client,available,held,total
1,0,0,0,false
2,2,0,2,false"#;

        let output_str = test_process_string(input_str).unwrap();
        assert_eq!(output_str.as_str(), expected_str);
    }

    #[test]
    fn test_case_5() {
        let input_str = r#"type, client, tx, amount
withdrawal, 2, 5, 3.0"#;

        let expected_str = r#"client,available,held,total
2,0,0,0,false"#;

        let output_str = test_process_string(input_str).unwrap();
        assert_eq!(output_str.as_str(), expected_str);
    }

    #[test]
    fn test_case_6() {
        let input_str = r#"type, client, tx, amount
dispute, 2, 52,"#;

        let expected_str = r#"client,available,held,total
2,0,0,0,false"#;

        let output_str = test_process_string(input_str).unwrap();
        assert_eq!(output_str.as_str(), expected_str);
    }

    #[test]
    fn test_case_7() {
        let input_str = r#"type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
dispute, 1,1,
chargeback, 1,1,
withdrawal, 2, 5, 3.0
deposit, 1, 1, 1.0"#;

        let expected_str = r#"client,available,held,total
1,0.5,0,0.5,true
2,2,0,2,false"#;

        let output_str = test_process_string(input_str).unwrap();
        assert_eq!(output_str.as_str(), expected_str);
    }
}
