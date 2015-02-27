use core::prelude::*;
use core::intrinsics;

use util;

#[repr(C, packed)]
#[allow(dead_code)]
struct Load_Info {
    entry_loc:          u32,
    init_data_loc:      u32,
    init_data_size:     u32,
    got_start_offset:   u32,
    got_end_offset:     u32,
    plt_start_offset:   u32,
    plt_end_offset:     u32,
    bss_start_offset:   u32,
    bss_end_offset:     u32,
}

extern {
    static _apps: u32;
    static _eapps: u32;
    static _end: u32;
}

pub unsafe fn load_apps() {
    util::println("In load apps");

    let start_ptr = &_apps as *const u32;
    util::print_num(start_ptr as u32);
    let end_ptr = &_eapps as *const u32;
    util::print_num(end_ptr as u32);

    // iterate through each pre-loaded app
    let mut app_ptr = start_ptr;
    let mut app_data_start = &_end as *const u32; //XXX: Needs to be allocated
    util::print_num(app_data_start as u32);
    util::println("\n");
    while app_ptr < end_ptr {
        let app_info: &'static Load_Info = intrinsics::transmute(app_ptr);

        util::println("Load Info Struct");
        util::print_num(app_ptr as u32);
        util::print_num(app_info.entry_loc as u32);
        util::print_num(app_info.init_data_loc as u32);
        util::print_num(app_info.plt_end_offset as u32);
        util::print_num(app_info.bss_end_offset as u32);
        util::println("--------");

        // copy data section from Flash to SRAM
        util::println("Data");
        let init_data_src: *const u32 = (app_info.init_data_loc + (app_ptr as u32)) as *const u32;
        let init_data_end: *const u32 = ((init_data_src as u32) + app_info.init_data_size) as *const u32;
        let mut data_src: *mut u32 = init_data_src as *mut u32;
        let mut data_dst: *mut u32 = app_data_start as *mut u32;
        while (data_src as *const u32) < init_data_end {
            *data_dst = *data_src;
            util::print_num(*data_src as u32);
            data_dst = data_dst.offset(1);
            data_src = data_src.offset(1);
        }
        util::println("--------");

        // zero out bss section
        util::println("BSS");
        let mut bss_start: *mut u32 = ((app_data_start as u32) + app_info.bss_start_offset) as *mut u32;
        let bss_size: u32 = app_info.bss_end_offset - app_info.bss_start_offset;
        let bss_end: *const u32 = ((bss_start as u32) + bss_size) as *const u32;
        while (bss_start as *const u32) < bss_end {
            *bss_start = 0;
            util::print_num(bss_start as u32);
            bss_start = bss_start.offset(1);
        }
        util::println("--------");

        // fixup Global Offset Table (GOT)
        util::println("GOT");
        let mut got_start: *mut u32 = ((app_data_start as u32) + app_info.got_start_offset) as *mut u32;
        let got_size: u32 = app_info.got_end_offset - app_info.got_start_offset;
        let got_end: *const u32 = ((got_start as u32) + got_size) as *const u32;
        while (got_start as *const u32) < got_end {
            let mut val: u32 = *got_start;
            if val >= 0x10000000 {
                // fixup for const data in flash
                val -= 0x10000000;
                val += app_ptr as u32;
            } else {
                // fixup for data in sram
                val += app_data_start as u32;
            }
            *got_start = val;
            util::print_num(val as u32);
            got_start = got_start.offset(1);
        }
        util::println("--------");

        //TODO: fixup Procedure Linkage Table (PLT)

        //TODO create task from app

        // move to next app
        app_ptr = init_data_end;
        app_data_start = bss_end; //XXX: Needs to be allocated
        util::print_num(app_ptr as u32);
        util::print_num(app_data_start as u32);
        util::println("");
    }
}

