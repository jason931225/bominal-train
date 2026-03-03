from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from html import escape
import re
from typing import Any, Literal

Theme = Literal["spring", "summer", "autumn", "winter"]

THEMES: dict[str, dict[str, str]] = {
    "spring": {
        "accent": "#DA5D8F",
        "accent_text": "#8F3A5F",
        "bg1": "#FFF4F9",
        "bg2": "#FFFFFF",
        "bg3": "#FFF3F7",
        "t1": "#FFC2DA",
        "t2": "#FFDBE9",
    },
    "summer": {
        "accent": "#249F85",
        "accent_text": "#1F6658",
        "bg1": "#F5FCFB",
        "bg2": "#FFFFFF",
        "bg3": "#F4F9FF",
        "t1": "#9CE4CF",
        "t2": "#9AD4F4",
    },
    "autumn": {
        "accent": "#C35B21",
        "accent_text": "#7D3A1F",
        "bg1": "#FFFAF5",
        "bg2": "#FFFFFF",
        "bg3": "#FFF7EF",
        "t1": "#FFBD7F",
        "t2": "#E09A6B",
    },
    "winter": {
        "accent": "#4C69B4",
        "accent_text": "#354878",
        "bg1": "#F6F9FF",
        "bg2": "#FFFFFF",
        "bg3": "#F3F7FF",
        "t1": "#ADC5EF",
        "t2": "#CEDDF7",
    },
}

BlockType = Literal["hero", "paragraph", "cta", "otp", "kv", "bullets", "mono", "divider"]


@dataclass(frozen=True)
class Block:
    type: BlockType
    data: dict[str, Any]


@dataclass(frozen=True)
class RenderedEmail:
    subject: str
    preheader: str
    html: str
    text: str


_POINTER_PATTERN = re.compile(r"{{\s*([a-zA-Z0-9_.-]+)\s*}}")


def _lookup_pointer(context: dict[str, Any], pointer: str) -> Any:
    current: Any = context
    for segment in pointer.split("."):
        if isinstance(current, dict) and segment in current:
            current = current[segment]
        else:
            return None
    return current


def _resolve_with_context(value: Any, context: dict[str, Any]) -> Any:
    if isinstance(value, str):

        def _replace(match: re.Match[str]) -> str:
            found = _lookup_pointer(context, match.group(1))
            return "" if found is None else str(found)

        return _POINTER_PATTERN.sub(_replace, value)

    if isinstance(value, dict):
        if "$ref" in value:
            pointer = str(value.get("$ref", ""))
            resolved = _lookup_pointer(context, pointer)
            if resolved is None:
                return value.get("default")
            return resolved
        return {k: _resolve_with_context(v, context) for k, v in value.items()}

    if isinstance(value, list):
        return [_resolve_with_context(item, context) for item in value]

    return value


def format_completion_summary(
    *,
    status: str,
    task: str,
    module: str,
    completion_date: str,
    completion_time: str,
    item: str,
    target_date: str,
    people: int,
) -> str:
    return (
        f"{status} {task} for {module} on {completion_date} {completion_time} - "
        f"{item} {target_date} for {people} people."
    )


