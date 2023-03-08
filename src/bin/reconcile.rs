use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use serde::Deserialize;
use std::{error::Error, io};

use self::models::*;
use reconciliation::*;

#[derive(Debug, Deserialize)]
struct Record {
    trans_id: i32,
    fcc_payment_ref: String,
    payment_ref: String,
    #[serde()]
    trans_date: String,
    amount: f32,
}

fn main() -> Result<(), Box<dyn Error>> {
    use self::schema::transactions::dsl::*;

    let connection = &mut establish_connection();
    let mut reader = csv::Reader::from_reader(io::stdin());

    let total_count = 176235;
    let mut previous = -1;
    let mut iteration = 0;

    for result in reader.deserialize() {
        let record: Record = result?;
        let results = transactions
            .filter(
                bank_ref_id
                    .eq(Some(record.fcc_payment_ref))
                    .or(provider_reference_id.eq(Some(record.payment_ref))),
            )
            .load::<Transaction>(connection)
            .expect("Error loading transactions");

        let complete = (iteration * 100) / total_count;
        if previous < complete {
            println!("====================================");
            println!("{}% complete", complete);
            previous = complete;
        }
        let mut rec_state = false;
        let mut rec_comment = "";
        let now = Utc::now();
        let rec_date = now.format("%Y-%b-%d");

        let trx_date = NaiveDate::parse_from_str(
            &record
                .trans_date
                .split_whitespace()
                .next()
                .as_ref()
                .unwrap(),
            "%Y-%m-%d",
        )
        .unwrap();

        if results.len() > 0 {
            if results.len() > 1 {
                println!("Lets see what happens");
                for result in results {
                    rec_state = false;
                    rec_comment = "is duplicated";
                    let values = UpdateTransaction {
                        reconciliation_state: Some(rec_state),
                        reconciliation_comment: Some(rec_comment.to_string()),
                        reconciliation_date: Some(rec_date.to_string()),
                    };
                    diesel::update(transactions.filter(id.eq(result.id)))
                        .set(values)
                        .get_result::<Transaction>(connection)
                        .unwrap();
                }
            } else {
                if results[0].amount != Some(record.amount.into()) {
                    rec_state = false;
                    rec_comment = "amount does not match";
                } else if Some(results[0].created_at.format("%Y-%m-%d").to_string())
                    != Some(trx_date.to_string())
                {
                    rec_state = false;
                    rec_comment = "date posted does not match";
                } else {
                    rec_state = true;
                    rec_comment = "";
                }
                let values = UpdateTransaction {
                    reconciliation_state: Some(rec_state),
                    reconciliation_comment: Some(rec_comment.to_string()),
                    reconciliation_date: Some(rec_date.to_string()),
                };
                let trx_id: i32 = results[0].id;
                diesel::update(transactions.filter(id.eq(trx_id)))
                    .set(values)
                    .get_result::<Transaction>(connection)
                    .unwrap();
            }
        } else {
            println!(
                "No transaction found for External Transaction ID: {}",
                record.trans_id
            );
        }
        iteration += 1;
        println!("==================================== {}", record.trans_id);
    }

    Ok(())
}
