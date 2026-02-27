#!/usr/bin/env python3
from __future__ import annotations

import math
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

SIZE = 512
AA = 1.25

Color = tuple[int, int, int]


@dataclass(frozen=True)
class Palette:
    season: str
    bg_top: Color
    bg_bottom: Color
    glow: Color
    panel: Color
    fg: Color
    accent: Color
    accent_2: Color


@dataclass(frozen=True)
class Style:
    key: str
    label: str
    card_radius: int
    grain_amp: float
    halo_radius: int
    halo_alpha: float
    motif_alpha: float
    shadow_alpha: float


PALETTES: tuple[Palette, ...] = (
    Palette(
        season="spring",
        bg_top=(209, 250, 229),
        bg_bottom=(45, 212, 191),
        glow=(254, 240, 138),
        panel=(240, 253, 244),
        fg=(17, 94, 89),
        accent=(236, 72, 153),
        accent_2=(74, 222, 128),
    ),
    Palette(
        season="summer",
        bg_top=(191, 219, 254),
        bg_bottom=(37, 99, 235),
        glow=(253, 224, 71),
        panel=(239, 246, 255),
        fg=(30, 64, 175),
        accent=(251, 146, 60),
        accent_2=(56, 189, 248),
    ),
    Palette(
        season="autumn",
        bg_top=(254, 215, 170),
        bg_bottom=(180, 83, 9),
        glow=(254, 243, 199),
        panel=(255, 247, 237),
        fg=(146, 64, 14),
        accent=(217, 119, 6),
        accent_2=(163, 230, 53),
    ),
    Palette(
        season="winter",
        bg_top=(219, 234, 254),
        bg_bottom=(67, 97, 238),
        glow=(224, 242, 254),
        panel=(238, 242, 255),
        fg=(30, 41, 59),
        accent=(125, 211, 252),
        accent_2=(147, 197, 253),
    ),
)


STYLES: tuple[Style, ...] = (
    Style(
        key="minimal-luxury",
        label="Minimal Luxury",
        card_radius=92,
        grain_amp=0.012,
        halo_radius=158,
        halo_alpha=0.28,
        motif_alpha=0.26,
        shadow_alpha=0.18,
    ),
    Style(
        key="cute-handdrawn",
        label="Cute Hand-Drawn",
        card_radius=118,
        grain_amp=0.042,
        halo_radius=168,
        halo_alpha=0.40,
        motif_alpha=0.40,
        shadow_alpha=0.14,
    ),
    Style(
        key="elegant-serif",
        label="Elegant Serif",
        card_radius=104,
        grain_amp=0.02,
        halo_radius=164,
        halo_alpha=0.34,
        motif_alpha=0.3,
        shadow_alpha=0.2,
    ),
)


class Canvas:
    def __init__(self, width: int, height: int) -> None:
        self.width = width
        self.height = height
        self.data = bytearray(width * height * 3)

    def set_pixel(self, x: int, y: int, color: Color) -> None:
        if x < 0 or y < 0 or x >= self.width or y >= self.height:
            return
        idx = (y * self.width + x) * 3
        self.data[idx] = color[0]
        self.data[idx + 1] = color[1]
        self.data[idx + 2] = color[2]

    def blend_pixel(self, x: int, y: int, color: Color, alpha: float) -> None:
        if alpha <= 0.0 or x < 0 or y < 0 or x >= self.width or y >= self.height:
            return
        if alpha >= 1.0:
            self.set_pixel(x, y, color)
            return
        idx = (y * self.width + x) * 3
        inv = 1.0 - alpha
        self.data[idx] = int(self.data[idx] * inv + color[0] * alpha + 0.5)
        self.data[idx + 1] = int(self.data[idx + 1] * inv + color[1] * alpha + 0.5)
        self.data[idx + 2] = int(self.data[idx + 2] * inv + color[2] * alpha + 0.5)


def clamp01(value: float) -> float:
    return 0.0 if value < 0.0 else 1.0 if value > 1.0 else value


def edge_alpha(distance: float, aa: float = AA) -> float:
    if distance <= -aa:
        return 1.0
    if distance >= aa:
        return 0.0
    return 0.5 - (distance / (2.0 * aa))


def mix_color(base: Color, top: Color, alpha: float) -> Color:
    alpha = clamp01(alpha)
    inv = 1.0 - alpha
    return (
        int(base[0] * inv + top[0] * alpha + 0.5),
        int(base[1] * inv + top[1] * alpha + 0.5),
        int(base[2] * inv + top[2] * alpha + 0.5),
    )


