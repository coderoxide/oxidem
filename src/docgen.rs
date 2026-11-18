use crate::ast::*;

pub struct DocGenerator;

impl DocGenerator {
    pub fn generate_markdown(prog: &Program, title: &str) -> String {
        let mut out = String::new();

        out.push_str(&format!("# Modül Dokümantasyonu: {}\n\n", title));
        out.push_str("> Bu dokümantasyon `oxide doc` tarafından otomatik üretilmiştir.\n\n");

        let mut functions = Vec::new();
        let mut structs = Vec::new();
        let mut traits = Vec::new();
        let mut enums = Vec::new();
        let mut consts = Vec::new();
        let mut impls = Vec::new();

        for item in &prog.items {
            match item {
                Item::Function(f) => functions.push(f),
                Item::Struct(s) => structs.push(s),
                Item::Trait(t) => traits.push(t),
                Item::Enum(e) => enums.push(e),
                Item::Const(c) => consts.push(c),
                Item::Impl(i) => impls.push(i),
                _ => {}
            }
        }

        // Summary Table of Contents
        out.push_str("## 📋 İçindekiler\n\n");
        if !consts.is_empty() {
            out.push_str("- [Sabitler (Constants)](#sabitler)\n");
        }
        if !functions.is_empty() {
            out.push_str("- [Fonksiyonlar (Functions)](#fonksiyonlar)\n");
        }
        if !structs.is_empty() {
            out.push_str("- [Yapılar (Structs)](#yapilar)\n");
        }
        if !enums.is_empty() {
            out.push_str("- [Enum'lar (Enums)](#enumlar)\n");
        }
        if !traits.is_empty() {
            out.push_str("- [Trait'ler (Traits)](#traitler)\n");
        }
        if !impls.is_empty() {
            out.push_str("- [Uygulamalar (Implementations)](#uygulamalar)\n");
        }
        out.push('\n');

        // Sabitler
        if !consts.is_empty() {
            out.push_str("---\n\n## <a id=\"sabitler\"></a>Sabitler\n\n");
            for c in consts {
                out.push_str(&format!("### `const {}: {}`\n\n", c.name, c.ty));
                if let Some(ref doc) = c.doc {
                    out.push_str(doc);
                    out.push_str("\n\n");
                }
            }
        }

        // Fonksiyonlar
        if !functions.is_empty() {
            out.push_str("---\n\n## <a id=\"fonksiyonlar\"></a>Fonksiyonlar\n\n");
            for f in functions {
                let params_str = f
                    .params
                    .iter()
                    .map(|p| {
                        let m = if p.is_mut { "mut " } else { "" };
                        format!("{}{}: {}", m, p.name, p.ty)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");

                out.push_str(&format!(
                    "### `fn {}({}) -> {}`\n\n",
                    f.name, params_str, f.return_type
                ));

                if let Some(ref doc) = f.doc {
                    out.push_str(doc);
                    out.push_str("\n\n");
                }
            }
        }

        // Struct'lar
        if !structs.is_empty() {
            out.push_str("---\n\n## <a id=\"yapilar\"></a>Yapılar (Structs)\n\n");
            for s in structs {
                out.push_str(&format!("### `struct {}`\n\n", s.name));
                if let Some(ref doc) = s.doc {
                    out.push_str(doc);
                    out.push_str("\n\n");
                }
                out.push_str("```rust\n");
                out.push_str(&format!("struct {} {{\n", s.name));
                for (fname, fty) in &s.fields {
                    out.push_str(&format!("    {}: {},\n", fname, fty));
                }
                out.push_str("}\n```\n\n");
            }
        }

        // Enum'lar
        if !enums.is_empty() {
            out.push_str("---\n\n## <a id=\"enumlar\"></a>Enum'lar\n\n");
            for e in enums {
                out.push_str(&format!("### `enum {}`\n\n", e.name));
                if let Some(ref doc) = e.doc {
                    out.push_str(doc);
                    out.push_str("\n\n");
                }
                out.push_str("Varyantlar:\n");
                for v in &e.variants {
                    out.push_str(&format!("* `{}`\n", v));
                }
                out.push('\n');
            }
        }

        // Trait'ler
        if !traits.is_empty() {
            out.push_str("---\n\n## <a id=\"traitler\"></a>Trait'ler\n\n");
            for t in traits {
                out.push_str(&format!("### `trait {}`\n\n", t.name));
                if let Some(ref doc) = t.doc {
                    out.push_str(doc);
                    out.push_str("\n\n");
                }
                out.push_str("Metotlar:\n\n");
                for m in &t.methods {
                    let p_str = m
                        .params
                        .iter()
                        .map(|p| {
                            let m_str = if p.is_mut { "mut " } else { "" };
                            format!("{}{}: {}", m_str, p.name, p.ty)
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    out.push_str(&format!("* `fn {}({}) -> {}`\n", m.name, p_str, m.return_type));
                    if let Some(ref m_doc) = m.doc {
                        out.push_str(&format!("  > {}\n", m_doc.replace('\n', "\n  > ")));
                    }
                }
                out.push('\n');
            }
        }

        // Impl blokları
        if !impls.is_empty() {
            out.push_str("---\n\n## <a id=\"uygulamalar\"></a>Uygulamalar (Implementations)\n\n");
            for imp in impls {
                let header = if let Some(ref tr) = imp.trait_name {
                    format!("impl {} for {}", tr, imp.target)
                } else {
                    format!("impl {}", imp.target)
                };
                out.push_str(&format!("### `{}`\n\n", header));
                for m in &imp.methods {
                    let p_str = m
                        .params
                        .iter()
                        .map(|p| {
                            let m_str = if p.is_mut { "mut " } else { "" };
                            format!("{}{}: {}", m_str, p.name, p.ty)
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
                    out.push_str(&format!("* `fn {}({}) -> {}`\n", m.name, p_str, m.return_type));
                    if let Some(ref m_doc) = m.doc {
                        out.push_str(&format!("  {}\n", m_doc.replace('\n', "\n  ")));
                    }
                }
                out.push('\n');
            }
        }

        out
    }

    pub fn generate_html(prog: &Program, title: &str) -> String {
        let md = Self::generate_markdown(prog, title);

        // Escape HTML special chars in text, and render simple markdown structure into HTML
        let body_html = Self::simple_markdown_to_html(&md);

        format!(
            r#"<!DOCTYPE html>
<html lang="tr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} - Oxide API Dokümantasyonu</title>
    <style>
        :root {{
            --bg-primary: #12151a;
            --bg-secondary: #1a1e24;
            --bg-card: #22272e;
            --text-primary: #e6edf3;
            --text-secondary: #8b949e;
            --accent: #58a6ff;
            --accent-green: #3fb950;
            --border: #30363d;
            --code-bg: #161b22;
        }}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            background-color: var(--bg-primary);
            color: var(--text-primary);
            line-height: 1.6;
            padding: 40px 20px;
        }}
        .container {{
            max-width: 960px;
            margin: 0 auto;
            background-color: var(--bg-secondary);
            border: 1px solid var(--border);
            border-radius: 8px;
            padding: 40px;
            box-shadow: 0 8px 24px rgba(0,0,0,0.5);
        }}
        h1 {{
            color: var(--accent);
            border-bottom: 2px solid var(--border);
            padding-bottom: 12px;
            margin-bottom: 24px;
        }}
        h2 {{
            color: var(--accent-green);
            border-bottom: 1px solid var(--border);
            padding-bottom: 8px;
            margin-top: 32px;
            margin-bottom: 16px;
        }}
        h3 {{
            color: var(--text-primary);
            margin-top: 20px;
            margin-bottom: 8px;
        }}
        a {{
            color: var(--accent);
            text-decoration: none;
        }}
        a:hover {{
            text-decoration: underline;
        }}
        code {{
            font-family: "JetBrains Mono", "Fira Code", Consolas, monospace;
            background-color: var(--code-bg);
            border: 1px solid var(--border);
            border-radius: 4px;
            padding: 2px 6px;
            font-size: 0.9em;
            color: #79c0ff;
        }}
        pre {{
            background-color: var(--code-bg);
            border: 1px solid var(--border);
            border-radius: 6px;
            padding: 16px;
            overflow-x: auto;
            margin: 16px 0;
        }}
        pre code {{
            background: none;
            border: none;
            padding: 0;
            color: var(--text-primary);
        }}
        blockquote {{
            border-left: 4px solid var(--accent);
            padding: 8px 16px;
            background-color: var(--bg-card);
            border-radius: 0 4px 4px 0;
            color: var(--text-secondary);
            margin: 16px 0;
        }}
        ul {{
            margin-left: 24px;
            margin-bottom: 16px;
        }}
        li {{
            margin-bottom: 6px;
        }}
        hr {{
            border: none;
            border-top: 1px solid var(--border);
            margin: 32px 0;
        }}
        .footer {{
            text-align: center;
            margin-top: 40px;
            color: var(--text-secondary);
            font-size: 0.85em;
        }}
    </style>
</head>
<body>
    <div class="container">
        {body_html}
        <div class="footer">
            Oxide Systems Programming Language &bull; Otomatik API Dokümantasyonu
        </div>
    </div>
</body>
</html>"#,
            title = title,
            body_html = body_html
        )
    }

    fn simple_markdown_to_html(md: &str) -> String {
        let mut html = String::new();
        let mut in_code_block = false;
        let mut in_list = false;

        for line in md.lines() {
            if line.starts_with("```") {
                if in_code_block {
                    html.push_str("</code></pre>\n");
                    in_code_block = false;
                } else {
                    if in_list {
                        html.push_str("</ul>\n");
                        in_list = false;
                    }
                    html.push_str("<pre><code>");
                    in_code_block = true;
                }
                continue;
            }

            if in_code_block {
                html.push_str(&Self::escape_html(line));
                html.push('\n');
                continue;
            }

            if line.starts_with("- ") || line.starts_with("* ") {
                if !in_list {
                    html.push_str("<ul>\n");
                    in_list = true;
                }
                let item_content = &line[2..];
                html.push_str(&format!("  <li>{}</li>\n", Self::format_inline(item_content)));
                continue;
            } else if in_list {
                html.push_str("</ul>\n");
                in_list = false;
            }

            if line.starts_with("# ") {
                html.push_str(&format!("<h1>{}</h1>\n", Self::format_inline(&line[2..])));
            } else if line.starts_with("## ") {
                html.push_str(&format!("<h2>{}</h2>\n", Self::format_inline(&line[3..])));
            } else if line.starts_with("### ") {
                html.push_str(&format!("<h3>{}</h3>\n", Self::format_inline(&line[4..])));
            } else if line.starts_with("> ") {
                html.push_str(&format!("<blockquote>{}</blockquote>\n", Self::format_inline(&line[2..])));
            } else if line == "---" {
                html.push_str("<hr>\n");
            } else if line.trim().is_empty() {
                // blank line
            } else {
                html.push_str(&format!("<p>{}</p>\n", Self::format_inline(line)));
            }
        }

        if in_list {
            html.push_str("</ul>\n");
        }
        if in_code_block {
            html.push_str("</code></pre>\n");
        }

        html
    }

    fn escape_html(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }

    fn format_inline(s: &str) -> String {
        // Convert `code` to <code>code</code>
        let mut res = String::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '`' {
                let mut code_content = String::new();
                let mut closed = false;
                while let Some(nc) = chars.next() {
                    if nc == '`' {
                        closed = true;
                        break;
                    }
                    code_content.push(nc);
                }
                if closed {
                    res.push_str(&format!("<code>{}</code>", Self::escape_html(&code_content)));
                } else {
                    res.push('`');
                    res.push_str(&Self::escape_html(&code_content));
                }
            } else if c == '[' {
                let mut text = String::new();
                let mut link = String::new();
                let mut closed_text = false;
                while let Some(nc) = chars.next() {
                    if nc == ']' {
                        closed_text = true;
                        break;
                    }
                    text.push(nc);
                }
                if closed_text && chars.peek() == Some(&'(') {
                    chars.next(); // skip '('
                    while let Some(nc) = chars.next() {
                        if nc == ')' {
                            break;
                        }
                        link.push(nc);
                    }
                    res.push_str(&format!("<a href=\"{}\">{}</a>", Self::escape_html(&link), Self::format_inline(&text)));
                } else {
                    res.push('[');
                    res.push_str(&Self::escape_html(&text));
                    if closed_text {
                        res.push(']');
                    }
                }
            } else {
                match c {
                    '<' => res.push_str("&lt;"),
                    '>' => res.push_str("&gt;"),
                    '&' => res.push_str("&amp;"),
                    _ => res.push(c),
                }
            }
        }
        res
    }
}
