[Here's the English version](https://github.com/MimiRabbit87/tuff_virtual_machine/blob/master/README.md)
# 凝灰岩虚拟机

凝灰岩虚拟机是 CHIP-8 解释器的一个 Rust 实现。

## 什么是凝灰岩？

凝灰岩是一个使用 Rust 编写的 CHIP-8 工具链套组，作为我的练手项目，而凝灰岩虚拟机是工具链的其中一个。

## 实现

- 完整的 CHIP-8 指令集
- 可调节的 CPU 频率（默认 500Hz）
- 实时的调试信息（包含寄存器、调用栈、程序计数器、各计时器等）
- 音频输出
- 窗口渲染
- 支持 `.ch8` ROM 文件

## 开始

### 下载发布构建

查看 [Releases](https://github.com/MimiRabbit87/tuff_virtual_machine/releases) 页面获取预构建二进制文件（目前仅有 Windows 64位 平台），
如果没有你的平台，可以自行从源码构建。
然后输入 `path/to/tuff_virtual_machine -h` 来看看你能用它做些什么。

## 开发者说明

### 从源码构建

#### 准备

- Rust（推荐稳定发布频道的最新工具链）
- Cargo

#### 克隆仓库

```bash
git clone git@github.com:MimiRabbit87/tuff_virtual_machine.git
cd tuff_virtual_machine
```

#### 运行

```bash
cd path/to/local_repository_cloned
cargo run -- -r path/to/your_rom_file.ch8
```

### 构建

构建调试版本：

```bash
cd path/to/local_repository_cloned
cargo build
```

构建发布版本（更佳、更快，推荐日常使用）：

```bash
cd path/to/local_repository_cloned
cargo build --release
```

## 命令行选项

|选项|描述|
|----|----|
|`-r, --run <PATH>`|要运行的 CHIP-8 ROM 文件路径|
|`-f, --frequency <HZ>`|设置 CPU 频率（指令/秒），默认 `500`。输入 `0` 来取消频率限制（不稳定）|
|`-i, --with-debug-information`|在终端上显示调试信息（寄存器、调用栈、计时器等|
|`--no-vertical-synchronization`|禁用 `60Hz` 的帧同步（即禁用模拟垂直同步，在高 CPU 频率下可能会导致渲染过快）|
|`-h, --help`|打印帮助信息（用 `--help` 查看详细信息）|
|`-V, --version`|打印版本信息|

## 调优

 - 以 `--release` 构建发布版来获得更好的体验。

 - 高于 `2KHz` 的频率可能导致高 CPU 占用和系统不稳定，请谨慎使用。

## 测试

凝灰岩虚拟机通过了以下 [Timendus CHIP-8 测试集](https://github.com/Timendus/chip8-test-suite)：

 - 1-chip8-logo.ch8

 - 2-ibm-logo.ch8

 - 3-corax+.ch8

 - 4-flags.ch8

 - 5-quirks.ch8

 - 6-keypad.ch8

 - 7-beep.ch8

## 贡献

尽管提 Issue 开 PR 就行，欢迎贡献。

## 开源协议

本项目以 MIT 协议开源，查看 [LICENSE](https://github.com/MimiRabbit87/tuff_virtual_machine/blob/masterLICENSE) 文件获取更多细节。
