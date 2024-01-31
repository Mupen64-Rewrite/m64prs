use std::error::Error;

use m64prs_core::Core;



fn main() -> Result<(), Box<dyn Error>> {
    Core::load("/usr/lib/libmupen64plus.so.2")?;
    let core = Core::get();

    Ok(())
}
