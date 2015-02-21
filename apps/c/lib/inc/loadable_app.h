#ifndef LOADABLE_APP_H
#define LOADABLE_APP_H

// Section location symbols created by compiler
extern unsigned int* _etext;
extern unsigned int* _edata;
extern unsigned int* _got;
extern unsigned int* _egot;
extern unsigned int* _plt;
extern unsigned int* _eplt;
extern unsigned int* _bss;
extern unsigned int* _ebss;

// Load Info is used by the runtime in order to load the application
//  Note that locations in the text section assume .text starts at 0x10000000
//  based on the ld file and are therefore updated
typedef struct {
    unsigned int* entry_loc;        /* Entry point for user application */
    unsigned int* init_data_loc;    /* Data initialization information in flash */
    unsigned int init_data_size;    /* Size of initialization information */
    unsigned int got_start_offset;  /* Offset to start of GOT */
    unsigned int got_end_offset;    /* Offset to end of GOT */
    unsigned int plt_start_offset;  /* Offset to start of PLT */
    unsigned int plt_end_offset;    /* Offset to end of PLT */
    unsigned int bss_start_offset;  /* Offset to start of BSS */
    unsigned int bss_end_offset;    /* Offset to end of BSS */
} Load_Info;

#define REGISTER_APP(name, init) \
    __attribute__ ((section(".load_info." #name), used)) \
    Load_Info app_info = { \
        .entry_loc          = (unsigned int*)((unsigned int)init - 0x10000000), \
        .init_data_loc      = (unsigned int*)((unsigned int)(&_etext) - 0x10000000), \
        .init_data_size     = (unsigned int)(&_edata), \
        .got_start_offset   = (unsigned int)(&_got), \
        .got_end_offset     = (unsigned int)(&_egot), \
        .plt_start_offset   = (unsigned int)(&_plt), \
        .plt_end_offset     = (unsigned int)(&_eplt), \
        .bss_start_offset   = (unsigned int)(&_bss), \
        .bss_end_offset     = (unsigned int)(&_ebss), \
    }

#endif
