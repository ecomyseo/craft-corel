"""Build the two Spanish manuals from their single Markdown sources."""

import argparse
import hashlib
import html
import json
import re
import shutil
import textwrap
from datetime import datetime, timezone
from pathlib import Path

import pymupdf as fitz
import markdown
import reportlab
from bs4 import BeautifulSoup, NavigableString
from PIL import Image, ImageDraw
from reportlab.lib import colors
from reportlab.lib.enums import TA_LEFT
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.platypus import (
    BaseDocTemplate, Frame, KeepTogether, LongTable, PageBreak, PageTemplate,
    Paragraph, Preformatted, Spacer, TableStyle,
)
from reportlab.platypus.tableofcontents import TableOfContents

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[2]
NAVY = colors.HexColor("#142e45")
TEAL = colors.HexColor("#087f8c")
GRAY = colors.HexColor("#536678")
WIDTH, HEIGHT = A4
MARGIN = 51
CONTENT_WIDTH = WIDTH - 2 * MARGIN

CSS = """
:root{--ink:#193347;--accent:#087f8c;--line:#dbe4e9;--paper:#fff}
*{box-sizing:border-box}html{scroll-behavior:smooth}body{margin:0;color:var(--ink);
background:#f1f5f8;font:16px/1.65 system-ui,-apple-system,'Segoe UI',sans-serif}
a{color:#076c87;text-underline-offset:3px}header{background:#142e45;color:white;padding:24px 5vw}
header p{margin:0;letter-spacing:.12em;font-size:12px}main{max-width:1060px;margin:24px auto;
padding:48px 66px;background:var(--paper);border:1px solid var(--line);border-radius:12px}
h1{font-size:2.5rem;line-height:1.16;margin:0 0 24px;color:#142e45;max-width:850px}
h2{font-size:1.65rem;line-height:1.25;margin:52px 0 20px;border-top:3px solid var(--accent);
padding-top:20px;scroll-margin-top:110px}h3{font-size:1.2rem;color:#087f8c}
p,li{max-width:85ch}li{margin:6px 0}table{border-collapse:collapse;width:100%;font-size:.92rem;
margin:24px 0;table-layout:fixed}th{background:#e8f2f4;text-align:left}th,td{border:1px solid
var(--line);padding:11px 13px;vertical-align:top;overflow-wrap:anywhere}tr:nth-child(even) td{
background:#f8fafb}code{font:.88em ui-monospace,Consolas,monospace;background:#edf3f5;padding:2px 4px;
border-radius:3px;overflow-wrap:anywhere}pre{white-space:pre-wrap;overflow-wrap:anywhere;padding:20px;
background:#edf3f5;border-left:4px solid var(--accent)}pre code{padding:0;background:none}
.toc{background:#f1f7f8;padding:24px 30px;border-radius:8px;margin:32px 0}.toc ul{margin:0}
.tools{position:sticky;top:0;background:#fff;border-bottom:1px solid var(--line);padding:12px 5vw;
z-index:4;display:flex;gap:16px;align-items:center;flex-wrap:wrap}.tools label{font-weight:600}
input[type=search]{font:inherit;min-width:220px;max-width:80vw;padding:8px 12px;border:1px solid
#a9becb;border-radius:6px}.count{font-size:13px;color:#526878}.hidden{display:none}
footer{max-width:1060px;margin:20px auto 50px;padding:0 24px;color:#536678;font-size:13px}
@media(max-width:700px){main{padding:25px 20px;margin:10px;border-radius:4px}h1{font-size:2rem}
table{font-size:.8rem}th,td{padding:8px}.tools{position:static}h2{scroll-margin-top:24px}}
@media print{body{background:white;font:10pt/1.45 sans-serif}.tools,header,footer{display:none}
main{margin:0;padding:0;max-width:none;border:0}h1{font-size:26pt}h2{font-size:16pt;margin-top:20pt;
break-after:avoid}p,li{max-width:none}table{font-size:8pt}tr{break-inside:avoid}
section.hidden{display:block!important}a{color:inherit}.toc{break-after:page}@page{size:A4;margin:18mm}}
"""

