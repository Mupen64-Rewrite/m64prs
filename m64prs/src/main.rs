use std::error::Error;



fn main() -> Result<(), Box<dyn Error>> {
    Err(m64prs_core::types::m64p_error::M64ERR_SUCCESS)?;
    Ok(())
}
