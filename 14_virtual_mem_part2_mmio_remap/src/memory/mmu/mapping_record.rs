// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2020-2022 Andre Richter <andre.o.richter@gmail.com>

//! A record of mapped pages.

// 今回追加されたファイル
// mappingを記憶することで，既存のmappingを再利用したり，mappingの内容を表示できるようにする．

use super::{
    AccessPermissions, Address, AttributeFields, MMIODescriptor, MemAttributes, MemoryRegion,
    Physical, Virtual,
};
use crate::{bsp, info, synchronization, synchronization::InitStateLock, warn};

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

/// Type describing a virtual memory mapping.
#[allow(missing_docs)]
#[derive(Copy, Clone)]
struct MappingRecordEntry {
    // 仮想memory mapping記述子
    // usersはpagesを使用しているdevice driverの名前
    pub users: [Option<&'static str>; 5],
    pub phys_start_addr: Address<Physical>,
    pub virt_start_addr: Address<Virtual>,
    pub num_pages: usize,
    pub attribute_fields: AttributeFields,
}

struct MappingRecord {
    // 仮想memory mapping記述子12個分
    inner: [Option<MappingRecordEntry>; 12],
}

//--------------------------------------------------------------------------------------------------
// Global instances
//--------------------------------------------------------------------------------------------------

static KERNEL_MAPPING_RECORD: InitStateLock<MappingRecord> =
    InitStateLock::new(MappingRecord::new());

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl MappingRecordEntry {
    pub fn new(
        // user名?
        name: &'static str,
        virt_region: &MemoryRegion<Virtual>,
        phys_region: &MemoryRegion<Physical>,
        attr: &AttributeFields,
    ) -> Self {
        Self {
            users: [Some(name), None, None, None, None],
            phys_start_addr: phys_region.start_addr(),
            virt_start_addr: virt_region.start_addr(),
            num_pages: phys_region.num_pages(),
            attribute_fields: *attr,
        }
    }

    // usersの中で未使用のuserを見つけて返す
    fn find_next_free_user(&mut self) -> Result<&mut Option<&'static str>, &'static str> {
        if let Some(x) = self.users.iter_mut().find(|x| x.is_none()) {
            return Ok(x);
        };

        Err("Storage for user info exhausted")
    }

    // 新しいuserを追加する
    pub fn add_user(&mut self, user: &'static str) -> Result<(), &'static str> {
        // 未使用のuserを見つける
        let x = self.find_next_free_user()?;
        // そこにuserを追加
        *x = Some(user);
        Ok(())
    }
}

impl MappingRecord {
    pub const fn new() -> Self {
        // 12個のNoneで初期化
        Self { inner: [None; 12] }
    }

    // 未使用のMappingRecordEntryを見つけて返す
    fn find_next_free(&mut self) -> Result<&mut Option<MappingRecordEntry>, &'static str> {
        if let Some(x) = self.inner.iter_mut().find(|x| x.is_none()) {
            return Ok(x);
        }

        Err("Storage for mapping info exhausted")
    }

    // 自身の中から引数で与えられるphys_pagesと同じDevice MMIO領域を表すものを探す
    fn find_duplicate(
        &mut self,
        phys_region: &MemoryRegion<Physical>,
    ) -> Option<&mut MappingRecordEntry> {
        self.inner
            .iter_mut()
            .filter(|x| x.is_some())
            .map(|x| x.as_mut().unwrap())
            .filter(|x| x.attribute_fields.mem_attributes == MemAttributes::Device)
            .find(|x| {
                if x.phys_start_addr != phys_region.start_addr() {
                    return false;
                }

                if x.num_pages != phys_region.num_pages() {
                    return false;
                }

                true
            })
    }

    // 新しいMappingRecordEntryを追加する
    pub fn add(
        &mut self,
        name: &'static str,
        virt_region: &MemoryRegion<Virtual>,
        phys_region: &MemoryRegion<Physical>,
        attr: &AttributeFields,
    ) -> Result<(), &'static str> {
        // 未使用のMappingRecordEntryを見つける
        let x = self.find_next_free()?;

        // そこに新しいMappingRecordEntryを追加する
        *x = Some(MappingRecordEntry::new(
            name,
            virt_region,
            phys_region,
            attr,
        ));
        Ok(())
    }

