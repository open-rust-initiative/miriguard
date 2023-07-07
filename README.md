# miriguard

## 0. 预备条件

- 已安装 rust nightly 工具链
  - 可以使用这个命令安装：`rustup toolchain install nightly`
- 已安装 miri 检查工具
  - 可以使用这个命令安装：`rustup component add miri`

## 1. 安装

步骤如下：
1. 下载 miriguard 项目源码，进入源码目录
2. 安装 miriguard 工具: `cargo install --path .`

## 2. 使用帮助

可以通过 `miriguard -h` 来查看帮助

```bash
$ miriguard -h
Usage: miriguard <COMMAND>

Commands:
  run
  test
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

`miriguard` 支持两个执行命令 `run` 和 `test`

`miriguard run` 的使用帮助如下：

```bash
$ miriguard run -h
Usage: miriguard run [OPTIONS]

Options:
      --bin <BIN>
      --example <EXAMPLE>
  -h, --help               Print help
  -V, --version            Print version
```

`miriguard test` 的使用帮助如下：

```bash
$ miriguard test -h 
Usage: miriguard test [TESTNAME]

Arguments:
  [TESTNAME]

Options:
  -h, --help     Print help
  -V, --version  Print version
```

说明：Miriguard 底层是基于 miri 进行检测，并对输出结果进行解析。

## 3. 检测结果说明

下面是 `miriguard` 检测输出的一个样例：

```
Error: error with using invalid raw pointer >>>>>
error: Undefined Behavior: pointer to alloc61791 was dereferenced after this allocation got freed
  --> src/lib.rs:23:29
   |
23 |     println!("{}", unsafe { *p });
   |                             ^^ pointer to alloc61791 was dereferenced after this allocation got freed
   |
   = help: this indicates a bug in the program: it performed an invalid operation, and caused Undefined Behavior
   = help: see https://doc.rust-lang.org/nightly/reference/behavior-considered-undefined.html for further information
   = note: BACKTRACE:
   = note: inside `tests::access_returned_stack_address` at src/lib.rs:23:29: 23:31
note: inside closure
  --> src/lib.rs:16:38
   |
15 |   #[test]
   |   ------- in this procedural macro expansion
16 |   fn access_returned_stack_address() {
   |                                      ^
   = note: this error originates in the attribute macro `test` (in Nightly builds, run with -Z macro-backtrace for more info)
<<<<<
```

- 输出结果的第一行为 `miriguard` 的分析结果，主要是用作辅助映射到 Rust 华为编程规范。
- `>>>>>` 和 `<<<<<<` 之间的输出结果为 miri 的原始输出结果，可用于分析具体原因。

## 4. 工具使用提醒

miriguard 的检测原理是通过软件的方式模拟程序在物理机器上的执行过程，并在模拟执行过程中检测可能出现内存安全的操作。
这就导致了 miriguard 的检测效率要比程序的实际执行效率低很多。

因此，在实际的使用过程中，尽量不要使用 miriguard 来直接 `run` 一个较大的项目。
而建议通过 `miriguard test` 的方式对项目的核心模块的单元测试以及集成测试进行内存安全分析与检查。

