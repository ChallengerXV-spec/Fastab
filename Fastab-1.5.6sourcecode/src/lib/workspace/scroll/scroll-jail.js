/**
 * 竖向滚动"看守器"。
 *
 * 目标行为（用户需求）：
 * - 滚动位置只由鼠标滚轮 / 滚动条拖拽 / 键盘翻页这类"用户主动操作"改变；
 * - 编辑器/浏览器为了"把插入光标露出来"而做的自动滚动，只允许很小幅度的跟随
 *   （minimal follow），不允许任何跳变（典型症状：Ctrl+Z、按回车后视口突然回到顶部）。
 *
 * 实现思路：维护一个 expected 值（"我们认为当前应该在这个位置"）。
 * - 用户主动滚动 → 直接接受并更新 expected；
 * - 非用户滚动且位移 ≤ maxAutoDelta → 视为最小跟随，接受；
 * - 非用户滚动且位移 > maxAutoDelta → 判定为跳变，立刻写回 expected。
 *
 * @param {HTMLElement | HTMLTextAreaElement} el
 * @param {{ maxAutoDelta?: number; userWindowMs?: number }} [options]
 */
export function createScrollJail(el, options = {}) {
  const maxAutoDelta = Math.max(0, Number(options.maxAutoDelta ?? 0));
  const userWindowMs = Math.max(0, Number(options.userWindowMs ?? 260));

  let expected = el.scrollTop;
  let userUntil = 0;
  let pointerDown = false;
  let destroyed = false;

  const markUser = () => {
    userUntil = performance.now() + userWindowMs;
  };

  const onWheel = () => markUser();
  const onKeydown = () => markUser();
  const onPointerDown = () => {
    pointerDown = true;
    markUser();
  };
  const onPointerUp = () => {
    pointerDown = false;
    markUser();
  };
  const onPointerCancel = () => {
    pointerDown = false;
  };

  const onScroll = () => {
    if (destroyed) return;
    const next = el.scrollTop;
    if (pointerDown || performance.now() < userUntil) {
      expected = next;
      return;
    }
    if (Math.abs(next - expected) <= maxAutoDelta) {
      expected = next;
      return;
    }
    // 跳变：回弹到上一次"被认可"的位置
    el.scrollTop = expected;
  };

  el.addEventListener("wheel", onWheel, { passive: true });
  el.addEventListener("keydown", onKeydown);
  el.addEventListener("pointerdown", onPointerDown);
  el.addEventListener("pointerup", onPointerUp);
  el.addEventListener("pointercancel", onPointerCancel);
  el.addEventListener("scroll", onScroll, { passive: true });

  return {
    /**
     * 由代码主动设置滚动位置（会被看守器认可，不会回弹）。
     * @param {number} top
     */
    set(top) {
      const value = Number.isFinite(top) ? top : 0;
      el.scrollTop = value;
      expected = el.scrollTop;
    },
    /** 把当前实际位置同步为基准（内容变化导致浏览器夹取后调用）。 */
    sync() {
      expected = el.scrollTop;
    },
    /** 当前基准位置。 */
    get top() {
      return expected;
    },
    destroy() {
      destroyed = true;
      el.removeEventListener("wheel", onWheel);
      el.removeEventListener("keydown", onKeydown);
      el.removeEventListener("pointerdown", onPointerDown);
      el.removeEventListener("pointerup", onPointerUp);
      el.removeEventListener("pointercancel", onPointerCancel);
      el.removeEventListener("scroll", onScroll);
    },
  };
}

/**
 * 渲染区（详情面板的 markdown 容器）使用：完全禁止非用户滚动。
 * @param {HTMLElement} el
 */
export function createStrictScrollJail(el) {
  return createScrollJail(el, { maxAutoDelta: 0 });
}

/**
 * 单行行高（px），用于推算"最小跟随"的容差。
 * @param {HTMLElement | HTMLTextAreaElement} el
 */
export function resolveLineHeight(el) {
  const raw = Number.parseFloat(getComputedStyle(el).lineHeight || "");
  if (Number.isFinite(raw) && raw > 0) return raw;
  const fontSize = Number.parseFloat(getComputedStyle(el).fontSize || "");
  return Number.isFinite(fontSize) && fontSize > 0 ? fontSize * 1.7 : 24;
}
