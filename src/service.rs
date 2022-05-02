use std::sync::Arc;

use futures::future::try_join_all;
use tokio_postgres::types::ToSql;
use tokio_postgres::Client;
use warp::http::response::Response;
use warp::http::StatusCode;
use warp::hyper::body::Bytes;
use warp::hyper::Body;
use warp::{Rejection, Reply};

use crate::models::{JsonError, Record};

pub async fn query_csv(
    csv_param: String,
    client: Arc<Client>,
) -> Result<Response<Body>, Rejection> {
    println!("csv_param: {}", csv_param);

    let (status, response) = match client
        .query_one(
            "SELECT id_str, a_int, opt_str, opt_float FROM my_table where id_str = $1",
            &[&csv_param],
        )
        .await
    {
        Ok(row) => {
            let id_str: String = row.get("id_str");
            let a_int: i32 = row.get("a_int");
            let opt_str: Option<String> = row.get("opt_str");
            let opt_float: Option<f64> = row.get("opt_float");

            let record = Record {
                id_str,
                a_int,
                opt_str,
                opt_float,
            };
            (StatusCode::OK, vec![record])
        }
        Err(error) => {
            eprintln!("type=error_from_query error={:?}", error);
            (StatusCode::NOT_FOUND, vec![])
        }
    };

    Ok(warp::reply::with_status(warp::reply::json(&response), status).into_response())
}

pub async fn transform_csv(
    request_body: Bytes,
    client: Arc<Client>,
) -> Result<Response<Body>, Rejection> {
    let bytes = request_body.to_vec();
    println!("request_body: {:?}", String::from_utf8_lossy(&bytes));

    //TODO: figure out how to stream this so that we don't need to have the entire request body in memory.
    //TODO: Also determine if we want to store "happy" records if a bad one is found.  I just bail here.
    let mut records = vec![];
    for record in csv::ReaderBuilder::new()
        // .has_headers(false) // can also do this depending on if it needs them
        .from_reader(bytes.as_slice())
        .deserialize::<Record>()
    {
        match record {
            Ok(record) => {
                records.push(record);
            }
            Err(error) => {
                let json_error = JsonError {
                    err: format!("type=csv_deserialization_error error={:?}", error,),
                };

                eprintln!("csv_deserialization_error: {:?}", error);

                return Ok(warp::reply::with_status(
                    warp::reply::json(&json_error),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
                .into_response());
            }
        }
    }

    //TODO: this isn't the prettiest db code, and it should/could use a prepared statement.
    let iterator =
        records
            .clone()
            .into_iter()
            .map(|record| match (record.opt_float, record.opt_str) {
                (Some(float), Some(string)) => client.query_raw(
                    "INSERT INTO my_table \
                                (id_str, a_int, opt_str, opt_float) \
                                VALUES ($1, $2, $3, $4)",
                    vec![
                        Box::new(record.id_str) as Box<dyn ToSql + Sync + Send>,
                        Box::new(record.a_int) as Box<dyn ToSql + Sync + Send>,
                        Box::new(string) as Box<dyn ToSql + Sync + Send>,
                        Box::new(float) as Box<dyn ToSql + Sync + Send>,
                    ],
                ),
                (Some(float), None) => client.query_raw(
                    "INSERT INTO my_table \
                                (id_str, a_int, opt_float) \
                                VALUES ($1, $2, $3)",
                    vec![
                        Box::new(record.id_str) as Box<dyn ToSql + Sync + Send>,
                        Box::new(record.a_int) as Box<dyn ToSql + Sync + Send>,
                        Box::new(float) as Box<dyn ToSql + Sync + Send>,
                    ],
                ),
                (None, Some(string)) => client.query_raw(
                    "INSERT INTO my_table \
                                (id_str, a_int, opt_str) \
                                VALUES ($1, $2, $3)",
                    vec![
                        Box::new(record.id_str) as Box<dyn ToSql + Sync + Send>,
                        Box::new(record.a_int) as Box<dyn ToSql + Sync + Send>,
                        Box::new(string) as Box<dyn ToSql + Sync + Send>,
                    ],
                ),
                (None, None) => client.query_raw(
                    "INSERT INTO my_table \
                                (id_str, a_int) \
                                VALUES ($1, $2)",
                    vec![
                        Box::new(record.id_str) as Box<dyn ToSql + Sync + Send>,
                        Box::new(record.a_int) as Box<dyn ToSql + Sync + Send>,
                    ],
                ),
            });

    match try_join_all(iterator).await {
        Ok(_) => Ok(
            warp::reply::with_status(warp::reply::json(&records), StatusCode::OK).into_response(),
        ),
        Err(error) => {
            let json_error = JsonError {
                err: format!("type=postgres_error error={:?}", error,),
            };

            eprintln!("postgres_error: {:?}", error);

            Ok(warp::reply::with_status(
                warp::reply::json(&json_error),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response())
        }
    }
}
