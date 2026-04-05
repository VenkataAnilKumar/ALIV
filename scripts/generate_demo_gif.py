"""
ALIV Demo GIF Generator
Produces docs/assets/demo.gif — a 1200x720 animated walkthrough of the
Query, Extraction, and Actions Preview screens using the demo corpus data.

Usage:
    python scripts/generate_demo_gif.py

Output: docs/assets/demo.gif
"""

import os
import sys
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

# ── Output ──────────────────────────────────────────────────────────────────
ROOT = Path(__file__).parent.parent
OUT = ROOT / "docs" / "assets" / "demo.gif"
OUT.parent.mkdir(parents=True, exist_ok=True)

W, H = 1200, 720
FPS = 12  # frames per second
SIDEBAR_W = 268

# ── Palette (matches index.css / tailwind.config.ts) ──────────────────────
APP_BG       = (8,   17,  26)
PANEL_BG     = (14,  25,  38)
PANEL_BG2    = (19,  35,  51)
PANEL_BORDER = (45,  80,  105, 60)
SIDEBAR_BG   = (8,   17,  26)
ACCENT_CYAN  = (83,  211, 209)
ACCENT_WARM  = (255, 157, 92)
ACCENT_INDIGO= (99,  102, 241)
TEXT_STRONG  = (236, 247, 255)
TEXT_SOFT    = (141, 167, 186)
TEXT_DIM     = (90,  120, 145)
SUCCESS      = (16,  185, 129)
WARNING      = (245, 158, 11)
DANGER       = (239, 68,  68)
BADGE_INFO   = (21,  63,  84)
BADGE_CYAN   = (83,  211, 209)
TOPBAR_BG    = (10,  20,  31)

# ── Font loader ─────────────────────────────────────────────────────────────
def _try_font(path: str, size: int):
    try:
        return ImageFont.truetype(path, size)
    except Exception:
        return None