def lerp_color(a: Color, b: Color, t: float) -> Color:
    t = clamp01(t)
    return (
        int(a[0] + (b[0] - a[0]) * t + 0.5),
        int(a[1] + (b[1] - a[1]) * t + 0.5),
        int(a[2] + (b[2] - a[2]) * t + 0.5),
    )


def sdf_rounded_rect(px: float, py: float, cx: float, cy: float, hx: float, hy: float, radius: float) -> float:
    qx = abs(px - cx) - (hx - radius)
    qy = abs(py - cy) - (hy - radius)
    ox = qx if qx > 0.0 else 0.0
    oy = qy if qy > 0.0 else 0.0
    outside = math.hypot(ox, oy)
    inside = min(max(qx, qy), 0.0)
    return outside + inside - radius


def draw_circle(canvas: Canvas, cx: float, cy: float, radius: float, color: Color, alpha: float = 1.0) -> None:
    x0 = max(0, int(cx - radius - AA - 1))
    x1 = min(canvas.width - 1, int(cx + radius + AA + 1))
    y0 = max(0, int(cy - radius - AA - 1))
    y1 = min(canvas.height - 1, int(cy + radius + AA + 1))
    for y in range(y0, y1 + 1):
        py = y + 0.5
        for x in range(x0, x1 + 1):
            px = x + 0.5
            dist = math.hypot(px - cx, py - cy) - radius
            canvas.blend_pixel(x, y, color, alpha * edge_alpha(dist))


def draw_ellipse(
    canvas: Canvas,
    cx: float,
    cy: float,
    rx: float,
    ry: float,
    color: Color,
    alpha: float = 1.0,
    rotation_deg: float = 0.0,
) -> None:
    angle = math.radians(rotation_deg)
    cos_a = math.cos(angle)
    sin_a = math.sin(angle)

    rmax = max(rx, ry) + AA + 1
    x0 = max(0, int(cx - rmax))
    x1 = min(canvas.width - 1, int(cx + rmax))
    y0 = max(0, int(cy - rmax))
    y1 = min(canvas.height - 1, int(cy + rmax))

    for y in range(y0, y1 + 1):
        py = y + 0.5 - cy
        for x in range(x0, x1 + 1):
            px = x + 0.5 - cx
            tx = px * cos_a + py * sin_a
            ty = -px * sin_a + py * cos_a
            dist = (math.hypot(tx / rx, ty / ry) - 1.0) * min(rx, ry)
            canvas.blend_pixel(x, y, color, alpha * edge_alpha(dist))


def draw_rounded_rect(
    canvas: Canvas,
    x0: float,
    y0: float,
    x1: float,
    y1: float,
    radius: float,
    color: Color,
    alpha: float = 1.0,
) -> None:
    cx = (x0 + x1) / 2.0
    cy = (y0 + y1) / 2.0
    hx = (x1 - x0) / 2.0
    hy = (y1 - y0) / 2.0

    bx0 = max(0, int(x0 - AA - 1))
    bx1 = min(canvas.width - 1, int(x1 + AA + 1))
    by0 = max(0, int(y0 - AA - 1))
    by1 = min(canvas.height - 1, int(y1 + AA + 1))

    for y in range(by0, by1 + 1):
        py = y + 0.5
        for x in range(bx0, bx1 + 1):
            px = x + 0.5
            dist = sdf_rounded_rect(px, py, cx, cy, hx, hy, radius)
            canvas.blend_pixel(x, y, color, alpha * edge_alpha(dist))


def point_segment_distance(px: float, py: float, ax: float, ay: float, bx: float, by: float) -> float:
    abx = bx - ax
    aby = by - ay
    apx = px - ax
    apy = py - ay
    ab_len_sq = abx * abx + aby * aby
    if ab_len_sq == 0:
        return math.hypot(px - ax, py - ay)
    t = clamp01((apx * abx + apy * aby) / ab_len_sq)
    qx = ax + t * abx
    qy = ay + t * aby
    return math.hypot(px - qx, py - qy)


def draw_line(
    canvas: Canvas,
    ax: float,
    ay: float,
    bx: float,
    by: float,
    thickness: float,
    color: Color,
    alpha: float = 1.0,
) -> None:
    half = thickness / 2.0
    x0 = max(0, int(min(ax, bx) - half - AA - 1))
    x1 = min(canvas.width - 1, int(max(ax, bx) + half + AA + 1))
    y0 = max(0, int(min(ay, by) - half - AA - 1))
    y1 = min(canvas.height - 1, int(max(ay, by) + half + AA + 1))

    for y in range(y0, y1 + 1):
        py = y + 0.5
        for x in range(x0, x1 + 1):
            px = x + 0.5
            dist = point_segment_distance(px, py, ax, ay, bx, by) - half
            canvas.blend_pixel(x, y, color, alpha * edge_alpha(dist))


