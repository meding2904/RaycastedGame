# Waits for the game window, focuses it and saves one screenshot of its client area.
#
# The capture uses PrintWindow with PW_RENDERFULLCONTENT rather than copying the screen: a screen
# grab also picks up any window that happens to be on top of the game, which is how an earlier
# gallery shot ended up half desktop. The client rect is what the renderer actually draws into, so
# the saved image is exactly the game frame, letterboxed by present().
param(
    [string]$Path = "D:\apps\RaycastedGame\docs\gallery\window.png",
    # Keys to send after focusing the window, e.g. "{ENTER}" or "w"
    [string]$Keys = "",
    [int]$WaitMs = 0
)

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Win {
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hWnd, IntPtr hdc, uint flags);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr hWnd, out RECT rect);
    [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr hWnd, ref POINT point);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr hWnd, uint msg, IntPtr wParam, IntPtr lParam);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
    [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
}
"@

$deadline = (Get-Date).AddSeconds(20)
$proc = $null
while ((Get-Date) -lt $deadline) {
    $proc = Get-Process wolf3d -ErrorAction SilentlyContinue | Where-Object { $_.MainWindowHandle -ne 0 } | Select-Object -First 1
    if ($proc) { break }
    Start-Sleep -Milliseconds 250
}

if (-not $proc) {
    Write-Output "no wolf3d window found"
    exit 1
}

$handle = $proc.MainWindowHandle
[Win]::ShowWindow($handle, 9) | Out-Null   # SW_RESTORE
[Win]::SetForegroundWindow($handle) | Out-Null
Start-Sleep -Milliseconds 900

# Posted key messages are not enough for winit, and SendKeys types into whatever has focus, so the
# window is activated first and the key is only posted as a fallback.
function Send-KeyToWindow($handle, $key) {
    $map = @{
        "ENTER" = 0x0D; "RETURN" = 0x0D; "UP" = 0x26; "DOWN" = 0x28
        "LEFT" = 0x25; "RIGHT" = 0x27; "SPACE" = 0x20; "ESC" = 0x1B; "TAB" = 0x09
    }
    $name = $key.Trim("{", "}").ToUpper()
    $vk = if ($map.ContainsKey($name)) { $map[$name] } else { [int][char]$name.ToUpper() }
    $scan = [int]$vk
    $down = [IntPtr](1 -bor ($scan -shl 16))
    $up = [IntPtr](0xC0000001 -bor ($scan -shl 16))
    [Win]::PostMessage($handle, 0x0100, [IntPtr]$vk, $down) | Out-Null   # WM_KEYDOWN
    Start-Sleep -Milliseconds 60
    [Win]::PostMessage($handle, 0x0101, [IntPtr]$vk, $up) | Out-Null     # WM_KEYUP
}

if ($Keys -ne "") {
    $shell = New-Object -ComObject wscript.shell
    $activated = $shell.AppActivate($proc.Id)
    Start-Sleep -Milliseconds 600
    if ($activated) {
        [System.Windows.Forms.SendKeys]::SendWait($Keys)
    } else {
        Send-KeyToWindow $handle $Keys
    }
    Start-Sleep -Milliseconds $WaitMs
}

$rect = New-Object Win+RECT
[Win]::GetClientRect($handle, [ref]$rect) | Out-Null
$width = $rect.Right - $rect.Left
$height = $rect.Bottom - $rect.Top
if ($width -le 0 -or $height -le 0) {
    Write-Output "window has no client area"
    exit 1
}

$bmp = New-Object System.Drawing.Bitmap $width, $height
$graphics = [System.Drawing.Graphics]::FromImage($bmp)
$hdc = $graphics.GetHdc()
$printed = [Win]::PrintWindow($handle, $hdc, 2)   # PW_RENDERFULLCONTENT
$graphics.ReleaseHdc($hdc)

if (-not $printed) {
    # Not every driver answers PrintWindow, so fall back to copying the screen region the client
    # area occupies. It can catch an overlapping window, but a shot beats no shot.
    $origin = New-Object Win+POINT
    $origin.X = 0
    $origin.Y = 0
    [Win]::ClientToScreen($handle, [ref]$origin) | Out-Null
    $graphics.CopyFromScreen($origin.X, $origin.Y, 0, 0, (New-Object System.Drawing.Size $width, $height))
}

$bmp.Save($Path, [System.Drawing.Imaging.ImageFormat]::Png)
$graphics.Dispose()
$bmp.Dispose()

Write-Output "saved $Path ($width x $height) from '$($proc.MainWindowTitle)'"
