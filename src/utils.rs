use bollard::Docker;
use color_eyre::Result;

fn driver_connector() -> Result<Docker> {
    Ok(Docker::connect_with_local_defaults()?)
}
