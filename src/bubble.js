class Bubble {
  constructor(el) {
    this.el = el;
    this.queue = [];
    this.displayTimer = null;
    this.displayMs = 4000;
    this.persistent = false;
  }

  show(text) {
    if (!text) {
      this.hide();
      return;
    }

    if (
      !this.el.classList.contains('hidden') &&
      !this.el.classList.contains('fade-out')
    ) {
      this.queue.push(text);
      return;
    }

    this._display(text);
  }

  _display(text) {
    this.el.textContent = text;
    this.el.classList.remove('hidden', 'fade-out');
    this.el.classList.add('visible');

    if (this.displayTimer) clearTimeout(this.displayTimer);

    if (!this.persistent) {
      this.displayTimer = setTimeout(() => {
        this.el.classList.add('fade-out');
        setTimeout(() => {
          this.el.classList.add('hidden');
          this.el.classList.remove('visible', 'fade-out');
          if (this.queue.length > 0) {
            const next = this.queue.shift();
            this._display(next);
          }
        }, 400);
      }, this.displayMs);
    }
  }

  setPersistent(on) {
    this.persistent = on;
    if (on && this.displayTimer) {
      clearTimeout(this.displayTimer);
      this.displayTimer = null;
    }
    if (!on) {
      // Restart the timer to auto-hide
      this.displayTimer = setTimeout(() => {
        this.el.classList.add('fade-out');
        setTimeout(() => {
          this.el.classList.add('hidden');
          this.el.classList.remove('visible', 'fade-out');
          if (this.queue.length > 0) {
            const next = this.queue.shift();
            this._display(next);
          }
        }, 400);
      }, this.displayMs);
    }
  }

  hide() {
    this.el.classList.add('hidden');
    this.el.classList.remove('visible', 'fade-out');
    this.queue = [];
    this.persistent = false;
    if (this.displayTimer) {
      clearTimeout(this.displayTimer);
      this.displayTimer = null;
    }
  }
}

window.Bubble = Bubble;
