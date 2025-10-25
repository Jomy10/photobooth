use ouroboros::self_referencing;
use sysinfo::DiskRefreshKind;

#[self_referencing]
pub struct StorageDevices {
    disks: sysinfo::Disks,
    #[borrows(disks)]
    #[covariant]
    mountable_disks: Vec<&'this sysinfo::Disk>
}

impl StorageDevices {
    /// Collects the current storage devices
    pub fn collect() -> StorageDevices {
        StorageDevices::new(
            sysinfo::Disks::new_with_refreshed_list_specifics(DiskRefreshKind::everything().with_storage()),
            |disks| disks.iter()
                        .filter(|disk| disk.is_removable() && !disk.is_read_only())
                        .collect()
        )
    }

    pub fn drives(&self) -> &Vec<& sysinfo::Disk> {
        self.borrow_mountable_disks()
    }
}