def load_fonts():
    candidates_sans = [
        "C:/Windows/Fonts/segoeui.ttf",
        "C:/Windows/Fonts/arial.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ]
    candidates_mono = [
        "C:/Windows/Fonts/consola.ttf",
        "C:/Windows/Fonts/cour.ttf",
        "/System/Library/Fonts/Monaco.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf",
    ]
    candidates_bold = [
        "C:/Windows/Fonts/segoeuib.ttf",
        "C:/Windows/Fonts/arialbd.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
    ]

    def best(candidates, size):
        for p in candidates:
            f = _try_font(p, size)
            if f:
                return f
        return ImageFont.load_default()

    return {
        "hero":    best(candidates_bold, 26),
        "title":   best(candidates_bold, 20),
        "body":    best(candidates_sans,  13),
        "body_sm": best(candidates_sans,  11),
        "label":   best(candidates_sans,  10),
        "mono":    best(candidates_mono,  11),
        "mono_sm": best(candidates_mono,  10),
        "eyebrow": best(candidates_sans,  9),
        "num":     best(candidates_bold, 28),
        "badge":   best(candidates_sans,  9),
    }

FONTS = load_fonts()

# ── Drawing helpers ──────────────────────────────────────────────────────────
def make_frame():
    img = Image.new("RGB", (W, H), APP_BG)
    draw_bg_gradient(img)
    return img, ImageDraw.Draw(img)

def draw_bg_gradient(img):
    # Subtle radial gradient hints via horizontal bands
    overlay = Image.new("RGB", (W, H), APP_BG)
    d = ImageDraw.Draw(overlay)
    for y in range(H):
        t = y / H
        r = int(8  + 4  * (1 - t))
        g = int(17 + 8  * (1 - t))
        b = int(26 + 15 * (1 - t))
        d.line([(0, y), (W, y)], fill=(r, g, b))
    img.paste(overlay)

def rounded_rect(draw, xy, radius=12, fill=None, outline=None, outline_width=1):
    x0, y0, x1, y1 = xy
    if fill:
        draw.rounded_rectangle([x0, y0, x1, y1], radius=radius, fill=fill,
                                outline=outline, width=outline_width)
    elif outline:
        draw.rounded_rectangle([x0, y0, x1, y1], radius=radius, fill=None,
                                outline=outline, width=outline_width)

def panel(draw, xy, radius=14):
    x0, y0, x1, y1 = xy
    rounded_rect(draw, [x0, y0, x1, y1], radius=radius,
                 fill=PANEL_BG, outline=(45, 82, 108), outline_width=1)
    # Subtle top highlight
    draw.rounded_rectangle([x0+1, y0+1, x1-1, y0+30], radius=radius,
                            fill=(25, 45, 63))

def badge(draw, x, y, text, tone="info"):
    tone_map = {
        "info":    ((21, 63, 84),  BADGE_CYAN),
        "success": ((10, 60, 40),  SUCCESS),
        "warning": ((60, 45, 10),  WARNING),
        "danger":  ((60, 20, 20),  DANGER),
        "pending": ((25, 35, 48),  TEXT_SOFT),
    }
    bg, fg = tone_map.get(tone, tone_map["pending"])
    tw = FONTS["badge"].getlength(text) if hasattr(FONTS["badge"], "getlength") else len(text) * 6
    px, py = 8, 4
    bw = int(tw) + px * 2
    bh = 18
    rounded_rect(draw, [x, y, x + bw, y + bh], radius=9, fill=bg)
    draw.text((x + px, y + py - 1), text, font=FONTS["badge"], fill=fg)
    return bw

def confidence_bar(draw, x, y, w, value, h=4):
    rounded_rect(draw, [x, y, x + w, y + h], radius=2, fill=(39, 39, 42))
    if value and value > 0:
        filled = int(w * value)
        color = SUCCESS if value >= 0.85 else (WARNING if value >= 0.60 else DANGER)
        rounded_rect(draw, [x, y, x + filled, y + h], radius=2, fill=color)

def text_truncate(font, text, max_w):
    if not hasattr(font, "getlength"):
        return text
    if font.getlength(text) <= max_w:
        return text
    while text and font.getlength(text + "…") > max_w:
        text = text[:-1]
    return text + "…"

# ── Sidebar ──────────────────────────────────────────────────────────────────
NAV_ITEMS = [
    ("Workspace", False),
    ("Ingestion", False),
    ("Query",     False),
    ("Extraction",False),
    ("Actions",   False),
    ("Audit",     False),
    ("Memory",    False),
    ("Benchmarks",False),
    ("Diagnostics",False),
]

def draw_sidebar(draw, active_label):
    # Background
    draw.rectangle([0, 0, SIDEBAR_W, H], fill=SIDEBAR_BG)
    # Right border
    draw.line([(SIDEBAR_W, 0), (SIDEBAR_W, H)], fill=(20, 42, 60))

    # Logo card
    panel(draw, [12, 12, SIDEBAR_W - 12, 118], radius=16)
    draw.text((24, 24), "ALIV", font=FONTS["eyebrow"], fill=ACCENT_CYAN)
    draw.text((24, 40), "Legal Intelligence", font=FONTS["title"], fill=TEXT_STRONG)
    draw.text((24, 64), "Workbench", font=FONTS["title"], fill=TEXT_STRONG)
    draw.text((24, 88), "Local-first  ·  Offline  ·  Cited answers", font=FONTS["body_sm"], fill=TEXT_SOFT)

    # Nav items
    y = 132
    for label, _ in NAV_ITEMS:
        is_active = label == active_label
        if is_active:
            rounded_rect(draw, [12, y, SIDEBAR_W - 12, y + 36], radius=10,
                         fill=(18, 40, 58), outline=(83, 211, 209, 80))
            # Cyan left accent bar
            draw.rounded_rectangle([12, y + 6, 16, y + 30], radius=2, fill=ACCENT_CYAN)
            draw.text((28, y + 10), label, font=FONTS["body"], fill=TEXT_STRONG)
        else:
            draw.text((28, y + 10), label, font=FONTS["body"], fill=TEXT_DIM)
        y += 42

    # Shortcuts card
    panel(draw, [12, H - 80, SIDEBAR_W - 12, H - 12], radius=12)
    draw.text((24, H - 68), "⌘K  switch   ⌘↵  run   Esc  close",
              font=FONTS["mono_sm"], fill=TEXT_DIM)

def draw_topbar(draw, screen_name, workspace="demo/corpus", ollama_ok=True):
    draw.rectangle([SIDEBAR_W, 0, W, 62], fill=TOPBAR_BG)
    draw.line([(SIDEBAR_W, 62), (W, 62)], fill=(20, 42, 60))
    draw.text((SIDEBAR_W + 20, 14), screen_name.upper(),
              font=FONTS["eyebrow"], fill=(ACCENT_CYAN[0], ACCENT_CYAN[1], ACCENT_CYAN[2]))
    draw.text((SIDEBAR_W + 20, 32), f"📁 {workspace}",
              font=FONTS["body_sm"], fill=TEXT_SOFT)
    status = "Ollama online" if ollama_ok else "Ollama offline"
    tone = SUCCESS if ollama_ok else DANGER
    col = W - 160
    badge(draw, col, 22, status, "success" if ollama_ok else "danger")

def page_header(draw, eyebrow, title, desc):
    x = SIDEBAR_W + 24
    draw.text((x, 80), eyebrow.upper(), font=FONTS["eyebrow"], fill=ACCENT_CYAN)
    draw.text((x, 98), title, font=FONTS["hero"], fill=TEXT_STRONG)
    draw.text((x, 130), desc, font=FONTS["body_sm"], fill=TEXT_SOFT)

def stat_card(draw, x, y, w, h, label, value, detail=""):
    panel(draw, [x, y, x + w, y + h], radius=12)
    draw.text((x + 14, y + 14), label.upper(), font=FONTS["eyebrow"], fill=TEXT_DIM)
    draw.text((x + 14, y + 30), str(value), font=FONTS["num"], fill=TEXT_STRONG)
    if detail:
        draw.text((x + 14, y + h - 18), detail, font=FONTS["body_sm"], fill=TEXT_SOFT)

# ── Screen builders ──────────────────────────────────────────────────────────

def screen_query_typing(progress: float):
    """Query screen: text being typed in the prompt box."""
    img, draw = make_frame()
    draw_sidebar(draw, "Query")
    draw_topbar(draw, "Query & Assemble")
    page_header(draw, "Research", "Query & Assemble",
                "Run retrieval over the active workspace and inspect ranked evidence.")

    full_query = "Find all agreements expiring in 90 days and extract the penalty clauses."
    visible = full_query[:max(1, int(len(full_query) * progress))]

    cx = SIDEBAR_W + 24
    cy = 160

    # Stat cards
    stat_card(draw, cx,       cy, 195, 72, "Total Hits",     "—",  "Top K 5")
    stat_card(draw, cx + 205, cy, 195, 72, "Prompt Tokens",  "—",  "Latency — ms")
    stat_card(draw, cx + 410, cy, 195, 72, "Conversation",   "new","Turns 0")

    # Prompt panel
    panel(draw, [cx, cy + 84, cx + 380, cy + 84 + 300], radius=14)
    draw.text((cx + 14, cy + 98), "Prompt", font=FONTS["body"], fill=TEXT_STRONG)
    # Textarea
    rounded_rect(draw, [cx + 14, cy + 120, cx + 366, cy + 320], radius=10,
                 fill=(8, 18, 29), outline=(45, 80, 108))
    # Typed text with word wrap
    words = visible.split(" ")
    line, lines = "", []
    max_chars = 42
    for w_ in words:
        if len(line) + len(w_) + 1 <= max_chars:
            line += (" " if line else "") + w_
        else:
            lines.append(line)
            line = w_
    if line:
        lines.append(line)
    ty = cy + 132
    for ln in lines[:6]:
        draw.text((cx + 22, ty), ln, font=FONTS["body"], fill=TEXT_STRONG)
        ty += 17

    # Blinking cursor
    if int(progress * 20) % 2 == 0:
        cx_ = cx + 22 + int(FONTS["body"].getlength(lines[-1]) if lines and hasattr(FONTS["body"], "getlength") else 0)
        draw.rectangle([cx_, ty - 17, cx_ + 1, ty - 4], fill=ACCENT_CYAN)

    # Run button
    rounded_rect(draw, [cx + 240, cy + 330, cx + 366, cy + 356], radius=10,
                 fill=ACCENT_CYAN)
    draw.text((cx + 275, cy + 338), "Run query", font=FONTS["body"], fill=(8, 17, 26))

    draw.text((cx + 14, cy + 332), "Ctrl/Cmd + Enter", font=FONTS["mono_sm"], fill=TEXT_DIM)

    return img


def screen_query_running():
    """Query screen: 'Running…' spinner state."""
    img, draw = make_frame()
    draw_sidebar(draw, "Query")
    draw_topbar(draw, "Query & Assemble")
    page_header(draw, "Research", "Query & Assemble",
                "Run retrieval over the active workspace and inspect ranked evidence.")

    cx = SIDEBAR_W + 24
    cy = 160
    stat_card(draw, cx,       cy, 195, 72, "Total Hits",    "—",  "Top K 5")
    stat_card(draw, cx + 205, cy, 195, 72, "Prompt Tokens", "—",  "Latency — ms")
    stat_card(draw, cx + 410, cy, 195, 72, "Conversation",  "new","Turns 0")

    panel(draw, [cx, cy + 84, cx + 380, cy + 84 + 300], radius=14)
    draw.text((cx + 14, cy + 98), "Prompt", font=FONTS["body"], fill=TEXT_STRONG)
    rounded_rect(draw, [cx + 14, cy + 120, cx + 366, cy + 320], radius=10,
                 fill=(8, 18, 29), outline=(45, 80, 108))
    query_text = "Find all agreements expiring in 90 days"
    draw.text((cx + 22, cy + 132), query_text, font=FONTS["body"], fill=TEXT_STRONG)
    draw.text((cx + 22, cy + 150), "and extract the penalty clauses.", font=FONTS["body"], fill=TEXT_STRONG)

    # Running pill
    rounded_rect(draw, [cx + 180, cy + 330, cx + 366, cy + 356], radius=10,
                 fill=(18, 40, 58), outline=ACCENT_CYAN)
    draw.text((cx + 205, cy + 338), "⟳  Running…", font=FONTS["body"], fill=ACCENT_CYAN)

    # Assembling context status
    panel(draw, [cx + 400, cy + 84, W - 24, cy + 144], radius=12)
    draw.text((cx + 414, cy + 98), "Assembling context from 5 documents…",
              font=FONTS["body_sm"], fill=TEXT_SOFT)
    rounded_rect(draw, [cx + 414, cy + 116, W - 38, cy + 124], radius=3, fill=(25, 45, 63))
    rounded_rect(draw, [cx + 414, cy + 116, cx + 580, cy + 124], radius=3, fill=ACCENT_CYAN)

    return img


def screen_query_results():
    """Query screen: results with 3 evidence hit cards."""
    img, draw = make_frame()
    draw_sidebar(draw, "Query")
    draw_topbar(draw, "Query & Assemble")
    page_header(draw, "Research", "Query & Assemble",
                "Run retrieval over the active workspace and inspect ranked evidence.")

    cx = SIDEBAR_W + 24
    cy = 160

    stat_card(draw, cx,       cy, 195, 72, "Total Hits",    "3",    "Top K 5")
    stat_card(draw, cx + 205, cy, 195, 72, "Prompt Tokens", "1,847","Latency 2,340 ms")
    stat_card(draw, cx + 410, cy, 195, 72, "Conversation",  "conv-001","Turns 1")

    # Answer panel
    panel(draw, [cx, cy + 84, cx + 390, cy + 84 + 300], radius=14)
    draw.text((cx + 14, cy + 100), "Assembled answer", font=FONTS["body"], fill=TEXT_STRONG)
    answer_lines = [
        "Three agreements have penalty clauses relevant",
        "to expiry within 90 days:",
        "",
        "1. Lease — Acme Commercial (exp. 2026-03-01)",
        "   Holdover: $2,800/day after expiry [§5]",
        "",
        "2. NDA — Axiom Legal (exp. 2026-07-15)",
        "   Breach penalty: $50,000 per incident [§5]",
        "",
        "3. Service Agmt — Delta (exp. 2025-12-31)",
        "   Delivery delay: $5,000/day, cap $60k/qtr [§4]",
    ]
    ty = cy + 122
    for ln in answer_lines:
        col = ACCENT_CYAN if ln.startswith(("1.", "2.", "3.")) else TEXT_SOFT if ln.startswith("  ") else TEXT_STRONG
        draw.text((cx + 14, ty), ln, font=FONTS["body_sm"], fill=col)
        ty += 16

    # Evidence cards
    hits = [
        ("0.947", "lease_acme_2025.txt", "§5",
         "$2,800/day holdover penalty shall apply for",
         "failure to vacate after lease expiry.", SUCCESS),
        ("0.891", "nda_axiom_legal_2025.txt", "§5",
         "Liquidated damages of $50,000 per breach",
         "of confidentiality obligations.", WARNING),
        ("0.823", "service_agreement_delta_2025.txt", "§4",
         "$5,000/day for delivery delays exceeding",
         "five business days. Cap: $60k/quarter.", ACCENT_INDIGO),
    ]
    hx = cx + 404
    hy = cy + 84
    for score, fname, page, line1, line2, col in hits:
        panel(draw, [hx, hy, W - 24, hy + 94], radius=12)
        badge(draw, hx + 10, hy + 10, f"score {score}", "info")
        badge(draw, hx + 100, hy + 10, "cited", "success")
        draw.text((hx + 10, hy + 34), text_truncate(FONTS["mono"], fname, 240),
                  font=FONTS["mono"], fill=TEXT_DIM)
        draw.text((hx + 240, hy + 34), page, font=FONTS["mono"], fill=ACCENT_CYAN)
        draw.text((hx + 10, hy + 55), line1, font=FONTS["body_sm"], fill=TEXT_STRONG)
        draw.text((hx + 10, hy + 71), line2, font=FONTS["body_sm"], fill=TEXT_SOFT)
        # Confidence bar at bottom
        confidence_bar(draw, hx + 10, hy + 88, W - 24 - hx - 20, 0.9 - (hits.index((score, fname, page, line1, line2, col)) * 0.05))
        hy += 102
    return img


def screen_extraction():
    """Extraction screen: structured fields with confidence bars."""
    img, draw = make_frame()
    draw_sidebar(draw, "Extraction")
    draw_topbar(draw, "Extraction")
    page_header(draw, "Structured Output", "Extraction",
                "Run a templated extraction pass and inspect the returned structured payload.")

    cx = SIDEBAR_W + 24
    cy = 160

    # Left: input panel
    panel(draw, [cx, cy, cx + 300, H - 24], radius=14)
    draw.text((cx + 14, cy + 14), "Extraction inputs", font=FONTS["body"], fill=TEXT_STRONG)
    draw.text((cx + 14, cy + 38), "Instruction", font=FONTS["body_sm"], fill=TEXT_SOFT)
    rounded_rect(draw, [cx + 14, cy + 54, cx + 286, cy + 200], radius=10,
                 fill=(8, 18, 29), outline=(45, 80, 108))
    instr_lines = [
        "Extract vendor name, client,",
        "effective date, expiry date,",
        "renewal window, and",
        "penalty clause from the",
        "lease agreement.",
    ]
    iy = cy + 66
    for ln in instr_lines:
        draw.text((cx + 22, iy), ln, font=FONTS["body_sm"], fill=TEXT_STRONG)
        iy += 16

    draw.text((cx + 14, cy + 212), "Template", font=FONTS["body_sm"], fill=TEXT_SOFT)
    rounded_rect(draw, [cx + 14, cy + 228, cx + 286, cy + 252], radius=8,
                 fill=(8, 18, 29), outline=(45, 80, 108))
    draw.text((cx + 22, cy + 234), "Lease summary", font=FONTS["body_sm"], fill=TEXT_STRONG)

    # Run button
    rounded_rect(draw, [cx + 14, cy + 264, cx + 286, cy + 290], radius=10,
                 fill=ACCENT_CYAN)
    draw.text((cx + 80, cy + 272), "Run extraction", font=FONTS["body"], fill=(8, 17, 26))

    # Right: fields table
    fields = [
        ("vendor_name",    "Acme Commercial Properties LLC",    0.96),
        ("client_name",    "Redstone Enterprises Inc.",         0.95),
        ("effective_date", "2025-03-01",                        0.98),
        ("expiry_date",    "2026-03-01",                        0.98),
        ("renewal_window", "60 days written notice by Jan 1",   0.91),
        ("penalty_clause", "$2,800/day holdover after expiry",  0.93),
    ]

    tx = cx + 316
    ty = cy

    # Header
    panel(draw, [tx, ty, W - 24, ty + 36], radius=10)
    draw.text((tx + 14, ty + 11), "FIELD", font=FONTS["eyebrow"], fill=TEXT_DIM)
    draw.text((tx + 180, ty + 11), "VALUE", font=FONTS["eyebrow"], fill=TEXT_DIM)
    draw.text((tx + 490, ty + 11), "CONFIDENCE", font=FONTS["eyebrow"], fill=TEXT_DIM)

    ty += 44
    for i, (field, value, conf) in enumerate(fields):
        bg = (14, 28, 42) if i % 2 == 0 else (11, 22, 34)
        draw.rectangle([tx, ty, W - 24, ty + 46], fill=bg)
        draw.text((tx + 14, ty + 8), field, font=FONTS["mono"], fill=ACCENT_CYAN)
        draw.text((tx + 180, ty + 8), text_truncate(FONTS["body_sm"], value, 290),
                  font=FONTS["body_sm"], fill=TEXT_STRONG)
        pct_text = f"{int(conf * 100)}%"
        draw.text((tx + 490, ty + 8), pct_text, font=FONTS["body_sm"], fill=TEXT_SOFT)
        confidence_bar(draw, tx + 490, ty + 26, 160, conf, h=5)
        ty += 46

    # Export button hint
    rounded_rect(draw, [W - 140, H - 52, W - 24, H - 28], radius=10,
                 fill=(14, 25, 38), outline=(45, 80, 108))
    draw.text((W - 125, H - 44), "Export CSV", font=FONTS["body_sm"], fill=TEXT_SOFT)

    return img


def screen_actions_preview():
    """Actions Preview: rename recommendations with risk badges."""
    img, draw = make_frame()
    draw_sidebar(draw, "Actions")
    draw_topbar(draw, "Safe File Actions")
    page_header(draw, "Guardrails", "Safe File Actions",
                "Review rename recommendations, approve a bounded batch, then execute.")

    cx = SIDEBAR_W + 24
    cy = 160

    stat_card(draw, cx,       cy, 150, 72, "Recommendations", "5",  "Generated")
    stat_card(draw, cx + 162, cy, 150, 72, "Selected",        "3",  "Low-risk only")
    stat_card(draw, cx + 324, cy, 150, 72, "High Risk",       "2",  "Pre-deselected")
    stat_card(draw, cx + 486, cy, 150, 72, "Approval",        "Ready", "Gate cleared")

    # Approval status bar
    rounded_rect(draw, [cx, cy + 84, W - 24, cy + 110], radius=10,
                 fill=(10, 40, 28), outline=(16, 185, 129, 60))
    draw.text((cx + 14, cy + 94), "✓  3 low-risk renames approved. Execute when ready.",
              font=FONTS["body_sm"], fill=SUCCESS)

    # Table header
    ty = cy + 122
    panel(draw, [cx, ty, W - 24, ty + 30], radius=8)
    cols = ["", "SOURCE → TARGET", "TYPE", "RISK", "CONFIDENCE"]
    cw   = [28, 490, 120, 80, 140]
    tx_ = cx + 10
    for col_label, cw_ in zip(cols, cw):
        draw.text((tx_ + 4, ty + 9), col_label, font=FONTS["eyebrow"], fill=TEXT_DIM)
        tx_ += cw_

    rows = [
        (True,  "invoice_harbourview_2026.txt",
                "Invoice_HarbourviewIT_2026-02-01.txt",
                "rename", "low",    0.94),
        (True,  "nda_axiom_legal_2025.txt",
                "NDA_AxiomLegal_2025-07-15.txt",
                "rename", "low",    0.91),
        (True,  "service_agreement_delta_2025.txt",
                "ServiceAgmt_DeltaAdvisory_2025-01-10.txt",
                "rename", "medium", 0.78),
        (False, "lease_acme_2025.txt",
                "Lease_AcmeCommercial_2025-03-01.txt",
                "rename", "high",   0.65),
        (False, "email_renewal_notice_2025.txt",
                "Email_AcmeRenewal_2025-11-15.txt",
                "rename", "high",   0.61),
    ]

    ty += 38
    for checked, src, dst, rtype, risk, conf in rows:
        bg = (60, 15, 15) if risk == "high" else (14, 28, 42)
        draw.rectangle([cx, ty, W - 24, ty + 52], fill=bg)
        draw.line([(cx, ty + 52), (W - 24, ty + 52)], fill=(20, 40, 58))

        # Checkbox
        cb_col = (45, 82, 108)
        rounded_rect(draw, [cx + 6, ty + 18, cx + 20, ty + 32], radius=3,
                     fill=ACCENT_CYAN if checked else (8, 18, 29),
                     outline=cb_col)
        if checked:
            draw.text((cx + 8, ty + 17), "✓", font=FONTS["badge"], fill=(8, 17, 26))

        # Source/Target
        col2_x = cx + 38
        draw.text((col2_x, ty + 10), text_truncate(FONTS["mono"], src, 460),
                  font=FONTS["mono"], fill=TEXT_DIM)
        draw.text((col2_x, ty + 30), text_truncate(FONTS["mono"], dst, 460),
                  font=FONTS["mono"], fill=TEXT_STRONG)

        # Type badge
        badge(draw, cx + 538, ty + 18, rtype, "pending")

        # Risk badge
        risk_tone = {"low": "success", "medium": "warning", "high": "danger"}.get(risk, "pending")
        badge(draw, cx + 658, ty + 18, risk, risk_tone)

        # Confidence
        draw.text((cx + 748, ty + 10), f"{int(conf * 100)}%", font=FONTS["body_sm"], fill=TEXT_SOFT)
        confidence_bar(draw, cx + 748, ty + 30, 140, conf, h=5)

        ty += 54

    # Execute button
    rounded_rect(draw, [W - 190, H - 52, W - 24, H - 28], radius=10, fill=ACCENT_CYAN)
    draw.text((W - 175, H - 44), "Execute approved", font=FONTS["body"], fill=(8, 17, 26))

    return img


def screen_undo_banner():
    """Actions Preview with undo success banner."""
    img, draw = make_frame()
    draw_sidebar(draw, "Actions")
    draw_topbar(draw, "Safe File Actions")
    page_header(draw, "Guardrails", "Safe File Actions",
                "Review rename recommendations, approve a bounded batch, then execute.")

    cx = SIDEBAR_W + 24
    cy = 160

    stat_card(draw, cx,       cy, 150, 72, "Recommendations", "5",  "Loaded")
    stat_card(draw, cx + 162, cy, 150, 72, "Selected",        "0",  "Post-execute")
    stat_card(draw, cx + 324, cy, 150, 72, "Succeeded",       "3",  "Renamed")
    stat_card(draw, cx + 486, cy, 150, 72, "Approval",        "Cleared", "Gate reset")

    # Success banner
    rounded_rect(draw, [cx, cy + 84, W - 24, cy + 114], radius=10,
                 fill=(10, 40, 28), outline=(16, 185, 129))
    draw.text((cx + 14, cy + 91),
              "✓  Batch batch-a3f9 executed — 3 files renamed successfully.",
              font=FONTS["body_sm"], fill=SUCCESS)
    draw.text((cx + 14, cy + 105),
              "   Click Undo to reverse this batch.",
              font=FONTS["body_sm"], fill=TEXT_SOFT)

    # Undo button hint
    rounded_rect(draw, [W - 140, cy + 90, W - 24, cy + 114], radius=8,
                 fill=(14, 28, 42), outline=(45, 82, 108))
    draw.text((W - 120, cy + 97), "↩  Undo batch", font=FONTS["body_sm"], fill=TEXT_SOFT)

    # Audit trail preview
    panel(draw, [cx, cy + 126, W - 24, H - 24], radius=14)
    draw.text((cx + 14, cy + 140), "Execution audit trail", font=FONTS["body"], fill=TEXT_STRONG)
    audit_rows = [
        ("Invoice_HarbourviewIT_2026-02-01.txt", "success",     "batch-a3f9"),
        ("NDA_AxiomLegal_2025-07-15.txt",        "success",     "batch-a3f9"),
        ("ServiceAgmt_DeltaAdvisory_2025-01-10.txt", "success", "batch-a3f9"),
    ]
    aty = cy + 166
    for fname, status, batch in audit_rows:
        draw.text((cx + 14, aty), fname, font=FONTS["mono_sm"], fill=TEXT_STRONG)
        badge(draw, cx + 450, aty - 2, status, "success")
        draw.text((cx + 560, aty), batch, font=FONTS["mono_sm"], fill=TEXT_DIM)
        aty += 24
    return img


# ── Animation sequence ───────────────────────────────────────────────────────
def build_frames():
    frames = []

    def add(img, count=1):
        for _ in range(count):
            frames.append(img.copy())

    # Screen 1: Typing query  (2 seconds = 24 frames at FPS=12)
    typing_frames = FPS * 2
    for i in range(typing_frames):
        prog = (i + 1) / typing_frames
        frames.append(screen_query_typing(prog))

    # Hold on full typed query
    add(screen_query_typing(1.0), FPS // 2)

    # Screen 2: Running (1 second)
    add(screen_query_running(), FPS)

    # Screen 3: Results (4 seconds)
    add(screen_query_results(), FPS * 4)

    # Screen 4: Extraction (3.5 seconds)
    add(screen_extraction(), int(FPS * 3.5))

    # Screen 5: Actions Preview (3 seconds)
    add(screen_actions_preview(), FPS * 3)

    # Screen 6: Undo banner (2.5 seconds)
    add(screen_undo_banner(), int(FPS * 2.5))

    # Brief hold at end before loop
    add(screen_query_typing(0.0), FPS // 2)

    return frames


# ── GIF encoder ──────────────────────────────────────────────────────────────
def save_gif(frames):
    print(f"Encoding {len(frames)} frames to {OUT}")
    delay_ms = int(1000 / FPS)

    # Build a shared palette from a spread of sample frames so all frames
    # use the same colour table — this ensures Pillow keeps every frame.
    sample_idx = list(range(0, len(frames), max(1, len(frames) // 16)))
    combined = Image.new("RGB", (W * len(sample_idx), H))
    for pos, fi in enumerate(sample_idx):
        combined.paste(frames[fi], (pos * W, 0))
    palette_ref = combined.quantize(colors=256, method=Image.Quantize.MEDIANCUT)

    print("Quantizing frames with shared palette...")
    quantized = []
    for i, img in enumerate(frames):
        q = img.quantize(palette=palette_ref, dither=1)
        quantized.append(q)
        if i % 30 == 0:
            sys.stdout.write(f"  frame {i}/{len(frames)}\r")
            sys.stdout.flush()

    print(f"\nSaving GIF ({len(quantized)} frames)...")
    quantized[0].save(
        OUT,
        format="GIF",
        save_all=True,
        append_images=quantized[1:],
        duration=delay_ms,
        loop=0,
        optimize=False,
    )
    size_mb = OUT.stat().st_size / 1024 / 1024
    print(f"Done: {OUT}  ({size_mb:.1f} MB, {len(frames)} frames, {FPS} fps)")


if __name__ == "__main__":
    print("ALIV Demo GIF Generator")
    print(f"Output: {OUT}")
    print(f"Dimensions: {W}×{H}  FPS: {FPS}")
    frames = build_frames()
    save_gif(frames)
