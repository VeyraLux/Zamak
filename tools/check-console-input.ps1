# 在真实的 Windows 控制台里验证中文键盘输入：新开一个控制台运行测试程序，用 WriteConsoleInputW
# 注入「你好 abc」「ok」和 Ctrl+Z，检查 io.read_line 读到的是 UTF-8 而不是本地代码页（GBK）字节。
# 用法（pwsh，仓库根目录）：pwsh -File tools/check-console-input.ps1 [-ZamPath zam/target/debug/zam.exe]
# 运行时会短暂弹出一个控制台窗口。
param([string]$ZamPath = 'zam/target/debug/zam.exe')
$ErrorActionPreference = 'Stop'
if (-not $IsWindows) { throw 'console input check only runs on Windows' }

Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class ZamConsoleInput {
    [DllImport("kernel32.dll", SetLastError = true)] static extern bool FreeConsole();
    [DllImport("kernel32.dll", SetLastError = true)] static extern bool AttachConsole(uint pid);
    [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Unicode)]
    static extern IntPtr CreateFileW(string name, uint access, uint share, IntPtr sec, uint disp, uint flags, IntPtr tmpl);
    [DllImport("kernel32.dll", SetLastError = true)] static extern bool CloseHandle(IntPtr h);
    [StructLayout(LayoutKind.Explicit, CharSet = CharSet.Unicode)]
    struct KeyEvent {
        [FieldOffset(0)] public int Down;
        [FieldOffset(4)] public ushort Repeat;
        [FieldOffset(6)] public ushort VirtualKey;
        [FieldOffset(8)] public ushort ScanCode;
        [FieldOffset(10)] public char Unicode;
        [FieldOffset(12)] public uint ControlKeys;
    }
    [StructLayout(LayoutKind.Explicit)]
    struct InputRecord {
        [FieldOffset(0)] public ushort EventType;
        [FieldOffset(4)] public KeyEvent Key;
    }
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern bool WriteConsoleInputW(IntPtr h, InputRecord[] buffer, uint length, out uint written);
    public static void Type(uint pid, string text) {
        FreeConsole();
        if (!AttachConsole(pid)) throw new Exception("AttachConsole failed: " + Marshal.GetLastWin32Error());
        IntPtr input = CreateFileW("CONIN$", 0xC0000000, 3, IntPtr.Zero, 3, 0, IntPtr.Zero);
        var records = new List<InputRecord>();
        foreach (char c in text) {
            for (int down = 1; down >= 0; down--) {
                var record = new InputRecord();
                record.EventType = 1;
                record.Key.Down = down;
                record.Key.Repeat = 1;
                record.Key.VirtualKey = (ushort)(c == '\r' ? 0x0D : 0);
                record.Key.Unicode = c;
                records.Add(record);
            }
        }
        uint written;
        bool ok = WriteConsoleInputW(input, records.ToArray(), (uint)records.Count, out written);
        CloseHandle(input);
        FreeConsole();
        if (!ok) throw new Exception("WriteConsoleInputW failed: " + Marshal.GetLastWin32Error());
    }
}
'@

$work = Join-Path ([IO.Path]::GetTempPath()) "zam-console-input-$PID"
New-Item -ItemType Directory -Force $work | Out-Null
try {
    $source = Join-Path $work 'main.zm'
    Set-Content -Path $source -Encoding utf8NoBOM -Value @'
use std/fs
use std/io
use std/string

fn main() -> !string {
    let mut line = ""
    let mut out = ""
    while io.read_line(&mut line)? {
        out += "{string.byte_len(&line)} {line}\n"
    }
    fs.write("out.txt", &out)?
}
'@
    $env:ZAMAK_CACHE_DIR = Join-Path $work 'cache'
    $build = & $ZamPath build $source 2>&1
    $exe = ($build | Where-Object { "$_" -like 'OUTPUT *' } | Select-Object -First 1) -replace '^OUTPUT ', ''
    if (-not $exe) { throw "build failed:`n$build" }
    $process = Start-Process -FilePath $exe -WorkingDirectory $work -PassThru
    Start-Sleep -Milliseconds 800
    [ZamConsoleInput]::Type([uint32]$process.Id, "你好 abc`rok`r" + [char]0x1A + "`r")
    if (-not $process.WaitForExit(10000)) { $process.Kill(); throw 'program did not finish' }
    $actual = [IO.File]::ReadAllText((Join-Path $work 'out.txt'), [Text.UTF8Encoding]::new($false))
    $expected = "10 你好 abc`n2 ok`n"
    if ($actual -ne $expected) { throw "unexpected input bytes: $([BitConverter]::ToString([Text.Encoding]::UTF8.GetBytes($actual)))" }
    'PASS console input is read as UTF-8'
} finally {
    Remove-Item Env:ZAMAK_CACHE_DIR -ErrorAction SilentlyContinue
    Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue
}