def _card_shell(*, subject: str, preheader: str, theme: dict[str, str]) -> list[str]:
    return [
        "<!doctype html><html><head><meta charset='utf-8'/>",
        "<meta name='viewport' content='width=device-width,initial-scale=1'/>",
        f"<title>{escape(subject)}</title>",
        "<style>",
        "body,table,td,p{margin:0;padding:0} table{border-collapse:collapse!important}",
        f".bg{{background:linear-gradient(180deg,{theme['bg1']} 0%,{theme['bg2']} 40%,{theme['bg3']} 100%);padding:28px 12px}}",
        ".card{max-width:640px;margin:0 auto;background:rgba(255,255,255,.82);"
        "border:1px solid rgba(255,255,255,.75);border-radius:26px;"
        "box-shadow:0 12px 30px -18px rgba(15,23,42,.40);overflow:hidden}",
        f".accent{{height:8px;background:linear-gradient(90deg,{theme['t1']} 0%,{theme['t2']} 55%,{theme['accent']} 110%)}}",
        ".pad{padding:22px 22px 18px 22px}",
        f".brand{{font-size:28px;font-weight:700;letter-spacing:-.02em;text-transform:lowercase;color:{theme['accent_text']};margin-bottom:10px}}",
        ".h1{font-size:22px;line-height:28px;margin:8px 0 6px 0;color:#0f172a;font-weight:800}",
        ".sub{font-size:14px;line-height:20px;color:#334155;margin-bottom:14px}",
        ".p{font-size:13px;line-height:19px;color:#0f172a;margin-top:10px}",
        f".btn{{display:inline-block;background:{theme['accent']};color:#fff!important;padding:12px 16px;border-radius:16px;font-weight:800;text-decoration:none}}",
        ".muted{color:#64748b}",
        ".hr{height:1px;background:rgba(148,163,184,.25);margin:16px 0}",
        ".otp{border:1px solid rgba(148,163,184,.25);border-radius:18px;padding:14px;background:rgba(255,255,255,.60);margin-top:12px}",
        ".otpl{font-size:12px;color:#475569;margin-bottom:6px}",
        ".otpc{font-family:ui-monospace,Menlo,Consolas,monospace;font-size:26px;letter-spacing:.22em;color:#0f172a}",
        ".otpm{font-size:12px;color:#64748b;margin-top:6px}",
        ".kv{width:100%;margin-top:10px;border:1px solid rgba(148,163,184,.20);border-radius:18px;overflow:hidden}",
        ".kv td{padding:10px 12px;font-size:13px;line-height:18px}",
        ".k{color:#475569;width:34%;background:rgba(255,255,255,.55)}",
        ".v{color:#0f172a;background:rgba(255,255,255,.75)}",
        ".mono{margin-top:12px;padding:12px;border-radius:16px;border:1px dashed rgba(148,163,184,.35);"
        "background:rgba(255,255,255,.55);font-family:ui-monospace,Menlo,Consolas,monospace;font-size:12px;line-height:18px;word-break:break-word}",
        "ul{margin:10px 0 0 18px;padding:0} li{margin:6px 0;line-height:18px;font-size:13px;color:#0f172a}",
        ".preheader{display:none!important;visibility:hidden;opacity:0;color:transparent;height:0;width:0}",
        ".footer{padding:16px 22px 22px;color:#475569;font-size:12px;line-height:18px}",
        "</style></head><body>",
        f"<div class='preheader'>{escape(preheader)}</div>",
        "<table role='presentation' width='100%' class='bg'><tr><td>",
        "<div class='card'><div class='accent'></div><div class='pad'>",
        "<div class='brand'>bominal</div>",
    ]


