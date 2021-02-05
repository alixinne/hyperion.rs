use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opts {}

#[paw::main]
fn main(opts: Opts) -> color_eyre::eyre::Result<()> {
    color_eyre::install()?;

    let db = hyperion::db::Db::try_default()?;
    let config = hyperion::models::Config::load(&db)?;


    println!("{:?}", config);

    Ok(())
}
