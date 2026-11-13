"""Generate original, deterministic passive-format fixtures; never parse their content."""
from pathlib import Path
from zipfile import ZIP_STORED, ZipFile, ZipInfo
from io import BytesIO

ROOT = Path(__file__).parent


def archive(entries):
    output = BytesIO()
    with ZipFile(output, "w", compression=ZIP_STORED) as result:
        for name, data in entries:
            entry = ZipInfo(name, date_time=(2026, 1, 1, 0, 0, 0))
            entry.external_attr = 0o100600 << 16
            result.writestr(entry, data)
    return output.getvalue()


stream = b"BT /F1 12 Tf 20 100 Td (Original fixture) Tj ET\n"
objects = [
    b"<< /Type /Catalog /Pages 2 0 R >>",
    b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 200 200] /Resources << /Font << /F1 5 0 R >> >> /Contents 4 0 R >>",
    f"<< /Length {len(stream)} >>\nstream\n".encode() + stream + b"endstream",
    b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>",
]
pdf = bytearray(b"%PDF-1.4\n")
offsets = [0]
for number, content in enumerate(objects, 1):
    offsets.append(len(pdf))
    pdf.extend(f"{number} 0 obj\n".encode() + content + b"\nendobj\n")
xref = len(pdf)
pdf.extend(f"xref\n0 {len(objects) + 1}\n0000000000 65535 f \n".encode())
for offset in offsets[1:]:
    pdf.extend(f"{offset:010} 00000 n \n".encode())
pdf.extend(f"trailer\n<< /Size {len(objects) + 1} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n".encode())
(ROOT / "original.pdf").write_bytes(pdf)

container = b'''<?xml version="1.0"?><container xmlns="urn:oasis:names:tc:opendocument:xmlns:container" version="1.0"><rootfiles><rootfile full-path="EPUB/package.opf" media-type="application/oebps-package+xml"/></rootfiles></container>'''
package = b'''<?xml version="1.0"?><package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:identifier id="id">urn:uuid:00000000-0000-4000-8000-000000000001</dc:identifier><dc:title>Original fixture</dc:title><dc:language>en</dc:language><meta property="dcterms:modified">2026-01-01T00:00:00Z</meta></metadata><manifest><item id="page" href="page.xhtml" media-type="application/xhtml+xml"/><item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/></manifest><spine><itemref idref="page"/></spine></package>'''
page = b'''<?xml version="1.0"?><html xmlns="http://www.w3.org/1999/xhtml"><head><title>Original fixture</title></head><body><p>Original fixture.</p></body></html>'''
nav = b'''<?xml version="1.0"?><html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><head><title>Contents</title></head><body><nav epub:type="toc"><ol><li><a href="page.xhtml">Original fixture</a></li></ol></nav></body></html>'''
(ROOT / "original.epub").write_bytes(archive([
    ("mimetype", b"application/epub+zip"), ("META-INF/container.xml", container),
    ("EPUB/package.opf", package), ("EPUB/page.xhtml", page), ("EPUB/nav.xhtml", nav),
]))
(ROOT / "hostile.epub").write_bytes(archive([
    ("mimetype", b"application/epub+zip"),
    ("../outside.txt", b"MUST NEVER BE EXTRACTED"),
    ("META-INF/container.xml", b'''<!DOCTYPE container [<!ENTITY external SYSTEM "file:///not-a-real-private-fixture">]><container>&external;</container>'''),
]))
(ROOT / "original.docx").write_bytes(archive([
    ("[Content_Types].xml", b'''<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>'''),
    ("_rels/.rels", b'''<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>'''),
    ("word/document.xml", b'''<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Original fixture.</w:t></w:r></w:p></w:body></w:document>'''),
]))
(ROOT / "hostile.html").write_text('''<!doctype html><html><body><script>throw new Error("MUST NEVER EXECUTE")</script><img src="https://example.invalid/must-not-fetch"><p>Literal markup fixture.</p></body></html>\n''')
print("Generated original PDF/EPUB/DOCX and hostile EPUB/HTML fixtures")
