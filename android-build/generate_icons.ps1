Add-Type -AssemblyName System.Drawing

$sizes = @{
    'mipmap-mdpi' = 48
    'mipmap-hdpi' = 72
    'mipmap-xhdpi' = 96
    'mipmap-xxhdpi' = 144
    'mipmap-xxxhdpi' = 192
}

foreach ($folder in $sizes.Keys) {
    $s = $sizes[$folder]
    $bmp = New-Object System.Drawing.Bitmap $s, $s
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias

    # Background rounded rect / circle
    $bgBrush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(255, 21, 18, 23))
    $g.FillEllipse($bgBrush, 2, 2, $s - 4, $s - 4)

    # Purple border
    $colPurple = [System.Drawing.Color]::FromArgb(255, 110, 76, 158)
    $wBorder = [float]($s * 0.05)
    $borderPen = New-Object System.Drawing.Pen ($colPurple, $wBorder)
    $g.DrawEllipse($borderPen, 2, 2, $s - 4, $s - 4)

    # Pink Gate symbol (AND/XOR shape)
    $colPink = [System.Drawing.Color]::FromArgb(255, 255, 79, 163)
    $wPink = [float]($s * 0.08)
    $pinkPen = New-Object System.Drawing.Pen ($colPink, $wPink)
    $pinkBrush = New-Object System.Drawing.SolidBrush $colPink

    $gx = $s * 0.28
    $gy = $s * 0.28
    $gw = $s * 0.44
    $gh = $s * 0.44

    # Gate body
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $path.AddLine([float]$gx, [float]$gy, [float]($gx + $gw * 0.5), [float]$gy)
    $path.AddArc([float]($gx + $gw * 0.1), [float]$gy, [float]($gw * 0.8), [float]$gh, -90, 180)
    $path.AddLine([float]($gx + $gw * 0.5), [float]($gy + $gh), [float]$gx, [float]($gy + $gh))
    $path.CloseFigure()
    $g.DrawPath($pinkPen, $path)

    # Input pins
    $g.DrawLine($pinkPen, [float]($gx - $gw * 0.35), [float]($gy + $gh * 0.3), [float]$gx, [float]($gy + $gh * 0.3))
    $g.DrawLine($pinkPen, [float]($gx - $gw * 0.35), [float]($gy + $gh * 0.7), [float]$gx, [float]($gy + $gh * 0.7))

    # Output pin & signal dot
    $g.DrawLine($pinkPen, [float]($gx + $gw * 0.9), [float]($gy + $gh * 0.5), [float]($gx + $gw * 1.3), [float]($gy + $gh * 0.5))
    $dotR = $s * 0.06
    $g.FillEllipse($pinkBrush, [float]($gx + $gw * 1.3 - $dotR), [float]($gy + $gh * 0.5 - $dotR), [float]($dotR * 2), [float]($dotR * 2))

    $outPath = "d:\DLCD\android-build\res\$folder\ic_launcher.png"
    $bmp.Save($outPath, [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose()
    $bmp.Dispose()
}

Write-Host "Icons generated successfully"