def point_in_polygon(px: float, py: float, points: list[tuple[float, float]]) -> bool:
    inside = False
    j = len(points) - 1
    for i, (xi, yi) in enumerate(points):
        xj, yj = points[j]
        intersects = ((yi > py) != (yj > py)) and (px < (xj - xi) * (py - yi) / ((yj - yi) + 1e-9) + xi)
        if intersects:
            inside = not inside
        j = i
    return inside


def draw_filled_polygon(canvas: Canvas, points: list[tuple[float, float]], color: Color, alpha: float = 1.0) -> None:
    x0 = max(0, int(min(p[0] for p in points) - 1))
    x1 = min(canvas.width - 1, int(max(p[0] for p in points) + 1))
    y0 = max(0, int(min(p[1] for p in points) - 1))
    y1 = min(canvas.height - 1, int(max(p[1] for p in points) + 1))
    offsets = ((0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75))

    for y in range(y0, y1 + 1):
        for x in range(x0, x1 + 1):
            coverage = 0
            for ox, oy in offsets:
                if point_in_polygon(x + ox, y + oy, points):
                    coverage += 1
            if coverage:
                canvas.blend_pixel(x, y, color, alpha * (coverage / 4.0))


def heart_outline_points(
    cx: float,
    cy: float,
    size: float,
    steps: int = 72,
    x_scale: float = 1.0,
    y_scale: float = 1.0,
) -> list[tuple[float, float]]:
    points: list[tuple[float, float]] = []
    for idx in range(steps):
        t = (idx / steps) * (2.0 * math.pi)
        hx = 16.0 * (math.sin(t) ** 3)
        hy = 13.0 * math.cos(t) - 5.0 * math.cos(2.0 * t) - 2.0 * math.cos(3.0 * t) - math.cos(4.0 * t)
        px = cx + (hx / 18.0) * size * x_scale
        py = cy - ((hy + 1.6) / 18.0) * size * y_scale
        points.append((px, py))
    return points


def draw_heart(
    canvas: Canvas,
    cx: float,
    cy: float,
    size: float,
    color: Color,
    alpha: float = 1.0,
    x_scale: float = 1.0,
    y_scale: float = 1.0,
) -> None:
    points = heart_outline_points(cx, cy, size, x_scale=x_scale, y_scale=y_scale)
    draw_filled_polygon(canvas, points, color, alpha)


def styled_colors(palette: Palette, style: Style) -> dict[str, Color]:
    if style.key == "minimal-luxury":
        cream = (247, 241, 229)
        deep = (52, 41, 33)
        gold = (208, 173, 107)
        return {
            "card": mix_color(palette.panel, cream, 0.66),
            "card_inner": mix_color(palette.panel, cream, 0.42),
            "ring": mix_color(palette.glow, cream, 0.44),
            "glyph": mix_color(palette.fg, deep, 0.58),
            "cut": mix_color(palette.panel, cream, 0.5),
            "accent": mix_color(palette.accent, gold, 0.62),
            "accent_2": mix_color(palette.accent_2, cream, 0.28),
            "border": mix_color(gold, palette.glow, 0.25),
        }

    if style.key == "cute-handdrawn":
        candy = (255, 228, 238)
        return {
            "card": mix_color(palette.panel, candy, 0.34),
            "card_inner": mix_color(palette.panel, candy, 0.2),
            "ring": mix_color(palette.glow, candy, 0.34),
            "glyph": mix_color(palette.fg, palette.accent_2, 0.18),
            "cut": mix_color(palette.panel, candy, 0.42),
            "accent": mix_color(palette.accent, candy, 0.24),
            "accent_2": mix_color(palette.accent_2, candy, 0.24),
            "border": mix_color(palette.panel, (255, 255, 255), 0.24),
        }

    violet = (214, 195, 231)
    plum = (60, 45, 82)
    return {
        "card": mix_color(palette.panel, (246, 243, 250), 0.48),
        "card_inner": mix_color(palette.panel, (252, 250, 255), 0.22),
        "ring": mix_color(palette.glow, (244, 237, 255), 0.32),
        "glyph": mix_color(palette.fg, plum, 0.42),
        "cut": mix_color(palette.panel, (246, 243, 250), 0.3),
        "accent": mix_color(palette.accent, violet, 0.44),
        "accent_2": mix_color(palette.accent_2, violet, 0.24),
        "border": mix_color(violet, palette.glow, 0.22),
    }


