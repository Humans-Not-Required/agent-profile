use agent_profile::db;
use agent_profile::create_rocket;

#[rocket::main]
#[allow(clippy::result_large_err)]
async fn main() -> Result<(), rocket::Error> {
    let db_path = db::get_db_path();
    let _rocket = create_rocket(&db_path).launch().await?;
    Ok(())
}
