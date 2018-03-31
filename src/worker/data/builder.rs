use std::fs::File;
use super::data::{Data, Storage};
use errors::Result;
use super::super::fs::workdir::WorkDir;
use common::DataType;
use worker::fs::tempfile::TempFileName;

enum BuilderStorage {
    Memory(Vec<u8>),
    File((File, TempFileName))
}

pub struct DataBuilder {
    storage: BuilderStorage,
    data_type: DataType,
}

impl DataBuilder {
    pub fn new(workdir: &WorkDir, data_type: DataType, expected_size: Option<usize>) -> Self {

        fn file_storage(workdir: &WorkDir) -> BuilderStorage {
            let f = workdir.make_temp_file();
            BuilderStorage::File((File::open(f.path()).unwrap(), f))
        }

        let storage = if let Some(size) = expected_size {
            if size < 256 * 1024 {
                BuilderStorage::Memory(Vec::with_capacity(size))
            } else {
                file_storage(workdir)
            }
        } else {
            file_storage(workdir)
        };
        DataBuilder { data_type, storage }
    }

    // TODO: Get rid of this method
    pub fn write_blob(&mut self, data: &Data) -> Result<()> {
        assert!(self.data_type == DataType::Blob && data.is_blob());
        match data.storage() {
            &Storage::Memory(ref bytes) => self.write(&bytes[..]),
            &Storage::Path(ref path) => {
                let mem = unsafe { ::memmap::Mmap::map(&File::open(&path.path)?) }?;
                self.write(&mem);
            }
        }
        Ok(())
    }

    pub fn write(&mut self, data: &[u8]) {
        match self.storage {
            BuilderStorage::Memory(ref mut buffer) => buffer.extend_from_slice(data),
            BuilderStorage::File(ref file, _) => file.write_all(data),
        }
    }

    pub fn build(&mut self) -> Data {
        match self.storage {
            BuilderStorage::Memory(ref mut buffer) => Data::new(
                Storage::Memory(::std::mem::replace(buffer, Vec::new())),
                self.data_type,
            )
        }
    }
}
