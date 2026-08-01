"""Generate LiteMark app icon master PNG (1024x1024) and run-ready source."""

from __future__ import annotations

import os
from PIL import Image, ImageDraw, ImageFilter, ImageChops

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "assets", "brand")
os.makedirs(OUT, exist_ok=True)

SIZE = 1024


def thick_line(draw: ImageDraw.ImageDraw, a, b, fill, w: int = 18) -> None:
    draw.line([a, b], fill=fill, width=w)
    r = w // 2
    draw.ellipse((a[0] - r, a[1] - r, a[0] + r, a[1] + r), fill=fill)
    draw.ellipse((b[0] - r, b[1] - r, b[0] + r, b[1] + r), fill=fill)


def make_icon(full_bleed: bool = True) -> Image.Image:
    """full_bleed: fill entire canvas (for tauri icon). Else floating tile with glow."""
    img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    margin = 0 if full_bleed else 72
    radius = 220 if full_bleed else 200

    # Soft glow only for marketing/floating variant
    if not full_bleed:
        glow = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
        gdraw = ImageDraw.Draw(glow)
        gdraw.rounded_rectangle(
            (48, 48, SIZE - 48, SIZE - 48), radius=220, fill=(37, 99, 235, 55)
        )
        glow = glow.filter(ImageFilter.GaussianBlur(28))
        img = Image.alpha_composite(img, glow)

    # Gradient background
    tile = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    td = ImageDraw.Draw(tile)
    for y in range(SIZE):
        t = y / (SIZE - 1)
        r = int(15 + (37 - 15) * t)
        g = int(40 + (99 - 40) * t)
        b = int(90 + (235 - 90) * t)
        td.line([(0, y), (SIZE, y)], fill=(r, g, b, 255))

    mask = Image.new("L", (SIZE, SIZE), 0)
    md = ImageDraw.Draw(mask)
    md.rounded_rectangle(
        (margin, margin, SIZE - margin, SIZE - margin), radius=radius, fill=255
    )
    tile.putalpha(mask)
    img = Image.alpha_composite(img, tile)

    # Top highlight
    highlight = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    hd = ImageDraw.Draw(highlight)
    hd.rounded_rectangle(
        (margin + 8, margin + 8, SIZE - margin - 8, margin + SIZE // 2),
        radius=max(radius - 20, 40),
        fill=(255, 255, 255, 30),
    )
    highlight = highlight.filter(ImageFilter.GaussianBlur(14))
    hm = Image.new("L", (SIZE, SIZE), 0)
    ImageDraw.Draw(hm).rounded_rectangle(
        (margin, margin, SIZE - margin, SIZE - margin), radius=radius, fill=255
    )
    ha = ImageChops.multiply(highlight.split()[-1], hm)
    highlight.putalpha(ha)
    img = Image.alpha_composite(img, highlight)
    draw = ImageDraw.Draw(img)

    # Document proportions relative to usable tile
    usable = SIZE - 2 * margin
    doc_l = margin + int(usable * 0.18)
    doc_t = margin + int(usable * 0.14)
    doc_r = margin + int(usable * 0.82)
    doc_b = margin + int(usable * 0.86)
    doc_radius = max(int(usable * 0.05), 24)

    # Document shadow
    sh = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    sd = ImageDraw.Draw(sh)
    off = max(int(usable * 0.02), 10)
    sd.rounded_rectangle(
        (doc_l + off, doc_t + off + 8, doc_r + off, doc_b + off + 8),
        radius=doc_radius,
        fill=(0, 0, 0, 70),
    )
    sh = sh.filter(ImageFilter.GaussianBlur(max(int(usable * 0.025), 12)))
    img = Image.alpha_composite(img, sh)
    draw = ImageDraw.Draw(img)

    # Paper
    draw.rounded_rectangle(
        (doc_l, doc_t, doc_r, doc_b), radius=doc_radius, fill=(255, 255, 255, 252)
    )

    # Folded corner
    fold = max(int((doc_r - doc_l) * 0.18), 64)
    draw.polygon(
        [
            (doc_r - fold, doc_t),
            (doc_r, doc_t + fold),
            (doc_r - fold, doc_t + fold),
        ],
        fill=(226, 232, 240, 255),
    )
    draw.polygon(
        [
            (doc_r - fold, doc_t),
            (doc_r, doc_t + fold),
            (doc_r - fold + 3, doc_t + fold),
        ],
        fill=(241, 245, 249, 255),
    )
    draw.line(
        [(doc_r - fold, doc_t), (doc_r, doc_t + fold)],
        fill=(148, 163, 184, 255),
        width=max(int(usable * 0.004), 2),
    )

    # Content lines
    pad = int((doc_r - doc_l) * 0.12)
    line_x0 = doc_l + pad
    line_x1 = doc_r - pad - fold // 3
    y = doc_t + int((doc_b - doc_t) * 0.18)
    h = max(int((doc_b - doc_t) * 0.045), 18)
    # Title bar
    draw.rounded_rectangle(
        (line_x0, y, line_x0 + int((line_x1 - line_x0) * 0.55), y + h + 8),
        radius=h // 2,
        fill=(37, 99, 235, 255),
    )
    y += h + int((doc_b - doc_t) * 0.08)
    body_h = max(int((doc_b - doc_t) * 0.028), 12)
    for i, w in enumerate([1.0, 0.92, 0.85, 0.78, 0.52]):
        x1 = line_x0 + int((line_x1 - line_x0) * w)
        color = (148, 163, 184, 230) if i else (100, 116, 139, 255)
        draw.rounded_rectangle(
            (line_x0, y, x1, y + body_h), radius=body_h // 2, fill=color
        )
        y += body_h + int((doc_b - doc_t) * 0.045)

    # Markdown "#" badge
    hash_cx = doc_l + int((doc_r - doc_l) * 0.28)
    hash_cy = doc_b - int((doc_b - doc_t) * 0.18)
    badge_r = int((doc_r - doc_l) * 0.12)
    draw.ellipse(
        (
            hash_cx - badge_r,
            hash_cy - badge_r,
            hash_cx + badge_r,
            hash_cy + badge_r,
        ),
        fill=(219, 234, 254, 255),
    )
    blue = (37, 99, 235, 255)
    stroke = max(int(badge_r * 0.22), 10)
    thick_line(
        draw,
        (hash_cx - int(badge_r * 0.32), hash_cy - int(badge_r * 0.55)),
        (hash_cx - int(badge_r * 0.14), hash_cy + int(badge_r * 0.55)),
        blue,
        stroke,
    )
    thick_line(
        draw,
        (hash_cx + int(badge_r * 0.14), hash_cy - int(badge_r * 0.55)),
        (hash_cx + int(badge_r * 0.32), hash_cy + int(badge_r * 0.55)),
        blue,
        stroke,
    )
    thick_line(
        draw,
        (hash_cx - int(badge_r * 0.55), hash_cy - int(badge_r * 0.18)),
        (hash_cx + int(badge_r * 0.55), hash_cy - int(badge_r * 0.08)),
        blue,
        stroke,
    )
    thick_line(
        draw,
        (hash_cx - int(badge_r * 0.55), hash_cy + int(badge_r * 0.12)),
        (hash_cx + int(badge_r * 0.55), hash_cy + int(badge_r * 0.22)),
        blue,
        stroke,
    )

    # Pen accent (editing)
    pen_x = doc_r - int((doc_r - doc_l) * 0.18)
    pen_y = doc_b - int((doc_b - doc_t) * 0.2)
    half = max(int((doc_r - doc_l) * 0.025), 10)
    draw.polygon(
        [
            (pen_x - half, pen_y - int((doc_b - doc_t) * 0.14)),
            (pen_x + half, pen_y - int((doc_b - doc_t) * 0.14)),
            (pen_x + half - 2, pen_y + int((doc_b - doc_t) * 0.04)),
            (pen_x - half + 2, pen_y + int((doc_b - doc_t) * 0.04)),
        ],
        fill=(15, 23, 42, 255),
    )
    draw.polygon(
        [
            (pen_x - half + 2, pen_y + int((doc_b - doc_t) * 0.04)),
            (pen_x + half - 2, pen_y + int((doc_b - doc_t) * 0.04)),
            (pen_x, pen_y + int((doc_b - doc_t) * 0.11)),
        ],
        fill=(56, 189, 248, 255),
    )
    draw.rectangle(
        (
            pen_x - half,
            pen_y - int((doc_b - doc_t) * 0.04),
            pen_x + half,
            pen_y - int((doc_b - doc_t) * 0.015),
        ),
        fill=(251, 191, 36, 255),
    )

    return img


def main() -> None:
    # Tauri wants squared PNG, preferably full-bleed with transparency only on corners.
    source = make_icon(full_bleed=True)
    source_path = os.path.join(OUT, "litemark-icon-source.png")
    source.save(source_path, "PNG")
    print("wrote", source_path)

    # Marketing / floating tile with soft glow
    promo = make_icon(full_bleed=False)
    promo_path = os.path.join(OUT, "litemark-icon-1024.png")
    promo.save(promo_path, "PNG")
    print("wrote", promo_path)

    # Preview 256
    source.resize((256, 256), Image.Resampling.LANCZOS).save(
        os.path.join(OUT, "litemark-icon-256.png"), "PNG"
    )
    print("done")


if __name__ == "__main__":
    main()
