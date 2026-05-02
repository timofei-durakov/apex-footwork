param(
    [string]$OutDir = "assets/alert_stickers"
)

Add-Type -AssemblyName System.Drawing

$ErrorActionPreference = "Stop"
$fullOut = Join-Path (Get-Location) $OutDir
New-Item -ItemType Directory -Force -Path $fullOut | Out-Null

function New-Canvas {
    $bitmap = New-Object System.Drawing.Bitmap 512, 512, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $graphics = [System.Drawing.Graphics]::FromImage($bitmap)
    $graphics.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
    $graphics.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
    $graphics.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
    $graphics.Clear([System.Drawing.Color]::Transparent)
    return @($bitmap, $graphics)
}

function New-Pen($color, $width) {
    $pen = New-Object System.Drawing.Pen $color, $width
    $pen.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
    $pen.EndCap = [System.Drawing.Drawing2D.LineCap]::Round
    $pen.LineJoin = [System.Drawing.Drawing2D.LineJoin]::Round
    return $pen
}

function New-Brush($color) {
    return New-Object System.Drawing.SolidBrush $color
}

function Color($hex, [int]$alpha = 255) {
    $hex = $hex.TrimStart("#")
    $r = [Convert]::ToInt32($hex.Substring(0, 2), 16)
    $g = [Convert]::ToInt32($hex.Substring(2, 2), 16)
    $b = [Convert]::ToInt32($hex.Substring(4, 2), 16)
    return [System.Drawing.Color]::FromArgb($alpha, $r, $g, $b)
}

function RoundedRect($x, $y, $w, $h, $r) {
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $d = $r * 2
    $path.AddArc($x, $y, $d, $d, 180, 90)
    $path.AddArc($x + $w - $d, $y, $d, $d, 270, 90)
    $path.AddArc($x + $w - $d, $y + $h - $d, $d, $d, 0, 90)
    $path.AddArc($x, $y + $h - $d, $d, $d, 90, 90)
    $path.CloseFigure()
    return $path
}

function Draw-StickerBase($g, $color) {
    $shadow = New-Brush (Color "000000" 72)
    $paper = New-Brush (Color "F7FBFF" 236)
    $rim = New-Pen $color 24
    $inner = New-Pen (Color "11151C" 110) 8
    $badge = RoundedRect 78 78 356 356 86
    $g.TranslateTransform(10, 14)
    $g.FillPath($shadow, $badge)
    $g.ResetTransform()
    $g.FillPath($paper, $badge)
    $g.DrawPath($rim, $badge)
    $g.DrawPath($inner, $badge)
    $shadow.Dispose()
    $paper.Dispose()
    $rim.Dispose()
    $inner.Dispose()
    $badge.Dispose()
}

function Save-Sticker($bitmap, $graphics, $name) {
    $graphics.Dispose()
    $path = Join-Path $fullOut $name
    $bitmap.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
    $bitmap.Dispose()
}

function Draw-Pedal($g, $x, $y, $angle, $fill) {
    $state = $g.Save()
    $g.TranslateTransform($x, $y)
    $g.RotateTransform($angle)
    $body = RoundedRect -48 -108 96 216 36
    $g.FillPath((New-Brush $fill), $body)
    $g.DrawPath((New-Pen (Color "11151C" 220) 10), $body)
    $slot = New-Pen (Color "11151C" 165) 9
    $g.DrawLine($slot, -22, -62, 22, -62)
    $g.DrawLine($slot, -24, -20, 24, -20)
    $g.DrawLine($slot, -20, 26, 20, 26)
    $g.DrawLine($slot, -15, 70, 15, 70)
    $slot.Dispose()
    $body.Dispose()
    $g.Restore($state)
}

function Draw-Wheel($g, $cx, $cy, $r, $color) {
    $outer = New-Pen $color 25
    $inner = New-Pen (Color "11151C" 185) 10
    $spoke = New-Pen (Color "11151C" 210) 13
    $g.DrawEllipse($outer, $cx - $r, $cy - $r, $r * 2, $r * 2)
    $g.DrawEllipse($inner, $cx - $r + 24, $cy - $r + 24, ($r - 24) * 2, ($r - 24) * 2)
    $g.DrawLine($spoke, $cx, $cy, $cx, $cy + $r - 30)
    $g.DrawLine($spoke, $cx, $cy, $cx - $r + 42, $cy - 16)
    $g.DrawLine($spoke, $cx, $cy, $cx + $r - 42, $cy - 16)
    $outer.Dispose()
    $inner.Dispose()
    $spoke.Dispose()
}

function PedalOverlap {
    $pair = New-Canvas
    $bitmap = $pair[0]
    $g = $pair[1]
    $red = Color "FF5260"
    Draw-StickerBase $g $red
    Draw-Pedal $g 207 266 -24 (Color "00D2B7")
    Draw-Pedal $g 305 266 24 (Color "FF5260")
    $xPen = New-Pen (Color "11151C" 235) 46
    $accent = New-Pen (Color "FF5260") 28
    $g.DrawLine($xPen, 170, 172, 342, 340)
    $g.DrawLine($xPen, 342, 172, 170, 340)
    $g.DrawLine($accent, 170, 172, 342, 340)
    $g.DrawLine($accent, 342, 172, 170, 340)
    $xPen.Dispose()
    $accent.Dispose()
    Save-Sticker $bitmap $g "pedal_overlap.png"
}

