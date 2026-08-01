/**
 * Generate ≥100 golden Markdown samples for hybrid roundtrip tests (M4).
 */
import { writeFileSync, mkdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const dir = join(dirname(fileURLToPath(import.meta.url)), "../test/fixtures");
mkdirSync(dir, { recursive: true });

const samples = [];

// Basic structure
samples.push("# Title\n\nParagraph.\n");
samples.push("## Heading 2\n\nText with **bold** and *italic*.\n");
samples.push("### H3\n\n~~strike~~ and `code`.\n");
samples.push("#### H4\n\nA [link](https://example.com).\n");
samples.push("##### H5\n\n![alt](./img.png)\n");
samples.push("###### H6\n\n> blockquote\n");
samples.push("Horizontal rule:\n\n---\n");
samples.push("Hard break  \nnext line\n");

// Lists
samples.push("- a\n- b\n- c\n");
samples.push("1. one\n2. two\n3. three\n");
samples.push("- [ ] todo\n- [x] done\n");
samples.push("- nested\n  - child\n  - child2\n");
samples.push("1. parent\n   - mix\n   - list\n");

// Code
samples.push("```\nplain\n```\n");
samples.push("```ts\nconst x = 1;\n```\n");
samples.push("```python\nprint('hi')\n```\n");
samples.push("```json\n{\"a\":1}\n```\n");
samples.push("```bash\necho hi\n```\n");
samples.push("```rust\nfn main() {}\n```\n");
samples.push("```mermaid\ngraph LR\n  A-->B\n```\n");

// Tables
samples.push("| a | b |\n| --- | --- |\n| 1 | 2 |\n");
samples.push("| Name | Age |\n| --- | ---: |\n| Alice | 30 |\n| Bob | 25 |\n");

// Math-ish (kept as text/code for hybrid)
samples.push("Inline math $E=mc^2$ stays text.\n");
samples.push("```math\n\\int_a^b f(x) dx\n```\n");

// Front matter
samples.push("---\ntitle: Demo\n---\n\n# Body\n");
samples.push("---\ntags: [a, b]\ndraft: true\n---\n\nPara.\n");

// Unicode / CJK
samples.push("# 中文标题\n\n这是一段**中文**内容。\n");
samples.push("Emoji ✅ and 🌍 work.\n");
samples.push("日本語のテストです。\n");
samples.push("한국어 테스트.\n");

// Mixed docs
for (let i = 0; i < 20; i++) {
  samples.push(
    `# Doc ${i}\n\nIntro paragraph ${i}.\n\n## Section\n\n- item ${i}\n- item ${i + 1}\n\n` +
      `\`\`\`ts\nexport const n = ${i};\n\`\`\`\n\n` +
      `> quote ${i}\n`,
  );
}

// Edge cases that should still roundtrip under GFM
samples.push("Multiple\n\n\n\nblank lines become two.\n");
samples.push("Trailing spaces should be trimmed    \n");
samples.push("A paragraph with an autolink https://example.org/path.\n");
samples.push("Footnote-like text[^1] is plain text in hybrid.\n");
samples.push("~~a~~ **b** *c* `d`\n");
samples.push("> nested\n>\n> > deeper\n");
samples.push("- a\n\n  paragraph in list\n");
samples.push("1. a\n\n2. b\n");
samples.push("| x |\n| - |\n| y |\n");
samples.push("```html\n<div>ok</div>\n```\n");
samples.push("```css\nbody { color: red; }\n```\n");
samples.push("```sql\nSELECT 1;\n```\n");
samples.push("```go\npackage main\n```\n");
samples.push("```java\nclass A {}\n```\n");
samples.push("```c\nint main(){return 0;}\n```\n");
samples.push("```yaml\na: 1\n```\n");
samples.push("```toml\na = 1\n```\n");
samples.push("```xml\n<root/>\n```\n");
samples.push("```sh\nls -la\n```\n");
samples.push("```powershell\nGet-ChildItem\n```\n");
samples.push("```text\nhello\n```\n");
samples.push("Empty paragraph after heading:\n\n# H\n\n");
samples.push("Only heading\n\n# Solo\n");
samples.push("Only list\n\n- only\n");
samples.push("Only code\n\n```\nx\n```\n");
samples.push("Link ref style is plain: [x][y]\n");
samples.push("Image ref ![a][b]\n");
samples.push("Strong emphasis ***both***\n");
samples.push("Escape \\* not strong\n");
samples.push("Angle brackets as text: a < b > c\n");
samples.push("Pipe in text not table | alone |\n");
samples.push("Long line: " + "word ".repeat(40) + "\n");
samples.push("Chinese list\n\n- 项目一\n- 项目二\n");
samples.push("Task mix\n\n- [x] a\n- [ ] b\n- c\n");
samples.push("Ordered start\n\n1. a\n2. b\n");
samples.push("Nested task\n\n- [ ] a\n  - [x] b\n");
samples.push("Table align\n\n| L | C | R |\n| --- | :---: | ---: |\n| a | b | c |\n");
samples.push("Multi para\n\nOne.\n\nTwo.\n\nThree.\n");
samples.push("Code then list\n\n```\nx\n```\n\n- y\n");
samples.push("List then code\n\n- y\n\n```\nx\n```\n");
samples.push("Quote with code\n\n> ```\n> x\n> ```\n");
samples.push("HR variants\n\n***\n\nText\n");
samples.push("Emphasis _underscore_\n");
samples.push("Double __strong__\n");
samples.push("Inline code with spaces ` a b `\n");
samples.push("URL <https://example.com>\n");
samples.push("Email-like text user@example.com\n");
samples.push("Win path C:\\\\Users\\\\Docs\\\\a.md as text\n");
samples.push("Tabs\tare\tkept in code:\n\n```\na\tb\n```\n");
samples.push("Empty\n");
samples.push("Single word\n");
samples.push("#\n\nEmpty heading text edge\n");
samples.push("1. only one item\n");
samples.push("- only one item\n");
samples.push("> only quote\n");
samples.push("---\n");
samples.push("```\n\n```\n");
samples.push("Final fixture with all basics\n\n# T\n\n**b** *i* `c`\n\n- l\n\n1. n\n\n> q\n\n```js\n1\n```\n\n| a |\n| - |\n| b |\n");

// Ensure ≥ 100
while (samples.length < 110) {
  samples.push(`# Generated ${samples.length}\n\nParagraph ${samples.length} with **bold**.\n\n- x\n- y\n`);
}

const manifest = [];
samples.forEach((content, i) => {
  const name = `g${String(i + 1).padStart(3, "0")}.md`;
  writeFileSync(join(dir, name), content, "utf8");
  manifest.push(name);
});
writeFileSync(join(dir, "manifest.json"), JSON.stringify(manifest, null, 2));
console.log(`Wrote ${manifest.length} fixtures to ${dir}`);