def render_email(
    *,
    subject: str,
    preheader: str,
    blocks: list[Block],
    theme: Theme | str = "spring",
    context: dict[str, Any] | None = None,
    footer_support_text: str = "Need help?",
    footer_support_url: str | None = None,
    footer_legal_text: str = "If you didn't request this, you can ignore this email.",
) -> RenderedEmail:
    context = context or {}
    resolved_subject = str(_resolve_with_context(subject, context))
    resolved_preheader = str(_resolve_with_context(preheader, context))
    resolved_footer_support_text = str(_resolve_with_context(footer_support_text, context))
    resolved_footer_support_url_raw = (
        _resolve_with_context(footer_support_url, context) if footer_support_url is not None else None
    )
    resolved_footer_support_url = str(resolved_footer_support_url_raw) if resolved_footer_support_url_raw else None
    resolved_footer_legal_text = str(_resolve_with_context(footer_legal_text, context))

    palette = THEMES.get(str(theme).lower(), THEMES["spring"])
    html_parts = _card_shell(subject=resolved_subject, preheader=resolved_preheader, theme=palette)
    text_lines: list[str] = ["bominal", "", resolved_preheader, ""]

    for block in blocks:
        data = _resolve_with_context(block.data, context)
        if block.type == "hero":
            title = str(data.get("title", ""))
            subtitle = str(data.get("subtitle", "")).strip()
            html_parts.append(f"<div class='h1'>{escape(title)}</div>")
            if subtitle:
                html_parts.append(f"<div class='sub'>{escape(subtitle)}</div>")
            text_lines.extend([title, subtitle] if subtitle else [title])
            text_lines.append("")
            continue

        if block.type == "paragraph":
            text = str(data.get("text", ""))
            html_parts.append(f"<div class='p'>{escape(text)}</div>")
            text_lines.extend([text, ""])
            continue

        if block.type == "cta":
            label = str(data.get("label", "Continue"))
            url = str(data.get("url", ""))
            helper = str(data.get("helper_text", "")).strip()
            html_parts.append(
                f"<div style='margin:12px 0 6px 0;'><a class='btn' href='{escape(url)}'>{escape(label)}</a></div>"
            )
            if helper:
                html_parts.append(
                    f"<div class='muted' style='font-size:12px;line-height:18px;margin-top:4px;'>{escape(helper)}</div>"
                )
            if url:
                html_parts.append(
                    f"<div class='muted' style='font-size:12px;line-height:18px;word-break:break-all;margin-top:8px;'>"
                    f"If the button doesn't work, copy and paste this link:<br>{escape(url)}</div>"
                )
            text_lines.append(f"{label}: {url}")
            if helper:
                text_lines.append(helper)
            text_lines.append("")
            continue

        if block.type == "otp":
            code = str(data.get("code", ""))
            ttl_minutes = int(data.get("ttl_minutes", 10))
            label = str(data.get("label", "One-time code"))
            html_parts.append(
                f"<div class='otp'><div class='otpl'>{escape(label)}</div><div class='otpc'>{escape(code)}</div>"
                f"<div class='otpm'>Expires in {ttl_minutes} minutes.</div></div>"
            )
            text_lines.extend([f"{label}: {code}", f"Expires in {ttl_minutes} minutes.", ""])
            continue

        if block.type == "kv":
            rows = data.get("rows") or []
            html_parts.append("<table class='kv' role='presentation' width='100%'>")
            for row in rows:
                k = str(row.get("k", ""))
                v = str(row.get("v", ""))
                html_parts.append(f"<tr><td class='k'>{escape(k)}</td><td class='v'>{escape(v)}</td></tr>")
                text_lines.append(f"{k}: {v}")
            html_parts.append("</table>")
            text_lines.append("")
            continue

        if block.type == "bullets":
            items = [str(item) for item in data.get("items") or []]
            html_parts.append("<ul>" + "".join(f"<li>{escape(item)}</li>" for item in items) + "</ul>")
            text_lines.extend([f"- {item}" for item in items])
            text_lines.append("")
            continue

        if block.type == "mono":
            text = str(data.get("text", ""))
            html_parts.append(f"<div class='mono'>{escape(text)}</div>")
            text_lines.extend([text, ""])
            continue

        if block.type == "divider":
            html_parts.append("<div class='hr'></div>")
            text_lines.extend(["------------------------", ""])
            continue

    year = datetime.now().year
    html_parts.append("</div>")
    html_parts.append("<div class='footer'>")
    if resolved_footer_support_url:
        html_parts.append(
            f"<div><a href='{escape(resolved_footer_support_url)}' style='color:{palette['accent_text']};text-decoration:none;font-weight:800;'>"
            f"{escape(resolved_footer_support_text)}</a></div>"
        )
    else:
        html_parts.append(
            f"<div style='font-weight:800;color:{palette['accent_text']};'>{escape(resolved_footer_support_text)}</div>"
        )
    html_parts.append(f"<div class='muted' style='margin-top:6px;'>{escape(resolved_footer_legal_text)}</div>")
    html_parts.append("</div></div>")
    html_parts.append(
        f"<div style='max-width:640px;margin:10px auto 0 auto;text-align:center;color:#94a3b8;font-size:11px;line-height:16px;'>&copy; {year} bominal</div>"
    )
    html_parts.append("</td></tr></table></body></html>")

    text_lines.extend([resolved_footer_support_text, resolved_footer_support_url or "", resolved_footer_legal_text])
    text = "\n".join(line for line in text_lines if line is not None).strip() + "\n"
    html = "".join(html_parts)
    return RenderedEmail(subject=resolved_subject, preheader=resolved_preheader, html=html, text=text)