SEARCH = """
const input=document.querySelector('#search'),counter=document.querySelector('#count');
const sections=[...document.querySelectorAll('main>section')];
const normalize=s=>s.normalize('NFD').replace(/[\\u0300-\\u036f]/g,'').toLowerCase();
function filter(){const q=normalize(input.value.trim());let n=0;for(const s of sections){
 const show=!q||normalize(s.textContent).includes(q);s.classList.toggle('hidden',!show);n+=show;}
 counter.textContent=q?`${n} de ${sections.length} apartados`:'Todos los apartados';}
input.addEventListener('input',filter);document.addEventListener('keydown',e=>{
 if(e.key==='/'&&document.activeElement!==input){e.preventDefault();input.focus();}
 if(e.key==='Escape'){input.value='';filter();input.blur();}});filter();
"""


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def fonts():
    directory = Path(reportlab.__file__).parent / "fonts"
    for name, filename in [("Vera", "Vera.ttf"), ("Vera-Bold", "VeraBd.ttf"),
                           ("Vera-Italic", "VeraIt.ttf"), ("Vera-BoldItalic", "VeraBI.ttf")]:
        pdfmetrics.registerFont(TTFont(name, directory / filename))
    pdfmetrics.registerFontFamily("Vera", normal="Vera", bold="Vera-Bold", italic="Vera-Italic",
                                  boldItalic="Vera-BoldItalic")
    shutil.copyfile(directory / "bitstream-vera-license.txt", HERE / "LICENCIA-TIPOGRAFIA.txt")


def inline(node):
    if isinstance(node, NavigableString):
        return html.escape(str(node))
    content = "".join(inline(child) for child in node.children)
    if node.name in ("strong", "b"):
        return "<b>" + content + "</b>"
    if node.name in ("em", "i"):
        return "<i>" + content + "</i>"
    if node.name == "code":
        return '<font name="Vera" size="8.3" color="#087180">' + content + "</font>"
    if node.name == "a":
        return '<a href="' + html.escape(node.get("href", ""), quote=True) + '" color="#087180">' + content + "</a>"
    if node.name == "br":
        return "<br/>"
    return content


class ManualDoc(BaseDocTemplate):
    def __init__(self, path, title):
        super().__init__(str(path), pagesize=A4, leftMargin=MARGIN, rightMargin=MARGIN,
                         topMargin=53, bottomMargin=47, title=title, author="Craft Corel",
                         subject="Documentación en castellano; edición 2026-11-08")
        self.short_title = title.replace("Craft Corel · ", "")
        frame = Frame(MARGIN, 47, CONTENT_WIDTH, HEIGHT - 100, leftPadding=0,
                      rightPadding=0, topPadding=0, bottomPadding=0)
        self.addPageTemplates(PageTemplate(id="manual", frames=[frame], onPage=self.page_header))

    def page_header(self, canvas, doc):
        canvas.saveState()
        if doc.page == 1:
            canvas.setFillColor(NAVY)
            canvas.rect(0, HEIGHT - 20, WIDTH, 20, stroke=0, fill=1)
            canvas.setFillColor(TEAL)
            canvas.rect(MARGIN, HEIGHT - 90, 60, 5, stroke=0, fill=1)
        else:
            canvas.setFont("Vera", 8)
            canvas.setFillColor(GRAY)
            canvas.drawString(MARGIN, HEIGHT - 31, "CRAFT COREL")
            canvas.drawRightString(WIDTH - MARGIN, HEIGHT - 31, self.short_title)
            canvas.setStrokeColor(colors.HexColor("#d4e0e7"))
            canvas.line(MARGIN, HEIGHT - 38, WIDTH - MARGIN, HEIGHT - 38)
        canvas.setFont("Vera", 7.5)
        canvas.setFillColor(GRAY)
        canvas.drawString(MARGIN, 27, "Edición 08-11-2026 · Aplicación independiente")
        canvas.drawRightString(WIDTH - MARGIN, 27, str(doc.page))
        canvas.restoreState()

    def afterFlowable(self, flowable):
        if hasattr(flowable, "heading_key"):
            self.canv.bookmarkPage(flowable.heading_key)
            self.canv.addOutlineEntry(flowable.heading_text, flowable.heading_key, 0, False)
            self.notify("TOCEntry", (0, flowable.heading_text, self.page, flowable.heading_key))