def draw_background(canvas: Canvas, palette: Palette, style: Style, variant_index: int) -> None:
    w = canvas.width
    h = canvas.height
    glow_x = w * (0.22 + 0.15 * (variant_index / 3.0))
    glow_y = h * (0.16 + 0.06 * (variant_index / 3.0))
    glow_radius = w * 0.74

    for y in range(h):
        for x in range(w):
            u = x / (w - 1)
            v = y / (h - 1)
            grain = style.grain_amp * math.sin((x * 0.043) + (y * 0.031) + variant_index * 1.7)
            t = clamp01((0.60 * u + 0.40 * v) + grain)
            base = lerp_color(palette.bg_top, palette.bg_bottom, t)

            dist = math.hypot(x - glow_x, y - glow_y)
            glow_alpha = clamp01(1.0 - dist / glow_radius) * (0.5 if style.key != "minimal-luxury" else 0.36)
            lit = mix_color(base, palette.glow, glow_alpha)

            vignette = clamp01((math.hypot(u - 0.5, v - 0.5) - 0.3) / 0.52)
            shaded = mix_color(lit, (14, 18, 32), vignette * 0.18)
            canvas.set_pixel(x, y, shaded)


def draw_card(canvas: Canvas, palette: Palette, style: Style) -> None:
    c = styled_colors(palette, style)

    if style.key == "minimal-luxury":
        draw_rounded_rect(canvas, 38, 38, 474, 474, style.card_radius + 10, c["border"], 0.44)
        draw_rounded_rect(canvas, 46, 46, 466, 466, style.card_radius, c["card"], 0.96)
        draw_rounded_rect(canvas, 60, 60, 452, 452, style.card_radius - 12, c["card_inner"], 0.28)
    elif style.key == "cute-handdrawn":
        draw_rounded_rect(canvas, 40, 42, 470, 472, style.card_radius, c["card"], 0.95)
        draw_rounded_rect(canvas, 56, 58, 454, 456, style.card_radius - 14, c["card_inner"], 0.22)
        draw_circle(canvas, 102, 408, 11, c["accent"], 0.12)
        draw_circle(canvas, 408, 102, 8, c["accent_2"], 0.16)
    else:
        draw_rounded_rect(canvas, 40, 40, 472, 472, style.card_radius, c["card"], 0.96)
        draw_rounded_rect(canvas, 52, 52, 460, 460, style.card_radius - 10, c["border"], 0.16)
        draw_rounded_rect(canvas, 62, 62, 450, 450, style.card_radius - 18, c["card_inner"], 0.2)


def draw_center_halo(canvas: Canvas, palette: Palette, style: Style) -> None:
    c = styled_colors(palette, style)
    draw_circle(canvas, 256, 252, style.halo_radius + 20, c["ring"], style.halo_alpha * 0.65)
    draw_circle(canvas, 256, 252, style.halo_radius - 10, c["ring"], style.halo_alpha)


def draw_season_motif(canvas: Canvas, palette: Palette, style: Style) -> None:
    c = styled_colors(palette, style)
    alpha = style.motif_alpha

    if palette.season == "spring":
        cx, cy = 392, 118
        for i in range(5):
            ang = i * (2.0 * math.pi / 5.0)
            draw_circle(canvas, cx + math.cos(ang) * 17, cy + math.sin(ang) * 17, 13, c["accent"], alpha + 0.2)
        draw_circle(canvas, cx, cy, 9, palette.glow, alpha + 0.3)
        draw_ellipse(canvas, 348, 126, 17, 9, c["accent_2"], alpha + 0.12, rotation_deg=-26)

    elif palette.season == "summer":
        cx, cy = 394, 116
        draw_circle(canvas, cx, cy, 27, palette.glow, alpha + 0.3)
        for i in range(8):
            ang = i * (2.0 * math.pi / 8.0)
            draw_line(
                canvas,
                cx + math.cos(ang) * 38,
                cy + math.sin(ang) * 38,
                cx + math.cos(ang) * 60,
                cy + math.sin(ang) * 60,
                7,
                c["accent"],
                alpha + 0.18,
            )

    elif palette.season == "autumn":
        draw_ellipse(canvas, 392, 118, 32, 18, c["accent"], alpha + 0.22, rotation_deg=-32)
        draw_ellipse(canvas, 404, 126, 22, 13, c["accent_2"], alpha + 0.12, rotation_deg=34)
        draw_line(canvas, 372, 136, 418, 98, 4, c["glyph"], alpha + 0.06)

    else:
        cx, cy = 394, 120
        for ang in (0, 60, 120):
            rad = math.radians(ang)
            draw_line(
                canvas,
                cx - math.cos(rad) * 42,
                cy - math.sin(rad) * 42,
                cx + math.cos(rad) * 42,
                cy + math.sin(rad) * 42,
                7,
                c["accent_2"],
                alpha + 0.2,
            )
        for ang in (0, 60, 120, 180, 240, 300):
            rad = math.radians(ang)
            draw_circle(canvas, cx + math.cos(rad) * 44, cy + math.sin(rad) * 44, 5, c["card"], alpha + 0.1)
        draw_circle(canvas, cx, cy, 8, c["card"], alpha + 0.2)


