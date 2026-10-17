// Zamak Book：语法高亮、侧栏 logo 与中英切换。在 mdBook 的 book.js 之后加载。
(function () {
    "use strict";

    // Zamak 的语法高亮：注释用 #，字符串里的 {表达式} 是插值，@wrapping 是装饰器。
    if (window.hljs) {
        hljs.registerLanguage("zamak", function (hljs) {
            const interpolation = { className: "subst", begin: /\{(?!\{)/, end: /\}/ };
            return {
                name: "Zamak",
                keywords: {
                    keyword:
                        "fn pub use const let mut struct enum if elif else while for in not and or " +
                        "match return break continue pass fail catch assert extern self",
                    literal: "true false True False",
                    type: "i64 f64 bool string Option",
                    built_in: "println print len str int float abs min max sum sorted range copy",
                },
                contains: [
                    hljs.HASH_COMMENT_MODE,
                    {
                        className: "string",
                        begin: /f?"/,
                        end: /"/,
                        contains: [hljs.BACKSLASH_ESCAPE, { begin: /\{\{|\}\}/ }, interpolation],
                    },
                    { className: "string", begin: /'(\\.|[^'\\])'/ },
                    { className: "meta", begin: /@[a-z]+/ },
                    { className: "number", begin: /\b(0x[0-9a-fA-F_]+|0b[01_]+|0o[0-7_]+|\d[\d_]*(\.\d[\d_]*)?(e[+-]?\d+)?)\b/ },
                    { className: "title", begin: /(?<=\bfn\s+)[a-z_][a-zA-Z0-9_]*/ },
                    { className: "type", begin: /\b[A-Z][a-zA-Z0-9]*\b/ },
                ],
            };
        });
        document.querySelectorAll("code.language-zamak").forEach(function (block) {
            block.textContent = block.textContent;
            hljs.highlightBlock(block);
        });
    }

    // 侧栏顶部放 logo：深色主题用深色版。书的根目录从本脚本的地址推出（…/theme/zamak-<哈希>.js）。
    const script = document.currentScript;
    const root = script ? script.src.replace(/theme\/zamak[^/]*\.js.*$/, "") : "";
    const sidebar = document.querySelector("#mdbook-sidebar, #sidebar");
    if (sidebar && root) {
        const link = document.createElement("a");
        link.className = "zamak-logo";
        link.href = root + "index.html";
        link.innerHTML =
            '<img class="zamak-logo-light" src="' + root + 'assets/logo-light.svg" alt="Zamak">' +
            '<img class="zamak-logo-dark" src="' + root + 'assets/logo-dark.svg" alt="Zamak">';
        sidebar.insertBefore(link, sidebar.firstChild);
    }

    // 中英切换：两版的章节文件名相同，换掉路径前缀就是同一页。
    const buttons = document.querySelector(".right-buttons");
    if (buttons) {
        const chinese = document.documentElement.lang.toLowerCase().startsWith("zh");
        const path = location.pathname;
        const target = chinese ? path.replace(/^\/zh-CN(\/|$)/, "/") : "/zh-CN" + path;
        const switcher = document.createElement("a");
        switcher.className = "zamak-language";
        switcher.href = target;
        switcher.textContent = chinese ? "English" : "中文";
        switcher.title = chinese ? "Read this page in English" : "阅读本页的中文版";
        buttons.insertBefore(switcher, buttons.firstChild);
    }
})();
