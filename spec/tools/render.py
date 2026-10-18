import re, sys, markdown
from markdown.extensions.toc import TocExtension
def gh(value, sep):
    v=value.strip().lower()
    v=re.sub(r'[^\w\- ]','',v)
    return v.replace(' ','-')
src=open(sys.argv[1]).read().replace('\\|','PIPEPH')
html=markdown.markdown(src, extensions=['tables','fenced_code','codehilite',TocExtension(slugify=gh)],
    extension_configs={'codehilite':{'guess_lang':False,'noclasses':True,'pygments_style':'friendly'}})
css='''
@page { size: A4; margin: 18mm 16mm; }
body { font-family: "DejaVu Sans", Arial, sans-serif; font-size: 10.5pt; line-height: 1.5; color:#222; }
h1 { font-size: 24pt; border-bottom: 2px solid #446; padding-bottom:4px;}
h2 { font-size: 16pt; color:#2a3a6a; border-bottom:1px solid #ccd; margin-top:28px; page-break-after: avoid;}
h3 { font-size: 12.5pt; color:#2a3a6a; page-break-after: avoid;} h4 { page-break-after: avoid;}
code { font-family: "DejaVu Sans Mono", monospace; font-size: 9pt; background:#f3f3f6; padding:0 2px; border-radius:3px;}
pre { background:#f6f7fa; border:1px solid #dde; border-radius:4px; padding:8px 10px; font-size:8.6pt; line-height:1.35; white-space:pre-wrap; page-break-inside: avoid;}
.codehilite { background:#f6f7fa !important; } pre code { background:none; padding:0;}
table { border-collapse: collapse; margin: 10px 0; font-size: 9pt; page-break-inside: avoid;}
th, td { border:1px solid #ccd; padding:4px 7px; vertical-align: top; text-align:left;}
th { background:#eef0f6; }
hr { border:none; border-top:1px solid #ddd; }
a { color:#2a4aa0; text-decoration:none; }
'''
html=html.replace('PIPEPH','|')
open(sys.argv[2],'w').write(f'<!doctype html><html><head><meta charset="utf-8"><title>The Verse Programming Language</title><style>{css}</style></head><body>{html}</body></html>')
