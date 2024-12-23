use std::io;

mod sealed {
    pub trait Sealed: Sized {}
}

pub trait GlibErrorExt: sealed::Sealed {
    fn try_into_io_error(self) -> Result<io::Error, Self>;
}

impl sealed::Sealed for glib::Error {}
impl GlibErrorExt for glib::Error {
    fn try_into_io_error(self) -> Result<io::Error, Self> {
        let glib_kind = match self.kind::<glib::FileError>() {
            Some(kind) => kind,
            None => return Err(self),
        };

        let rust_kind = match glib_kind {
            glib::FileError::Exist => io::ErrorKind::AlreadyExists,
            glib::FileError::Isdir => io::ErrorKind::IsADirectory,
            glib::FileError::Acces => io::ErrorKind::PermissionDenied,
            glib::FileError::Nametoolong => io::ErrorKind::InvalidInput,
            glib::FileError::Noent => io::ErrorKind::NotFound,
            glib::FileError::Notdir => io::ErrorKind::NotADirectory,
            glib::FileError::Nxio => io::ErrorKind::NotFound,
            glib::FileError::Nodev => io::ErrorKind::Unsupported,
            glib::FileError::Rofs => io::ErrorKind::ReadOnlyFilesystem,
            glib::FileError::Txtbsy => io::ErrorKind::ResourceBusy,
            glib::FileError::Fault => io::ErrorKind::Other,
            glib::FileError::Loop => io::ErrorKind::Other,
            glib::FileError::Nospc => io::ErrorKind::StorageFull,
            glib::FileError::Nomem => io::ErrorKind::OutOfMemory,
            glib::FileError::Mfile => io::ErrorKind::Other,
            glib::FileError::Nfile => io::ErrorKind::Other,
            glib::FileError::Badf => io::ErrorKind::InvalidInput,
            glib::FileError::Inval => io::ErrorKind::InvalidInput,
            glib::FileError::Pipe => io::ErrorKind::BrokenPipe,
            glib::FileError::Again => io::ErrorKind::Other,
            glib::FileError::Intr => io::ErrorKind::Interrupted,
            glib::FileError::Io => io::ErrorKind::Other,
            glib::FileError::Perm => io::ErrorKind::PermissionDenied,
            glib::FileError::Nosys => io::ErrorKind::Unsupported,
            glib::FileError::Failed => io::ErrorKind::Other,
            _ => todo!(),
        };

        Ok(io::Error::new(rust_kind, self))
    }
}