def draw_romance_particles(canvas: Canvas, palette: Palette, style: Style) -> None:
    c = styled_colors(palette, style)
    a = 0.22 if style.key == "minimal-luxury" else 0.34
    draw_circle(canvas, 156, 338, 8, c["accent"], a)
    draw_circle(canvas, 348, 348, 8, c["accent_2"], a)
    draw_heart(canvas, 360, 178, 12, c["accent"], a + 0.1, x_scale=1.05, y_scale=0.95)
    if style.key == "cute-handdrawn":
        draw_circle(canvas, 132, 306, 6, c["accent_2"], 0.26)
        draw_heart(canvas, 330, 332, 10, c["accent"], 0.2)


def draw_letter_b(canvas: Canvas, x: float, y: float, scale: float, color: Color, cut: Color, style: Style) -> None:
    stem_w = 22 * scale
    stem_h = 150 * scale
    loop_r = 38 * scale
    inner_r = 19 * scale

    if style.key == "minimal-luxury":
        stem_w = 18 * scale
        loop_r = 33 * scale
        inner_r = 16 * scale
    if style.key == "cute-handdrawn":
        stem_w = 24 * scale
        loop_r = 40 * scale

    draw_rounded_rect(canvas, x, y, x + stem_w, y + stem_h, 10 * scale, color, 0.96)
    draw_circle(canvas, x + stem_w + 30 * scale, y + 42 * scale, loop_r, color, 0.96)
    draw_circle(canvas, x + stem_w + 30 * scale, y + 104 * scale, loop_r, color, 0.96)
    draw_rounded_rect(canvas, x + stem_w - 3 * scale, y + 18 * scale, x + stem_w + 34 * scale, y + 130 * scale, 16 * scale, color, 0.96)
    draw_circle(canvas, x + stem_w + 36 * scale, y + 42 * scale, inner_r, cut, 0.98)
    draw_circle(canvas, x + stem_w + 36 * scale, y + 104 * scale, inner_r, cut, 0.98)

    if style.key == "elegant-serif":
        draw_rounded_rect(canvas, x - 4 * scale, y - 4 * scale, x + stem_w + 8 * scale, y + 8 * scale, 5 * scale, color, 0.96)
        draw_rounded_rect(canvas, x - 4 * scale, y + stem_h - 8 * scale, x + stem_w + 8 * scale, y + stem_h + 4 * scale, 5 * scale, color, 0.96)


def draw_letter_o(canvas: Canvas, cx: float, cy: float, radius: float, color: Color, cut: Color, style: Style) -> None:
    thickness = 0.54 if style.key == "minimal-luxury" else 0.52
    if style.key == "cute-handdrawn":
        thickness = 0.56
    draw_circle(canvas, cx, cy, radius, color, 0.96)
    draw_circle(canvas, cx, cy, radius * thickness, cut, 0.98)


def draw_letter_m(canvas: Canvas, x: float, y: float, scale: float, color: Color, style: Style) -> None:
    stem_w = 21 * scale
    stem_h = 128 * scale

    draw_rounded_rect(canvas, x, y, x + stem_w, y + stem_h, 9 * scale, color, 0.96)
    draw_rounded_rect(canvas, x + 70 * scale, y, x + 70 * scale + stem_w, y + stem_h, 9 * scale, color, 0.96)
    draw_line(canvas, x + stem_w - 2 * scale, y + 10 * scale, x + 35 * scale, y + 76 * scale, 14 * scale, color, 0.96)
    draw_line(canvas, x + 35 * scale, y + 76 * scale, x + 70 * scale, y + 10 * scale, 14 * scale, color, 0.96)

    if style.key == "elegant-serif":
        draw_rounded_rect(canvas, x - 3 * scale, y - 4 * scale, x + stem_w + 6 * scale, y + 8 * scale, 5 * scale, color, 0.96)
        draw_rounded_rect(canvas, x + 67 * scale, y - 4 * scale, x + 70 * scale + stem_w + 6 * scale, y + 8 * scale, 5 * scale, color, 0.96)


