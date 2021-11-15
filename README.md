# Summary

This is a very simple tool that prints out function, global, and source line
information from a `.dbg` "DI" COFF debug file.

This can handle both `DI` magic files and CAB (cabinet) files with `DI` files
inside of them.

# Format

This outputs a format:

```
F <addr> <function>
G <addr> <global>
S <addr> <source>:<line>
```

# Binary Ninja Plugin

Included is a `binaryninja` plugin. Copy the folder `binaryninja/dbg_load` to
your `~/.binaryninja/plugins` folder.

Then install this tool to your `PATH` by `cargo install --path .`.

You can then use `Tools > Plugins > Load COFF DBG File` to load a `.DBG` or
`.DB_` file into your program. For best results turn off auto-analysis and
load the symbols before analyzing the binary as it will let Binary Ninja know
exactly where functions are!

![Binary Ninja screenshot showing symbolized and typed output](/binaryninja/example.png)

# Example

```
cargo run --release /home/pleb/nt/isos/fre/SUPPORT/DEBUG/MIPS/SYMBOLS/EXE/WRITE.DB_
F 000010dc WinMain
F 000011fc __F3_$WinMainCRTStartup
F 00001220 WinMainCRTStartup
F 00001434 _XcptFilter
F 00001444 __C_specific_handler
F 00001454 _setargv
F 0000145c _matherr
F 00001464 _initterm
G 00000000 header
G 00001000 __imp_GetModuleHandleA
G 00001004 __imp_GetStartupInfoA
G 00001008 __imp_GetCommandLineA
G 0000100c KERNEL32_NULL_THUNK_DATA
G 00001010 __imp__fmode
G 00001014 __imp_exit
G 00001018 __imp___setusermatherr
G 0000101c __imp__exit
G 00001020 __imp__acmdln
G 00001024 __imp___getmainargs
G 00001028 __imp__initterm
G 0000102c __imp__commode
G 00001030 __imp___set_app_type
G 00001034 __imp__XcptFilter
G 00001038 __imp___C_specific_handler
G 0000103c MSVCRT_NULL_THUNK_DATA
G 00001040 __imp_ShellExecuteA
G 00001044 SHELL32_NULL_THUNK_DATA
G 000010dc .text
G 000011fc .text
G 00001454 .text
G 0000145c .text
G 00001474 ___S2_$WinMainCRTStartupd
G 00001488 __IMPORT_DESCRIPTOR_SHELL32
G 0000149c __IMPORT_DESCRIPTOR_KERNEL32
G 000014b0 __IMPORT_DESCRIPTOR_MSVCRT
G 000014c4 __NULL_IMPORT_DESCRIPTOR
G 00001530 .idata$6
G 00001560 .idata$6
G 00001606 .idata$6
G 00002000 __xc_a
G 00002004 __xc_z
G 00002008 __xi_a
G 0000200c __xi_z
G 00002010 _$$1$d1
G 0000201c _commode
G 00002020 _dowildcard
G 00002024 _fmode
G 00002028 _newmode
G 0000202c __defaultmatherr
G 00002030 __onexitbegin
G 00002034 __onexitend
G 00004120 .rsrc$02
G 00006000 end
S 000010dc D:\nt\private\windows\shell\accesory\write\write.c:6
S 000010e8 D:\nt\private\windows\shell\accesory\write\write.c:8
S 000010f8 D:\nt\private\windows\shell\accesory\write\write.c:10
S 000010fc D:\nt\private\windows\shell\accesory\write\write.c:8
S 00001104 D:\nt\private\windows\shell\accesory\write\write.c:10
S 00001108 D:\nt\private\windows\shell\accesory\write\write.c:37
S 0000110c D:\nt\private\windows\shell\accesory\write\write.c:16
S 0000113c D:\nt\private\windows\shell\accesory\write\write.c:21
S 00001144 D:\nt\private\windows\shell\accesory\write\write.c:22
S 00001148 D:\nt\private\windows\shell\accesory\write\write.c:24
S 0000114c D:\nt\private\windows\shell\accesory\write\write.c:23
S 00001150 D:\nt\private\windows\shell\accesory\write\write.c:25
S 0000115c D:\nt\private\windows\shell\accesory\write\write.c:26
S 00001170 D:\nt\private\windows\shell\accesory\write\write.c:27
S 00001174 D:\nt\private\windows\shell\accesory\write\write.c:32
S 00001184 D:\nt\private\windows\shell\accesory\write\write.c:34
S 00001188 D:\nt\private\windows\shell\accesory\write\write.c:33
S 0000118c D:\nt\private\windows\shell\accesory\write\write.c:34
S 0000119c D:\nt\private\windows\shell\accesory\write\write.c:37
S 000011a0 D:\nt\private\windows\shell\accesory\write\write.c:36
S 000011a4 D:\nt\private\windows\shell\accesory\write\write.c:37
S 000011b0 D:\nt\private\windows\shell\accesory\write\write.c:40
S 000011ec D:\nt\private\windows\shell\accesory\write\write.c:42
S 000011fc crtexe.c:345
S 00001220 crtexe.c:175
S 00001228 crtexe.c:199
S 0000123c crtexe.c:209
S 00001244 crtexe.c:214
S 00001254 crtexe.c:209
S 00001264 crtexe.c:214
S 00001268 crtexe.c:215
S 00001278 crtexe.c:234
S 0000127c crtexe.c:215
S 00001280 crtexe.c:241
S 0000128c crtexe.c:242
S 000012a4 crtexe.c:251
S 000012b8 crtexe.c:268
S 000012c0 crtexe.c:266
S 000012c8 crtexe.c:268
S 000012e8 crtexe.c:266
S 000012ec crtexe.c:274
S 00001300 crtexe.c:287
S 00001308 crtexe.c:274
S 0000130c crtexe.c:317
S 00001310 crtexe.c:287
S 00001314 crtexe.c:290
S 00001320 crtexe.c:296
S 00001350 crtexe.c:301
S 00001358 crtexe.c:302
S 0000135c crtexe.c:304
S 00001360 crtexe.c:303
S 00001364 crtexe.c:305
S 00001370 crtexe.c:306
S 00001384 crtexe.c:312
S 00001394 crtexe.c:314
S 00001398 crtexe.c:313
S 0000139c crtexe.c:314
S 000013ac crtexe.c:317
S 000013b0 crtexe.c:316
S 000013b4 crtexe.c:317
S 000013c0 crtexe.c:330
S 000013f8 crtexe.c:343
S 0000140c crtexe.c:344
S 00001414 crtexe.c:350
S 00001428 crtexe.c:354
S 00001454 dllargv.c:49
S 0000145c merr.c:33
```

