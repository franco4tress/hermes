/* STM32H753ZI memory map
 *
 * FLASH  : 2 MiB  starting at 0x0800_0000
 * DTCMRAM: 128 KiB starting at 0x2000_0000  (used as main RAM / stack)
 *
 * Larger memories (AXI SRAM at 0x2400_0000 etc.) can be added as additional
 * regions if needed, but cortex-m-rt only requires FLASH + RAM.
 */
MEMORY
{
    FLASH  (rx)  : ORIGIN = 0x08000000, LENGTH = 2M
    RAM    (xrw) : ORIGIN = 0x20000000, LENGTH = 128K
}