    // 全MappingRecordEntryを表示
    pub fn print(&self) {
        const KIB_RSHIFT: u32 = 10; // log2(1024).
        const MIB_RSHIFT: u32 = 20; // log2(1024 * 1024).

        // 表示項目
        info!("      -------------------------------------------------------------------------------------------------------------------------------------------");
        info!(
            "      {:^44}     {:^30}   {:^7}   {:^9}   {:^35}",
            "Virtual", "Physical", "Size", "Attr", "Entity"
        );
        info!("      -------------------------------------------------------------------------------------------------------------------------------------------");

        // 各MappingRecordEntryを表示
        for i in self.inner.iter().flatten() {
            let size = i.num_pages * bsp::memory::mmu::KernelGranule::SIZE;
            let virt_start = i.virt_start_addr;
            let virt_end_inclusive = virt_start + (size - 1);
            let phys_start = i.phys_start_addr;
            let phys_end_inclusive = phys_start + (size - 1);

            // 領域サイズ
            let (size, unit) = if (size >> MIB_RSHIFT) > 0 {
                (size >> MIB_RSHIFT, "MiB")
            } else if (size >> KIB_RSHIFT) > 0 {
                (size >> KIB_RSHIFT, "KiB")
            } else {
                (size, "Byte")
            };

            let attr = match i.attribute_fields.mem_attributes {
                MemAttributes::CacheableDRAM => "C", // Cacheable領域
                MemAttributes::Device => "Dev",      // Device MMIO領域
            };

            let acc_p = match i.attribute_fields.acc_perms {
                AccessPermissions::ReadOnly => "RO",  // Read Only
                AccessPermissions::ReadWrite => "RW", // Read Write
            };

            let xn = if i.attribute_fields.execute_never {
                "XN" // 実行不可
            } else {
                "X" // 実行可能
            };

            // MappingRecordEntryの情報を表示
            info!(
                "      {}..{} --> {}..{} | \
                        {: >3} {} | {: <3} {} {: <2} | {}",
                virt_start,
                virt_end_inclusive,
                phys_start,
                phys_end_inclusive,
                size,
                unit,
                attr,
                acc_p,
                xn,
                i.users[0].unwrap() // Device driver名
            );

            // その他のDevice driver名の表示
            for k in i.users[1..].iter() {
                if let Some(additional_user) = *k {
                    info!(
                        "                                                                                                            | {}",
                        additional_user
                    );
                }
            }
        }

        info!("      -------------------------------------------------------------------------------------------------------------------------------------------");
    }
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------
use synchronization::interface::ReadWriteEx;

/// Add an entry to the mapping info record.
/// 新しいMappingRecordEntryの追加
pub fn kernel_add(
    name: &'static str,
    virt_region: &MemoryRegion<Virtual>,
    phys_region: &MemoryRegion<Physical>,
    attr: &AttributeFields,
) -> Result<(), &'static str> {
    KERNEL_MAPPING_RECORD.write(|mr| mr.add(name, virt_region, phys_region, attr))
}

pub fn kernel_find_and_insert_mmio_duplicate(
    mmio_descriptor: &MMIODescriptor,
    new_user: &'static str,
) -> Option<Address<Virtual>> {
    // Device MMIO領域の物理page
    let phys_region: MemoryRegion<Physical> = (*mmio_descriptor).into();

    KERNEL_MAPPING_RECORD.write(|mr| {
        // mmio_descriptorと同じ領域を表すMappingRecordEntryを見つける
        let dup = mr.find_duplicate(&phys_region)?;
        // そこにnew_userを追加する
        if let Err(x) = dup.add_user(new_user) {
            warn!("{}", x);
        }
        // mmio_descriptorが表す領域の仮想開始addressを返す
        Some(dup.virt_start_addr)
    })
}

/// Human-readable print of all recorded kernel mappings.
/// kernel mappingsとして記述されている全MappingRecordEntryの情報を表示する
pub fn kernel_print() {
    KERNEL_MAPPING_RECORD.read(|mr| mr.print());
}
