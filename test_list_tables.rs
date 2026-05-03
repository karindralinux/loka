use tokio_postgres::NoTls;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (client, connection) = tokio_postgres::connect(
        "host=127.0.0.1 port=5432 user=postgres password= dbname=maia-local",
        NoTls,
    )
    .await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let schema = "public";
    let rows = client
        .query(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = $1 AND table_type = 'BASE TABLE' ORDER BY table_name",
            &[&schema],
        )
        .await?;

    println!("Total rows: {}", rows.len());

    let tables: Vec<String> = rows
        .into_iter()
        .filter_map(|r| {
            let res = r.try_get::<_, String>(0);
            if let Err(e) = &res {
                println!("Error extracting row: {}", e);
            }
            res.ok()
        })
        .collect();

    println!("Extracted tables: {:?}", tables);
    Ok(())
}
