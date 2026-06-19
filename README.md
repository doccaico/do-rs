### Do (Windows only .-.)

### Required Softwares
- busybox64u
- curl
- git
- less
- rg
- tar

### Build and Minimizing Binary Size
```sh
$ cargo build --release
$ upx --best --lzma do.exe
```

### Clippy
```sh
$ cargo clippy 2>&1 | less
```

### Memo
- [Color](https://stackoverflow.com/questions/6297072/color-for-the-prompt-just-the-prompt-proper-in-cmd-exe-and-powershell)
