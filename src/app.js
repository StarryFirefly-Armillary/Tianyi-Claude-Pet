let animator;
let bubble;
let lastBubble = '';
let toolLockUntil = 0;
let idleStart = 0;
let idleAnimTimer = null;
let permissionPending = false;
let permissionTimer = null;
let currentScale = 0.5;

class CSSAnimator {
  constructor(el) {
    this.el = el;
    this.currentAnimation = null;
  }

  play(state) {
    if (state === this.currentAnimation) return;
    this.currentAnimation = state;
    this.el.dataset.anim = state;
  }

  stop() {
    this.currentAnimation = null;
    this.el.dataset.anim = '';
  }
}

function setScale(s) {
  if (s === currentScale) return;
  currentScale = s;
  document.documentElement.style.setProperty('--scale', s);
}

async function init() {
  const spriteEl = document.getElementById('sprite');
  const bubbleEl = document.getElementById('bubble');
  animator = new CSSAnimator(spriteEl);
  bubble = new Bubble(bubbleEl);
  setScale(0.333);
  animator.play('waving');
  bubble.show('天依来啦~');
  window._petBubble = bubble;
  window._petAnimator = animator;
  const ipc = window.__TAURI_INTERNALS__;
  document.addEventListener('mousedown', (e) => {
    if (e.button !== 0) return;
    try { if (ipc && ipc.invoke) ipc.invoke('start_drag'); } catch (_) {}
  });
  pollState();
}

function isToolBubble(text) {
  return text && (
    text.includes('正在读取') || text.includes('正在写') ||
    text.includes('正在执行') || text.includes('正在调度') ||
    text.includes('正在搜索') || text.includes('正在获取') ||
    text.includes('正在分析') || text.includes('正在构建') ||
    text.includes('正在组织') ||
    text.includes('工作中')
  );
}

async function pollState() {
  while (true) {
    try {
      const r = await fetch('http://127.0.0.1:9527/api/current');
      if (r.ok) {
        const data = await r.json();
        if (data.scale) {
          setScale(data.scale);
        }
        if (data.animation) {
          window._petAnimator.play(data.animation);
          if (permissionPending && data.animation !== 'waving' && data.animation !== 'idle') {
            exitPermission();
            window._petBubble.hide();
          }
          if (permissionPending && data.animation === 'idle') {
            exitPermission();
            window._petBubble.hide();
          }
          if (data.animation === 'idle') {
            if (!idleStart) idleStart = Date.now();
            scheduleIdleAnim();
          } else {
            idleStart = 0;
            cancelIdleAnim();
          }
        }
        if (data.bubble && data.bubble !== lastBubble) {
          const now = Date.now();
          if (data.bubble.includes('等待指示')) {
            if (!permissionPending) {
              permissionPending = true;
              window._petBubble.setPersistent(true);
              setTimeout(() => {
                if (permissionPending) { exitPermission(); }
              }, 120000);
            }
            lastBubble = data.bubble;
            window._petBubble.show(data.bubble);
          } else if (permissionPending) {
            if (isToolBubble(data.bubble)) {
              exitPermission();
              lastBubble = data.bubble;
              window._petBubble.show(data.bubble);
              toolLockUntil = now + 1200;
            }
          } else {
            if (isToolBubble(lastBubble) && now < toolLockUntil && !isToolBubble(data.bubble)) {
              // keep
            } else {
              lastBubble = data.bubble;
              window._petBubble.show(data.bubble);
              if (isToolBubble(data.bubble)) toolLockUntil = now + 1200;
            }
          }
        }
      }
    } catch (_) {}
    await new Promise(r => setTimeout(r, 400));
  }
}

function exitPermission() {
  permissionPending = false;
  if (permissionTimer) { clearInterval(permissionTimer); permissionTimer = null; }
  if (window._petBubble) {
    window._petBubble.setPersistent(false);
    window._petBubble.hide();
  }
}

function scheduleIdleAnim() {
  if (idleAnimTimer) return;
  idleAnimTimer = setTimeout(doIdleAnim, 15000 + Math.random() * 30000);
}
function doIdleAnim() {
  idleAnimTimer = null;
  if (!idleStart) return;
  const pick = ['jumping','waving','chatting'][Math.floor(Math.random()*3)];
  window._petAnimator.play(pick);
  setTimeout(() => {
    if (window._petAnimator && idleStart) {
      window._petAnimator.play('idle');
      scheduleIdleAnim();
    }
  }, 2000);
}
function cancelIdleAnim() {
  if (idleAnimTimer) { clearTimeout(idleAnimTimer); idleAnimTimer = null; }
}
document.addEventListener('DOMContentLoaded', init);
