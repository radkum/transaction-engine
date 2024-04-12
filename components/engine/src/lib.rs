extern crate core;

use csv::Trim;

use crate::{engine::Engine, error::EngineError, record::Record};

mod engine;
mod error;
mod record;

#[tokio::main]
pub async fn process_transactions<R: std::io::Read, W: std::io::Write>(
    io_reader: R,
    io_writer: W,
) -> Result<(), EngineError> {
    internal_process_transactions(io_reader, io_writer).await
}

async fn internal_process_transactions<R: std::io::Read, W: std::io::Write>(
    io_reader: R,
    io_writer: W,
) -> Result<(), EngineError> {
    let mut rdr =
        csv::ReaderBuilder::new().trim(Trim::All).comment(Some(b'#')).from_reader(io_reader);

    let mut engine = Engine::new();

    // we need to process records (transactions) in the right order, so we wait until
    // the transaction is sent and then move on to the next one
    for result in rdr.deserialize() {
        let record: Record = result?;
        engine.process_record(record).await?;
    }

    // Once each transaction is processed, we can receive portfolios (fund summaries) for each client
    // we don't need them sorted, so we just print them out of order
    engine.print_wallets(io_writer).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use crate::EngineError::{CsvError, RecordError};

    async fn test_process_transaction<R: std::io::Read>(
        io_reader: R,
    ) -> Result<String, EngineError> {
        let mut output_str = Cursor::new(Vec::<u8>::new());
        internal_process_transactions(io_reader, &mut output_str).await?;

        Ok(String::from_utf8(output_str.into_inner()).unwrap())
    }

    #[tokio::test]
    async fn input_file_works() {
        let expected_str = r#"client,available,held,total
1,1.5,0,1.5,false
2,2,0,2,false"#;
        let file = std::fs::File::open("..\\..\\transaction.csv").unwrap();

        let output_str = test_process_transaction(file).await.unwrap();
        assert_eq!(output_str.as_str(), expected_str);
    }

    #[tokio::test]
    async fn incorrect_csv_format() {
        let input_str = r#"type, client, tx, amount
withdrawal, 2, ads, 3.0, asdf, asdf"#;

        let Err(CsvError(error)) = test_process_transaction(input_str.as_bytes()).await else {
            panic!();
        };
        assert_eq!(
            error.to_string(),
            "CSV error: record 1 (line: 2, byte: 25): found record with 6 fields, but the \
             previous record has 4 fields"
        );
    }

    #[tokio::test]
    async fn incorrect_csv_format_2() {
        let input_str = r#"type, client, tx, amount
withdrawal, 1, 3.0,"#;

        let Err(CsvError(error)) = test_process_transaction(input_str.as_bytes()).await else {
            panic!();
        };
        assert_eq!(
            error.to_string(),
            "CSV deserialize error: record 1 (line: 2, byte: 25): field 2: invalid digit found in \
             string"
        );
    }

    #[tokio::test]
    async fn incorrect_csv_format_3() {
        let input_str = r#"type, client, tx, amount
deposit, 1,1,"#;

        let Err(RecordError(error)) = test_process_transaction(input_str.as_bytes()).await else {
            panic!();
        };
        assert_eq!(error.to_string(), "The amount field is missing for deposit transaction in csv");
    }

    #[tokio::test]
    async fn incorrect_csv_format_4() {
        let input_str = r#"type, client, tx, amount
dispute, 1,1,1"#;

        let Err(RecordError(error)) = test_process_transaction(input_str.as_bytes()).await else {
            panic!();
        };
        assert_eq!(error.to_string(), "The amount should be empty for dispute in csv");
    }

    #[tokio::test]
    async fn incorrect_csv_format_5() {
        let input_str = r#"type, client, tx, amount
unknown123, 1,1,1"#;

        let Err(RecordError(error)) = test_process_transaction(input_str.as_bytes()).await else {
            panic!();
        };
        assert_eq!(error.to_string(), "Unknown transaction type: \"unknown123\"");
    }

    #[tokio::test]
    async fn test_case_1() {
        let input_str = r#"type, client, tx, amount
deposit, 1, 1, 1.1111
deposit, 2, 2, 2.0
deposit, 1, 3, 2.0
withdrawal, 1, 4, 1.5
dispute, 1,1,
withdrawal, 2, 5, 3.0"#;

        let expected_str = r#"client,available,held,total
1,0.5,1.1111,1.6111,false
2,2,0,2,false"#;

        let output_str = test_process_transaction(input_str.as_bytes()).await.unwrap();
        assert_eq!(output_str.as_str(), expected_str);
    }

    #[tokio::test]
    async fn test_case_2() {
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

        let output_str = test_process_transaction(input_str.as_bytes()).await.unwrap();
        assert_eq!(output_str.as_str(), expected_str);
    }

    #[tokio::test]
    async fn test_case_3() {
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

        let output_str = test_process_transaction(input_str.as_bytes()).await.unwrap();
        assert_eq!(output_str.as_str(), expected_str);
    }

    #[tokio::test]
    async fn test_case_4() {
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

        let output_str = test_process_transaction(input_str.as_bytes()).await.unwrap();
        assert_eq!(output_str.as_str(), expected_str);
    }

    #[tokio::test]
    async fn test_case_5() {
        let input_str = r#"type, client, tx, amount
withdrawal, 2, 5, 3.0"#;

        let expected_str = r#"client,available,held,total
2,0,0,0,false"#;

        let output_str = test_process_transaction(input_str.as_bytes()).await.unwrap();
        assert_eq!(output_str.as_str(), expected_str);
    }

    #[tokio::test]
    async fn test_case_6() {
        let input_str = r#"type, client, tx, amount
dispute, 2, 52,"#;

        let expected_str = r#"client,available,held,total
2,0,0,0,false"#;

        let output_str = test_process_transaction(input_str.as_bytes()).await.unwrap();
        assert_eq!(output_str.as_str(), expected_str);
    }

    #[tokio::test]
    async fn test_case_7() {
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

        let output_str = test_process_transaction(input_str.as_bytes()).await.unwrap();
        assert_eq!(output_str.as_str(), expected_str);
    }

    #[tokio::test]
    async fn test_case_8() {
        // there is no debit on account after withdraw so dispute and chargeback are ignored
        let input_str = r#"type, client, tx, amount
deposit, 1, 1, 1.0
withdrawal, 1, 2, 1.0
dispute, 1,1,
chargeback, 1,1,"#;

        let expected_str = r#"client,available,held,total
1,0,0,0,false"#;

        let output_str = test_process_transaction(input_str.as_bytes()).await.unwrap();
        assert_eq!(output_str.as_str(), expected_str);
    }

    #[tokio::test]
    async fn test_case_9() {
        let input_str = r#"type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 1, 2, 3.0
dispute, 1, 2,
withdrawal, 1,3, 2.1111"#;

        let expected_str = r#"client,available,held,total
1,1,3,4,false"#;

        let output_str = test_process_transaction(input_str.as_bytes()).await.unwrap();
        assert_eq!(output_str.as_str(), expected_str);
    }

    #[tokio::test]
    async fn test_case_10() {
        let input_str = r#"type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 1, 2, 3.0
dispute, 1, 2,
withdrawal, 1,3, 2.1111
resolve, 1,2,"#;

        let expected_str = r#"client,available,held,total
1,4,0,4,false"#;

        let output_str = test_process_transaction(input_str.as_bytes()).await.unwrap();
        assert_eq!(output_str.as_str(), expected_str);
    }

    #[tokio::test]
    async fn test_case_11() {
        let input_str = r#"type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 1, 2, 3.0
dispute, 1, 2,
withdrawal, 1,3, 2.1111
resolve, 1,2,
withdrawal, 1,3, 2.1111
dispute, 1, 2,
chargeback, 1,2,"#;

        let expected_str = r#"client,available,held,total
1,1.8889,0,1.8889,false"#;

        let output_str = test_process_transaction(input_str.as_bytes()).await.unwrap();
        assert_eq!(output_str.as_str(), expected_str);
    }

    #[tokio::test]
    async fn test_case_12() {
        let input_str = r#"type, client, tx, amount
deposit, 3, 1, 5.1234
withdrawal, 3, 2, 4.1
deposit, 6, 3, 5.1234
dispute, 6, 3,
resolve, 6, 3,"#;

        let expected_str = r#"client,available,held,total
3,1.0234,0,1.0234,false
6,5.1234,0,5.1234,false"#;

        let output_str = test_process_transaction(input_str.as_bytes()).await.unwrap();
        assert_eq!(output_str.as_str(), expected_str);
    }

    #[tokio::test]
    async fn test_case_13() {
        let input_str = r#"type, client, tx, amount
deposit, 3, 1, 5.1
withdrawal, 3, 2, 4.1234
withdrawal, 3, 5, 0.9766
deposit, 6, 3, 5.1234
dispute, 6, 3,
resolve, 6, 3,
dispute, 6, 3,
chargeback, 6, 3,"#;

        let expected_str = r#"client,available,held,total
3,0,0,0,false
6,0,0,0,true"#;

        let output_str = test_process_transaction(input_str.as_bytes()).await.unwrap();
        assert_eq!(output_str.as_str(), expected_str);
    }
}
