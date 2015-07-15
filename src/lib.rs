#![feature(no_std, core, core_prelude, asm, raw)]
#![no_std]

#![crate_name = "raw_cpuid"]
#![crate_type = "lib"]

#[macro_use]
extern crate core;

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
#[prelude_import]
use std::prelude::v1::*;

use core::prelude::*;
use core::iter;
use core::raw;
use core::str;
use core::mem::{transmute};
use core::fmt;
use core::slice;

const MAX_ENTRIES: usize = 32;

#[macro_export]
macro_rules! cpuid {
    ($eax:expr)
        => ( $crate::cpuid1($eax as u32) );

    ($eax:expr, $ecx:expr)
        => ( $crate::cpuid2($eax as u32, $ecx as u32) );

}

fn cpuid2(eax: u32, ecx: u32) -> CpuIdResult {
    let mut res = CpuIdResult{eax: 0, ebx: 0, ecx: 0, edx: 0};

    unsafe {
        asm!("movl $0, %eax" : : "r" (eax) : "eax");
        asm!("movl $0, %ecx" : : "r" (ecx) : "ecx");
        asm!("cpuid" : "={eax}"(res.eax) "={ebx}"(res.ebx)
                       "={ecx}"(res.ecx) "={edx}"(res.edx)
                     :: "eax", "ebx", "ecx", "edx");
    }

    res
}

fn cpuid1(eax: u32) -> CpuIdResult {
    let mut res = CpuIdResult{eax: 0, ebx: 0, ecx: 0, edx: 0};

    unsafe {
        asm!("movl $0, %eax" : : "r" (eax) : "eax");
        asm!("cpuid" : "={eax}"(res.eax) "={ebx}"(res.ebx)
                       "={ecx}"(res.ecx) "={edx}"(res.edx)
                     :: "eax", "ebx", "ecx", "edx");
    }

    res
}

enum CpuIdLeaf {
    VendorInformation,
    FeatureInformation,
    CacheInformation,
    ProcessorSerial,
    CacheParameters,
    MonitorMwait,
    ThermalPowerManagement,
    StructuredExtendedFeature,
    DirectCacheAccess,
    PerformanceMonitoring,
    ExtendedTopology,
    ProcessorExtendedState,
    QualityofService,
    ExtendedFunction,
}

struct LeafData(CpuIdLeaf, &'static str, u32);

const LEAF_INFORMATION: [LeafData; 14] = [
    LeafData(CpuIdLeaf::VendorInformation, "GenuineIntel", 0x0),
    LeafData(CpuIdLeaf::FeatureInformation, "Version/Feature Information", 0x1),
    LeafData(CpuIdLeaf::CacheInformation, "Cache and TLB Information", 0x2),
    LeafData(CpuIdLeaf::ProcessorSerial, "Processor serial number", 0x3),
    LeafData(CpuIdLeaf::CacheParameters, "Deterministic Cache Parameters", 0x4),
    LeafData(CpuIdLeaf::MonitorMwait, "MONITOR/MWAIT", 0x5),
    LeafData(CpuIdLeaf::ThermalPowerManagement, "Thermal and Power Management", 0x6),
    LeafData(CpuIdLeaf::StructuredExtendedFeature, "Structured Extended Feature Flags", 0x7),
    LeafData(CpuIdLeaf::DirectCacheAccess, "Direct Cache Access Information", 0x9),
    LeafData(CpuIdLeaf::PerformanceMonitoring, "Architectural Performance Monitoring", 0xA),
    LeafData(CpuIdLeaf::ExtendedTopology, "Extended Topology Enumeration", 0xB),
    LeafData(CpuIdLeaf::ProcessorExtendedState, "Processor Extended State Enumeration", 0xD),
    LeafData(CpuIdLeaf::QualityofService, "Quality of Service", 0xF),
    LeafData(CpuIdLeaf::ExtendedFunction, "Extended Function CPUID Information", 0x80000000),
];

#[derive(Debug)]
pub struct CpuId;

#[derive(Debug, Copy, Clone)]
pub struct CpuIdResult {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32
}

impl CpuId {

    pub fn get_vendor_information(&self) -> CpuIdVendorInfo {
        let res = cpuid!(0);
        CpuIdVendorInfo{ebx: res.ebx, ecx: res.ecx, edx: res.edx}
    }

    pub fn get_feature_information(&self) -> CpuIdFeatureInfo {
        let res = cpuid!(1);
        CpuIdFeatureInfo{eax: res.eax, ebx: res.ebx, ecx: res.ecx, edx: res.edx}
    }

}

pub struct CpuIdVendorInfo {
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32
}

fn as_bytes(v: &u32) -> &[u8] {
    let start = v as *const u32 as *const u8;
    unsafe { slice::from_raw_parts(start, 4) }
}

impl fmt::Display for CpuIdVendorInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe {
            write!(f, "{}{}{}",
               str::from_utf8_unchecked(as_bytes(&self.ebx)),
               str::from_utf8_unchecked(as_bytes(&self.edx)),
               str::from_utf8_unchecked(as_bytes(&self.ecx)))
        }
    }
}

pub struct CpuIdFeatureInfo {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32
}

impl CpuIdFeatureInfo {

    pub fn get_extended_family_id(&self) -> u8 {
        ((self.eax >> 20) & 0xff) as u8
    }

    pub fn get_extended_model_id(&self) -> u8 {
        ((self.eax >> 16) & 0b1111) as u8
    }

    pub fn get_family_id(&self) -> u8 {
        ((self.eax >> 8) & 0b1111) as u8
    }

    pub fn get_model(&self) -> u8 {
        ((self.eax >> 4) & 0b1111) as u8
    }

    pub fn get_stepping_id(&self) -> u8 {
        ((self.eax & 0b1111)) as u8
    }

    pub fn get_brand_index(&self) -> u8 {
        (self.ebx) as u8
    }

    pub fn get_cflush_cache_line_size(&self) -> u8 {
        ((self.ebx >> 8)) as u8
    }

    pub fn get_local_apic_id(&self) -> u8 {
        ((self.ebx >> 24)) as u8
    }
}

#[cfg(test)]
#[test]
fn genuine_intel() {
    let cpu: CpuId = CpuId;
    let vinfo = cpu.get_vendor_information();
    println!("{}", vinfo);

    // GenuineIntel
    assert!(vinfo.ebx == 0x756e6547);
    assert!(vinfo.edx == 0x49656e69);
    assert!(vinfo.ecx == 0x6c65746e);
}

#[cfg(test)]
#[test]
fn feature_info() {
    let cpu: CpuId = CpuId;
    let finfo = cpu.get_feature_information();

    println!("{}", finfo.get_extended_family_id());
    println!("{}", finfo.get_extended_model_id());
    println!("{}", finfo.get_family_id());
    println!("{}", finfo.get_model());
    println!("{}", finfo.get_stepping_id());
    println!("{}", finfo.get_brand_index());
    println!("{}", finfo.get_cflush_cache_line_size());
    println!("{}", finfo.get_local_apic_id());
}