function Coasting {
    $pair = New-Canvas
    $bitmap = $pair[0]
    $g = $pair[1]
    $gold = Color "FFC65C"
    Draw-StickerBase $g $gold
    $ring = New-Pen (Color "11151C" 210) 30
    $goldPen = New-Pen $gold 20
    $g.DrawEllipse($ring, 164, 150, 184, 184)
    $g.DrawEllipse($goldPen, 180, 166, 152, 152)
    $foot = RoundedRect 216 286 80 132 34
    $g.FillPath((New-Brush (Color "FFC65C" 205)), $foot)
    $g.DrawPath((New-Pen (Color "11151C" 190) 9), $foot)
    $toeBrush = New-Brush (Color "11151C" 185)
    foreach ($toe in @(@(204, 272, 22), @(232, 254, 24), @(264, 252, 22), @(292, 266, 18))) {
        $g.FillEllipse($toeBrush, $toe[0], $toe[1], $toe[2], $toe[2])
    }
    $ring.Dispose()
    $goldPen.Dispose()
    $toeBrush.Dispose()
    $foot.Dispose()
    Save-Sticker $bitmap $g "coasting.png"
}

function ThrottleWithLock {
    $pair = New-Canvas
    $bitmap = $pair[0]
    $g = $pair[1]
    $red = Color "FF5260"
    Draw-StickerBase $g $red
    Draw-Wheel $g 234 244 88 (Color "FFC65C")
    $wedge = New-Object System.Drawing.Drawing2D.GraphicsPath
    $wedge.AddPolygon(@(
        [System.Drawing.Point]::new(296, 305),
        [System.Drawing.Point]::new(404, 248),
        [System.Drawing.Point]::new(366, 372)
    ))
    $g.FillPath((New-Brush (Color "00D2B7")), $wedge)
    $g.DrawPath((New-Pen (Color "11151C" 220) 11), $wedge)
    $lockPen = New-Pen (Color "FF5260") 24
    $g.DrawArc($lockPen, 130, 120, 248, 248, 205, 185)
    $g.DrawLine($lockPen, 145, 196, 120, 158)
    $g.DrawLine($lockPen, 145, 196, 186, 185)
    $lockPen.Dispose()
    $wedge.Dispose()
    Save-Sticker $bitmap $g "throttle_with_lock.png"
}

function BrakeReleaseSnap {
    $pair = New-Canvas
    $bitmap = $pair[0]
    $g = $pair[1]
    $red = Color "FF5260"
    Draw-StickerBase $g $red
    Draw-Pedal $g 222 278 -8 (Color "FF5260")
    $bolt = New-Object System.Drawing.Drawing2D.GraphicsPath
    $bolt.AddPolygon(@(
        [System.Drawing.Point]::new(294, 112),
        [System.Drawing.Point]::new(238, 250),
        [System.Drawing.Point]::new(298, 240),
        [System.Drawing.Point]::new(252, 398),
        [System.Drawing.Point]::new(390, 202),
        [System.Drawing.Point]::new(318, 218)
    ))
    $g.FillPath((New-Brush (Color "FFC65C")), $bolt)
    $g.DrawPath((New-Pen (Color "11151C" 225) 12), $bolt)
    $g.DrawLine((New-Pen (Color "FF5260") 20), 158, 358, 120, 398)
    $g.DrawLine((New-Pen (Color "FF5260") 20), 178, 386, 150, 430)
    $bolt.Dispose()
    Save-Sticker $bitmap $g "brake_release_snap.png"
}

function SteeringSaw {
    $pair = New-Canvas
    $bitmap = $pair[0]
    $g = $pair[1]
    $gold = Color "FFC65C"
    Draw-StickerBase $g $gold
    Draw-Wheel $g 256 236 96 (Color "00D2B7")
    $zig = New-Pen (Color "FF5260") 25
    $shadow = New-Pen (Color "11151C" 210) 43
    $points = @(
        [System.Drawing.Point]::new(118, 358),
        [System.Drawing.Point]::new(168, 312),
        [System.Drawing.Point]::new(220, 382),
        [System.Drawing.Point]::new(270, 312),
        [System.Drawing.Point]::new(324, 382),
        [System.Drawing.Point]::new(394, 326)
    )
    $g.DrawLines($shadow, $points)
    $g.DrawLines($zig, $points)
    $zig.Dispose()
    $shadow.Dispose()
    Save-Sticker $bitmap $g "steering_saw.png"
}

function SteeringSaturated {
    $pair = New-Canvas
    $bitmap = $pair[0]
    $g = $pair[1]
    $red = Color "FF5260"
    Draw-StickerBase $g $red
    Draw-Wheel $g 256 256 92 (Color "FFC65C")
    $stopPen = New-Pen (Color "11151C" 230) 30
    $redPen = New-Pen (Color "FF5260") 18
    $g.DrawLine($stopPen, 120, 132, 120, 380)
    $g.DrawLine($stopPen, 392, 132, 392, 380)
    $g.DrawLine($redPen, 120, 132, 120, 380)
    $g.DrawLine($redPen, 392, 132, 392, 380)
    $g.DrawLine((New-Pen (Color "00D2B7") 22), 318, 174, 370, 126)
    $g.DrawLine((New-Pen (Color "00D2B7") 22), 322, 338, 374, 390)
    $stopPen.Dispose()
    $redPen.Dispose()
    Save-Sticker $bitmap $g "steering_saturated.png"
}

PedalOverlap
Coasting
ThrottleWithLock
BrakeReleaseSnap
SteeringSaw
SteeringSaturated