def pdf(soup, path, title):
    styles = getSampleStyleSheet()
    base = ParagraphStyle("body", fontName="Vera", fontSize=9.4, leading=14.3,
                          textColor=NAVY, spaceAfter=8, splitLongWords=True, alignment=TA_LEFT)
    h2 = ParagraphStyle("chapter", parent=base, fontName="Vera-Bold", fontSize=16.3,
                        leading=21, spaceBefore=21, spaceAfter=12, keepWithNext=True)
    cover = ParagraphStyle("cover", parent=h2, fontSize=31, leading=38, spaceAfter=29)
    cell = ParagraphStyle("cell", parent=base, fontSize=8.05, leading=11.9, spaceAfter=0)
    bullet = ParagraphStyle("bullet", parent=base, leftIndent=14, firstLineIndent=-10, spaceAfter=6)
    code = ParagraphStyle("code", parent=base, fontName="Courier", fontSize=8, leading=11,
                          backColor=colors.HexColor("#eef5f6"), borderPadding=10, spaceBefore=6,
                          spaceAfter=14)
    story = []
    headings = []
    for node in soup.children:
        if isinstance(node, NavigableString):
            continue
        if node.name == "h1":
            story += [Spacer(1, 62), Paragraph(inline(node), cover)]
        elif node.name == "div" and "toc" in node.get("class", []):
            story += [PageBreak(), Paragraph("Índice", h2), Spacer(1, 9)]
            toc = TableOfContents()
            toc.levelStyles = [ParagraphStyle("toc", parent=base, fontSize=10,
                                              leading=16.5, spaceBefore=5)]
            story += [toc, PageBreak()]
        elif node.name in ("h2", "h3"):
            para = Paragraph(inline(node), h2)
            para.heading_key = node.get("id")
            para.heading_text = node.get_text(" ", strip=True)
            headings.append(para.heading_text)
            story.append(para)
        elif node.name == "p":
            paragraph = Paragraph(inline(node), base)
            next_node = node.find_next_sibling()
            if next_node is not None and next_node.name == "pre":
                paragraph.keepWithNext = True
            story.append(paragraph)
        elif node.name in ("ul", "ol"):
            for index, item in enumerate(node.find_all("li", recursive=False), 1):
                prefix = str(index) + ". " if node.name == "ol" else "• "
                story.append(Paragraph(prefix + inline(item), bullet))
            story.append(Spacer(1, 4))
        elif node.name == "pre":
            content = node.get_text().rstrip("\n")
            lines = []
            for line in content.splitlines():
                lines.extend(textwrap.wrap(line, width=91, replace_whitespace=False,
                                           drop_whitespace=False) or [""])
            story.append(KeepTogether([Preformatted("\n".join(lines), code)]))
        elif node.name == "table":
            rows = [[Paragraph(inline(c), cell) for c in row.find_all(["th", "td"], recursive=False)]
                    for row in node.find_all("tr")]
            columns = len(rows[0])
            ratios = {2: [0.32, 0.68], 3: [0.25, 0.35, 0.40]}.get(columns, [1 / columns] * columns)
            table = LongTable(rows, colWidths=[CONTENT_WIDTH * x for x in ratios],
                              repeatRows=1, hAlign="LEFT")
            table.setStyle(TableStyle([
                ("BACKGROUND", (0, 0), (-1, 0), colors.HexColor("#e6f2f4")),
                ("ROWBACKGROUNDS", (0, 1), (-1, -1), [colors.white, colors.HexColor("#f6f9fb")]),
                ("GRID", (0, 0), (-1, -1), 0.4, colors.HexColor("#d4e0e7")),
                ("VALIGN", (0, 0), (-1, -1), "TOP"),
                ("LEFTPADDING", (0, 0), (-1, -1), 8), ("RIGHTPADDING", (0, 0), (-1, -1), 8),
                ("TOPPADDING", (0, 0), (-1, -1), 6), ("BOTTOMPADDING", (0, 0), (-1, -1), 6),
            ]))
            story += [Spacer(1, 6), table, Spacer(1, 12)]
    ManualDoc(path, title).multiBuild(story)
    return headings


