use std::sync::Arc;

use tokio_postgres::{Config, NoTls};
use warp::Filter;

mod models;
mod service;

//TODO: literally any kind of logging would be nice, as well as any kind of metrics.

#[tokio::main]
pub async fn main() {
    // Connect to the database.
    // TODO: for quick and dirty, we just run a local postgres server.  If you want to run this,
    // TODO: make sure this user name and password exists and has the "CreateDB" permission for the role.
    let (client, connection) = Config::new()
        .host("localhost")
        .password("password")
        .user("postgres")
        .connect(NoTls)
        .await
        .unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let client = Arc::new(client);

    client
        .query(
            "\
            CREATE TABLE IF NOT EXISTS my_table ( \
            id_str VARCHAR(450) NOT NULL,\
            a_int INTEGER NOT NULL,\
            opt_str VARCHAR(450),\
            opt_float DOUBLE PRECISION,\
            PRIMARY KEY (id_str)\
            )\
            ",
            &[],
        )
        .await
        .expect("unable to create table?");

    let client_clone = client.clone();
    let query_route = warp::get()
        .and(warp::path("csv"))
        .and(warp::path::param())
        .and(warp::path::end())
        .and(warp::any().map(move || client_clone.clone()))
        .and_then(service::query_csv);

    let transform_route = warp::post()
        .and(warp::path("csv"))
        .and(warp::path::end())
        .and(warp::body::bytes())
        .and(warp::any().map(move || client.clone()))
        .and_then(service::transform_csv);

    let routes = transform_route.or(query_route);
    let port = 12345;
    let service_fut = warp::serve(routes).run(([0, 0, 0, 0], port));

    service_fut.await;
}
