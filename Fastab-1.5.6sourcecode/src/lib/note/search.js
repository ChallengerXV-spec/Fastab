/**
 * 笔记搜索（1.5.1 起只做字符匹配，不再支持拼音）。
 *
 * 匹配源 = 笔记标题 + 正文，规则：大小写不敏感的子串匹配。
 *
 * 归一化规则（1.5.0 引入，保留）：
 * - NFKD 会把全角字母/数字（ＡＢＣ、３４）转成半角，把重音字母（é、ǎ）分解成基字母 + 组合符；
 *   旧实现只保留 `a-z0-9` 与中文，全角输入会被整段清空，导致查询串变空、
 *   `matchNote` 返回“全部匹配”，表现就是“输入英文字符完全筛不动”。
 * - 保留的字符集合为“所有语言的字母与数字”（\p{L} / \p{N}）。
 */
const SPLIT_RE = /[^\p{L}\p{N}]+/gu;
const DIACRITICS_RE = /[\u0300-\u036f]/g;

/** @param {string} s */
function normalize(s) {
  return String(s ?? '')
    .normalize('NFKD')
    .replace(DIACRITICS_RE, '')
    .toLowerCase()
    .replace(SPLIT_RE, '');
}

/**
 * Match query against note text (title + body). Case-insensitive substring only.
 * @param {string} query - Search query
 * @param {string} text - Note text
 * @returns {{ matched: boolean, score: number }}
 */
export function matchNote(query, text) {
  const rawQuery = String(query ?? '').trim();
  if (!rawQuery) return { matched: true, score: 100 };

  const q = normalize(rawQuery);
  const rawText = String(text ?? '');
  const rawTextLower = rawText.toLowerCase();

  // 归一化后为空（例如只输入了标点）：退化为原文子串比较，
  // 绝不能返回“匹配一切”，否则搜索框看起来像坏了。
  if (!q) {
    return rawTextLower.includes(rawQuery.toLowerCase())
      ? { matched: true, score: 70 }
      : { matched: false, score: 0 };
  }

  const normText = normalize(rawText);
  if (normText === q) return { matched: true, score: 100 };
  if (normText.startsWith(q)) return { matched: true, score: 90 };
  if (normText.includes(q)) return { matched: true, score: 75 };
  // 原文子串（保留空格/重音等被归一化丢掉的形态）
  if (rawTextLower.includes(rawQuery.toLowerCase())) return { matched: true, score: 72 };

  return { matched: false, score: 0 };
}