def draw_word_bom(canvas: Canvas, palette: Palette, style: Style) -> None:
    c = styled_colors(palette, style)
    glyph = c["glyph"]
    cut = c["cut"]

    if style.key == "minimal-luxury":
        b_scale = 0.78
        draw_letter_b(canvas, 112, 192, b_scale, glyph, cut, style)
        draw_letter_o(canvas, 258, 252, 40, glyph, cut, style)
        draw_letter_m(canvas, 304, 190, 0.9, glyph, style)
    elif style.key == "cute-handdrawn":
        draw_letter_b(canvas, 108, 190, 0.84, glyph, cut, style)
        draw_letter_o(canvas, 258, 252, 43, glyph, cut, style)
        draw_letter_m(canvas, 306, 190, 0.9, glyph, style)
        draw_circle(canvas, 345, 188, 7, c["accent"], 0.22)
    else:
        draw_letter_b(canvas, 116, 190, 0.8, glyph, cut, style)
        draw_letter_o(canvas, 258, 252, 41, glyph, cut, style)
        draw_letter_m(canvas, 306, 188, 0.92, glyph, style)


def draw_word_bom_korean(canvas: Canvas, palette: Palette, style: Style) -> None:
    c = styled_colors(palette, style)
    glyph = c["glyph"]
    cut = c["cut"]

    top_radius = 20 if style.key != "minimal-luxury" else 14
    draw_rounded_rect(canvas, 174, 158, 338, 252, top_radius, glyph, 0.96)  # ㅂ outer
    draw_rounded_rect(canvas, 202, 186, 310, 224, 12, cut, 0.98)
    draw_line(canvas, 202, 206, 310, 206, 12 if style.key == "minimal-luxury" else 14, glyph, 0.95)

    draw_line(canvas, 202, 272, 314, 272, 16, glyph, 0.96)  # ㅗ base
    draw_line(canvas, 258, 246, 258, 272, 12, glyph, 0.96)

    draw_rounded_rect(canvas, 194, 298, 318, 388, 16, glyph, 0.96)  # ㅁ outer
    draw_rounded_rect(canvas, 222, 326, 290, 360, 10, cut, 0.98)

    if style.key == "elegant-serif":
        draw_rounded_rect(canvas, 170, 154, 188, 168, 5, glyph, 0.96)
        draw_rounded_rect(canvas, 324, 154, 342, 168, 5, glyph, 0.96)


def draw_stage(canvas: Canvas, palette: Palette, style: Style, variant_index: int) -> None:
    draw_background(canvas, palette, style, variant_index)
    draw_card(canvas, palette, style)
    draw_center_halo(canvas, palette, style)


def draw_variant_love(canvas: Canvas, palette: Palette, style: Style) -> None:
    c = styled_colors(palette, style)
    draw_stage(canvas, palette, style, 0)

    base_heart = 74
    x_scale = 1.06
    y_scale = 0.94
    if style.key == "minimal-luxury":
        base_heart = 66
        x_scale = 1.0
        y_scale = 0.96
    elif style.key == "cute-handdrawn":
        base_heart = 80
        x_scale = 1.14
        y_scale = 0.9

    shadow = mix_color(c["glyph"], (18, 22, 40), 0.45)
    outer = c["accent"]
    inner = mix_color(c["accent"], c["cut"], 0.45)
    core = mix_color(inner, c["ring"], 0.4)

    draw_heart(canvas, 256, 260, base_heart + 5, shadow, style.shadow_alpha, x_scale=x_scale, y_scale=y_scale)
    draw_heart(canvas, 256, 248, base_heart, outer, 0.96, x_scale=x_scale, y_scale=y_scale)
    draw_heart(canvas, 256, 248, base_heart * 0.72, inner, 0.92, x_scale=x_scale, y_scale=y_scale)
    draw_heart(canvas, 256, 248, base_heart * 0.54, core, 0.88, x_scale=x_scale, y_scale=y_scale)
    draw_ellipse(canvas, 282, 220, 12, 7, c["cut"], 0.4, rotation_deg=-24)

    draw_romance_particles(canvas, palette, style)
    draw_season_motif(canvas, palette, style)


def draw_variant_b(canvas: Canvas, palette: Palette, style: Style) -> None:
    c = styled_colors(palette, style)
    draw_stage(canvas, palette, style, 1)

    glyph = c["glyph"]
    cut = c["cut"]
    shadow = mix_color(glyph, (10, 14, 26), 0.5)
    scale = 1.42 if style.key != "minimal-luxury" else 1.3

    draw_letter_b(canvas, 196, 170, scale, shadow, cut, style)
    draw_letter_b(canvas, 190, 162, scale, glyph, cut, style)
    draw_ellipse(canvas, 292, 196, 14, 8, c["cut"], 0.34, rotation_deg=-24)

    draw_romance_particles(canvas, palette, style)
    draw_season_motif(canvas, palette, style)


