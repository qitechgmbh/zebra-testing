use reqwest::blocking::Client;
use arrow::ipc::reader::StreamReader;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let client = Client::new();

    let response = client
        .get("http://localhost:3000/data")
        .send()?;

    let reader = StreamReader::try_new(response, None)?;

    for batch_result in reader {
        let batch = batch_result?;

        let num_rows = batch.num_rows();
        let num_cols = batch.num_columns();

        for row_idx in 0..num_rows {
            print!("row {}: ", row_idx);

            for col_idx in 0..num_cols {
                let column = batch.column(col_idx);

                // convert value to string (simple version)
                let value = column
                    .as_any()
                    .downcast_ref::<arrow::array::Int16Array>()
                    .map(|arr| arr.value(row_idx))
                    .unwrap_or(0);

                print!("{} ", value);
            }

            println!();
        }
    }

    Ok(())
}