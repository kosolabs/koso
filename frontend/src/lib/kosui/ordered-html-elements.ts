export class OrderedHTMLElements {
  #items: HTMLElement[] = [];

  get items(): HTMLElement[] {
    return this.#items;
  }

  getInsertionIndex(item: HTMLElement): number {
    if (this.#items.includes(item)) {
      throw new Error(`Ordered items already contains ${item}`);
    }
    for (let i = 0; i < this.#items.length; i++) {
      const position = item.compareDocumentPosition(this.#items[i]);
      if ((position & Node.DOCUMENT_POSITION_FOLLOWING) !== 0) {
        return i;
      }
    }
    return this.#items.length;
  }

  add(item: HTMLElement) {
    const index = this.getInsertionIndex(item);
    this.#items.splice(index, 0, item);
  }

  delete(item: HTMLElement) {
    const index = this.#items.indexOf(item);
    if (index === -1) {
      return;
    }
    this.items.splice(index, 1);
  }
}