def build(name, evidence):
    source = HERE / (name + ".md")
    raw = source.read_text(encoding="utf-8")
    rendered = markdown.markdown(raw, extensions=["toc", "tables", "fenced_code"],
                                 extension_configs={"toc": {"toc_depth": "2-3"}})
    soup = BeautifulSoup(rendered, "html.parser")
    title = soup.h1.get_text(" ", strip=True)
    title_html = html.escape(title)
    sections = []
    current = []
    intro = []
    for child in soup.children:
        if getattr(child, "name", None) == "h2":
            if current:
                sections.append('<section>' + ''.join(current) + '</section>')
            current = [str(child)]
        elif current:
            current.append(str(child))
        else:
            intro.append(str(child))
    if current:
        sections.append('<section>' + ''.join(current) + '</section>')
    html_path = HERE / (name + ".html")
    html_path.write_text('<!doctype html><html lang="es"><head><meta charset="utf-8">'
                        '<meta name="viewport" content="width=device-width,initial-scale=1">'
                        '<title>' + title_html + '</title><style>' + CSS + '</style></head><body>'
                        '<header><p>CRAFT COREL · DOCUMENTACIÓN</p></header>'
                        '<div class="tools"><label for="search">Buscar en este manual</label>'
                        '<input id="search" type="search" placeholder="Texto, color, capas…" '
                        'aria-label="Buscar apartados"><span class="count" id="count" role="status" '
                        'aria-live="polite"></span></div><main>' + ''.join(intro) + ''.join(sections) +
                        '</main><footer>Documento autónomo. / abre la búsqueda; Esc la limpia. '
                        'El PDF procede de la misma fuente de contenido.</footer><script>' + SEARCH +
                        '</script></body></html>', encoding="utf-8")
    pdf_path = HERE / (name + ".pdf")
    headings = pdf(soup, pdf_path, title)
    document = fitz.open(pdf_path)
    text = "\n".join(page.get_text() for page in document)
    page_dir = evidence / "render" / name
    page_dir.mkdir(parents=True, exist_ok=True)
    renders = []
    bounds = []
    thumbnails = []
    for index, page in enumerate(document):
        image_path = page_dir / f"p{index + 1:02}.png"
        pix = page.get_pixmap(matrix=fitz.Matrix(1.2, 1.2), alpha=False)
        pix.save(image_path)
        renders.append({"page": index + 1, "path": str(image_path), "sha256": digest(image_path)})
        for block in page.get_text("blocks"):
            x0, y0, x1, y1 = block[:4]
            if x0 < 20 or y0 < 15 or x1 > page.rect.width - 15 or y1 > page.rect.height - 15:
                bounds.append({"page": index + 1, "bbox": block[:4], "text": str(block[4])[:100]})
        thumb = Image.open(image_path).convert("RGB")
        thumb.thumbnail((260, 368))
        card = Image.new("RGB", (280, 400), "#e9eff3")
        card.paste(thumb, ((280 - thumb.width) // 2, 8))
        ImageDraw.Draw(card).text((12, 379), f"{name} / {index + 1}", fill="#142e45")
        thumbnails.append(card)
    sheet = Image.new("RGB", (280 * 4, 400 * ((len(thumbnails) + 3) // 4)), "white")
    for index, thumb in enumerate(thumbnails):
        sheet.paste(thumb, ((index % 4) * 280, (index // 4) * 400))
    sheet_path = evidence / (name + "-contacto.png")
    sheet.save(sheet_path)
    (evidence / (name + "-texto.txt")).write_text(text, encoding="utf-8")
    html_check = BeautifulSoup(html_path.read_text(encoding="utf-8"), "html.parser")
    ids = {node["id"] for node in html_check.select("[id]")}
    broken = [node["href"] for node in html_check.select('a[href^="#"]') if node["href"][1:] not in ids]
    missing = [heading for heading in headings if heading not in re.sub(r"\s+", " ", text)]
    report = {
        "name": name, "title": title, "pages": len(document), "headings": headings,
        "source": {"path": str(source), "sha256": digest(source)},
        "html": {"path": str(html_path), "bytes": html_path.stat().st_size, "sha256": digest(html_path)},
        "pdf": {"path": str(pdf_path), "bytes": pdf_path.stat().st_size, "sha256": digest(pdf_path)},
        "broken_internal_links": broken, "missing_pdf_headings": missing,
        "out_of_page_text_blocks": bounds, "replacement_characters": text.count("\ufffd"),
        "pdf_outline_entries": len(document.get_toc()), "renders": renders,
        "contact_sheet": str(sheet_path), "rasterizer": "PyMuPDF/MuPDF", "visual_review_pending": True,
        "font": "Bitstream Vera, open license preserved in LICENCIA-TIPOGRAFIA.txt",
    }
    if broken or missing or bounds or report["replacement_characters"]:
        raise ValueError(json.dumps(report, ensure_ascii=False))
    document.close()
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--evidence", type=Path, required=True)
    args = parser.parse_args()
    evidence = args.evidence.resolve()
    if not evidence.is_relative_to(ROOT / "evidencias/craft-corel-nativo/publicacion/manuales"):
        raise ValueError("Evidence output must remain in Atlas publication-manual zone")
    evidence.mkdir(parents=True, exist_ok=True)
    fonts()
    reports = [build(name, evidence) for name in ("manual-usuario", "guia-tecnica")]
    report = {"generated_at_utc": datetime.now(timezone.utc).isoformat(), "manuals": reports,
              "generator_sha256": digest(Path(__file__)), "source_of_truth": "one Markdown per manual",
              "application_executed": False, "cargo_executed": False}
    (evidence / "validacion-generada.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps([{"name": r["name"], "pages": r["pages"], "pdf_bytes": r["pdf"]["bytes"],
                       "broken_links": len(r["broken_internal_links"]), "bounds_errors": len(r["out_of_page_text_blocks"])}
                      for r in reports]))


if __name__ == "__main__":
    main()