def draw_variant_bom(canvas: Canvas, palette: Palette, style: Style) -> None:
    draw_stage(canvas, palette, style, 2)
    draw_word_bom(canvas, palette, style)
    draw_romance_particles(canvas, palette, style)
    draw_season_motif(canvas, palette, style)


def draw_variant_bom_korean(canvas: Canvas, palette: Palette, style: Style) -> None:
    draw_stage(canvas, palette, style, 3)
    draw_word_bom_korean(canvas, palette, style)
    draw_romance_particles(canvas, palette, style)
    draw_season_motif(canvas, palette, style)


def write_ppm(path: Path, canvas: Canvas) -> None:
    header = f"P6\n{canvas.width} {canvas.height}\n255\n".encode("ascii")
    with path.open("wb") as handle:
        handle.write(header)
        handle.write(canvas.data)


def run_convert(command: list[str]) -> None:
    result = subprocess.run(command, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError(f"Command failed: {' '.join(command)}\n{result.stdout}\n{result.stderr}")


def write_outputs(ppm_path: Path, png_path: Path, icns_path: Path) -> None:
    run_convert(["sips", "-s", "format", "png", str(ppm_path), "--out", str(png_path)])
    run_convert(["sips", "-s", "format", "icns", str(png_path), "--out", str(icns_path)])


VariantFn = Callable[[Canvas, Palette, Style], None]


VARIANTS: tuple[tuple[str, str, VariantFn], ...] = (
    ("love-season", "love + season theme", draw_variant_love),
    ("b-season", "B + season theme", draw_variant_b),
    ("bom-season", "BOM + season theme", draw_variant_bom),
    ("bom-korean-season", "봄 + season theme", draw_variant_bom_korean),
)


OUTPUT_ROOT = Path(__file__).resolve().parents[1] / "public" / "favicons" / "prototypes"


def generate_icon(style: Style, palette: Palette, variant_key: str, draw_fn: VariantFn, variant_index: int) -> tuple[Path, Path]:
    base_dir = OUTPUT_ROOT / style.key / palette.season
    base_dir.mkdir(parents=True, exist_ok=True)
    stem = f"{palette.season}-{variant_key}"

    canvas = Canvas(SIZE, SIZE)
    draw_fn(canvas, palette, style)

    tmp_root = OUTPUT_ROOT / "_tmp"
    tmp_root.mkdir(parents=True, exist_ok=True)
    ppm_path = tmp_root / f"{style.key}-{stem}.ppm"
    png_path = base_dir / f"{stem}.png"
    icns_path = base_dir / f"{stem}.icns"
    write_ppm(ppm_path, canvas)
    write_outputs(ppm_path, png_path, icns_path)
    return png_path, icns_path


def clean_existing_outputs() -> None:
    if OUTPUT_ROOT.exists():
        for item in OUTPUT_ROOT.iterdir():
            if item.name == ".DS_Store":
                item.unlink()
            elif item.is_dir():
                shutil.rmtree(item)
            else:
                item.unlink()
    OUTPUT_ROOT.mkdir(parents=True, exist_ok=True)


def write_manifest(paths: list[tuple[str, str, str, str, Path, Path]]) -> None:
    lines = [
        "# Seasonal Bominal Icon Prototypes",
        "",
        "Generated by `web/scripts/generate-seasonal-favicons.py`.",
        "",
        "This set is fully personal/romantic (no train/reservation visuals).",
        "",
    ]

    for style in STYLES:
        lines.append(f"## {style.label}")
        lines.append("")
        lines.append("Variants:")
        lines.append("- `love-season`: love + season theme")
        lines.append("- `b-season`: B + season theme")
        lines.append("- `bom-season`: BOM + season theme")
        lines.append("- `bom-korean-season`: 봄 + season theme")
        lines.append("")

        for season in ("spring", "summer", "autumn", "winter"):
            lines.append(f"### {season.title()}")
            lines.append("")
            for _, _, season_key, variant_key, png_path, icns_path in [
                p for p in paths if p[0] == style.key and p[2] == season
            ]:
                lines.append(
                    f"- `{variant_key}`: `{png_path.relative_to(OUTPUT_ROOT.parent.parent)}` + "
                    f"`{icns_path.relative_to(OUTPUT_ROOT.parent.parent)}`"
                )
            lines.append("")

    (OUTPUT_ROOT / "README.md").write_text("\n".join(lines), encoding="utf-8")


def write_preview_html(paths: list[tuple[str, str, str, str, Path, Path]]) -> None:
    style_sections: list[str] = []

    for style in STYLES:
        season_blocks: list[str] = []
        for season in ("spring", "summer", "autumn", "winter"):
            cards: list[str] = []
            style_rows = [p for p in paths if p[0] == style.key and p[2] == season]
            for _, _, _, variant_key, png_path, _ in style_rows:
                rel = png_path.relative_to(OUTPUT_ROOT)
                cards.append(
                    f"""
                    <figure class=\"card\">
                      <img src=\"{rel.as_posix()}\" alt=\"{style.label} {season} {variant_key}\" />
                      <figcaption>{season} / {variant_key}</figcaption>
                    </figure>
                    """
                )
            season_blocks.append(f"<h3>{season.title()}</h3><div class='grid'>{''.join(cards)}</div>")

        style_sections.append(
            f"""
            <section class=\"style-section\">
              <h2>{style.label}</h2>
              {''.join(season_blocks)}
            </section>
            """
        )

    html = f"""<!doctype html>
<html lang=\"en\">
<head>
  <meta charset=\"utf-8\" />
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />
  <title>Bominal Seasonal Icon Prototypes</title>
  <style>
    :root {{
      --bg: #0f172a;
      --panel: #1e293b;
      --text: #e5e7eb;
      --muted: #9ca3af;
      --border: #334155;
    }}
    * {{ box-sizing: border-box; }}
    body {{
      margin: 0;
      background:
        radial-gradient(circle at 10% -10%, #1d4ed8 0%, transparent 36%),
        radial-gradient(circle at 90% 0%, #7c3aed 0%, transparent 30%),
        var(--bg);
      color: var(--text);
      font-family: "Avenir Next", "Trebuchet MS", "Segoe UI", sans-serif;
      line-height: 1.4;
      padding: 30px 16px 50px;
    }}
    main {{ max-width: 1240px; margin: 0 auto; }}
    h1 {{ margin: 0 0 8px; font-size: 2rem; }}
    p {{ margin: 0 0 28px; color: var(--muted); }}
    .style-section {{
      margin: 24px 0 36px;
      padding: 16px;
      border: 1px solid var(--border);
      border-radius: 18px;
      background: linear-gradient(180deg, rgba(255,255,255,0.06), rgba(255,255,255,0.01));
    }}
    h2 {{ margin: 0 0 12px; font-size: 1.3rem; }}
    h3 {{ margin: 16px 0 10px; font-size: 1rem; color: #cbd5e1; }}
    .grid {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(170px, 1fr));
      gap: 12px;
    }}
    .card {{
      margin: 0;
      padding: 10px;
      border: 1px solid var(--border);
      border-radius: 14px;
      background: rgba(15, 23, 42, 0.35);
      display: flex;
      flex-direction: column;
      gap: 8px;
      align-items: center;
    }}
    .card img {{
      width: 100%;
      max-width: 160px;
      border-radius: 16px;
      box-shadow: 0 8px 24px rgba(0,0,0,0.35);
    }}
    figcaption {{
      font-size: 0.85rem;
      color: var(--muted);
      text-align: center;
      text-transform: lowercase;
    }}
  </style>
</head>
<body>
  <main>
    <h1>Bominal Personal Seasonal Icon Prototypes</h1>
    <p>3 style families × 4 seasons × 4 variants = 48 icons. Includes <code>.png</code> and <code>.icns</code>.</p>
    {''.join(style_sections)}
  </main>
</body>
</html>
"""

    (OUTPUT_ROOT / "preview.html").write_text(html, encoding="utf-8")


def main() -> None:
    clean_existing_outputs()

    all_paths: list[tuple[str, str, str, str, Path, Path]] = []
    for style in STYLES:
        for season in PALETTES:
            for variant_index, (variant_key, variant_label, draw_fn) in enumerate(VARIANTS):
                png_path, icns_path = generate_icon(style, season, variant_key, draw_fn, variant_index)
                all_paths.append((style.key, style.label, season.season, variant_key, png_path, icns_path))

    write_manifest(all_paths)
    write_preview_html(all_paths)

    tmp_root = OUTPUT_ROOT / "_tmp"
    if tmp_root.exists():
        shutil.rmtree(tmp_root)

    print("Generated personal seasonal icon prototypes:")
    for style_key, _, season, variant_key, png_path, icns_path in all_paths:
        print(f"- {style_key:<15} {season:<6} {variant_key:<18} {png_path} {icns_path}")


if __name__ == "__main__":
    main()
