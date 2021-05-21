# Paging
- A page is 4KiB.
- A page entry is 8B.
- A page has 512 entries. (we fit one page table in one page)
- x86_64 only supports 52b physical addresses
- Page (table) entry contains physical address of page.
- Physical address represented on 40b
  [ addr % 4KiB ~= 52b - 12b ]
- entry layout `flags[0:8] . free[9:11] . phys_addr[12:51] . free[52:62] . NXE[63]`

- **flags**
    - [0] present differentiates mapped/unmapped pages. *fun fact: swapped out when copied to disk (oom), page fault will occur on acces, OS can then re-load the page.
    - [1] writable
    - [2] user access, if zero then only kernel mode can access
    - [3] write through caching, no cache is used for this page
    - [4] disable cache
    - [5] accessed, set by CPU when used
    - [6] dirty, set when a write occurred
    - [7] huge page/null, allows creation of larger pages (for level 2/3 table). 
      addr then directly maps to memory. (512 * 4KiB = 2MiB pages for level 2, 1GiB for level 3)
    - [8] available in all addr. spaces (cf. TLB)
    - [63] no execute

## Translation Lookaside Buffer (TLB)
- caches address translations (4 mem. accesses are required) in CPU
- not fully transparent, does not update/remove when contents change
- kernel must manually update TLB (`invlpg` instr.)