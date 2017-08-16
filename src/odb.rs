use std::marker;
use std::io;
use libc::{c_char};

use {raw, Oid, Object, Error};
use util::Binding;

/// A structure to represent a git ODB rstream
pub struct OdbReader<'repo> {
    raw: *mut raw::git_odb_stream,
    _marker: marker::PhantomData<Object<'repo>>,
}

impl<'repo> Binding for OdbReader<'repo> {
    type Raw = *mut raw::git_odb_stream;

    unsafe fn from_raw(raw: *mut raw::git_odb_stream) -> OdbReader<'repo> {
        OdbReader {
            raw: raw,
            _marker: marker::PhantomData,
        }
    }
    fn raw(&self) -> *mut raw::git_odb_stream { self.raw }
}

impl<'repo> Drop for OdbReader<'repo> {
    fn drop(&mut self) {
        unsafe { raw::git_odb_stream_free(self.raw) }
    }
}

impl<'repo> io::Read for OdbReader<'repo> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        unsafe {
            let ptr = buf.as_ptr() as *mut c_char;
            let len = buf.len();
            let res = raw::git_odb_stream_read(self.raw, ptr, len);
            if res < 0 {
                Err(io::Error::new(io::ErrorKind::Other, "Read error"))
            } else {
                Ok(len)
            }
        }
    }
}

/// A structure to represent a git ODB wstream
pub struct OdbWriter<'repo> {
    raw: *mut raw::git_odb_stream,
    _marker: marker::PhantomData<Object<'repo>>,
}

impl<'repo> OdbWriter<'repo> {
    /// Finalize ODB writing stream and write the object to the object database
    pub fn finalize(&mut self) -> Result<Oid, Error> {
        let mut raw = raw::git_oid { id: [0; raw::GIT_OID_RAWSZ] };
        unsafe {
            try_call!(raw::git_odb_stream_finalize_write(&mut raw, self.raw));
            Ok(Binding::from_raw(&raw as *const _))
        }
    }
}

impl<'repo> Binding for OdbWriter<'repo> {
    type Raw = *mut raw::git_odb_stream;

    unsafe fn from_raw(raw: *mut raw::git_odb_stream) -> OdbWriter<'repo> {
        OdbWriter {
            raw: raw,
            _marker: marker::PhantomData,
        }
    }
    fn raw(&self) -> *mut raw::git_odb_stream { self.raw }
}

impl<'repo> Drop for OdbWriter<'repo> {
    fn drop(&mut self) {
        unsafe { raw::git_odb_stream_free(self.raw) }
    }
}

impl<'repo> io::Write for OdbWriter<'repo> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unsafe {
            let ptr = buf.as_ptr() as *const c_char;
            let len = buf.len();
            let res = raw::git_odb_stream_write(self.raw, ptr, len);
            if res < 0 {
                Err(io::Error::new(io::ErrorKind::Other, "Write error"))
            } else {
                Ok(buf.len())
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

#[cfg(test)]
mod tests {
    use std::io::prelude::*;
    use tempdir::TempDir;
    use {Repository, ObjectType};

    #[test]
    #[ignore]
    fn reader() {
        let td = TempDir::new("test").unwrap();
        let repo = Repository::init(td.path()).unwrap();
        let dat = [4, 3, 5, 6, 9];
        let id = repo.blob(&dat).unwrap();
        let mut rs = repo.odb_reader(id).unwrap();
        let mut buf = [3];
        let rl = rs.read(&mut buf).unwrap();
        assert_eq!(rl, 3);
        assert_eq!(buf, &dat[0..3]);
        let rl = rs.read(&mut buf).unwrap();
        assert_eq!(rl, 2);
        assert_eq!(buf, &dat[3..5]);
    }

    #[test]
    fn writer() {
        let td = TempDir::new("test").unwrap();
        let repo = Repository::init(td.path()).unwrap();
        let dat = [4, 3, 5, 6, 9];
        let mut ws = repo.odb_writer(dat.len(), ObjectType::Blob).unwrap();
        let wl = ws.write(&dat[0..3]).unwrap();
        assert_eq!(wl, 3);
        let wl = ws.write(&dat[3..5]).unwrap();
        assert_eq!(wl, 2);
        let id = ws.finalize().unwrap();
        let blob = repo.find_blob(id).unwrap();
        assert_eq!(blob.content(), dat);
    }
}
