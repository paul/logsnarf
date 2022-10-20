use tracing::debug;

use logsnarf::{settings::Settings, util};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    util::setup()?;
    let settings = Settings::new()?;
    println!("Hello, again world!");
    debug!("{:?}", settings);

    util::teardown()?;
    Ok(())
}
