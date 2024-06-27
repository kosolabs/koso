export class Task {
  id: string;
  title: string;
  children: string[];

  constructor(id: string, title: string, children: string[]) {
    this.id = id;
    this.title = title;
    this.children = children;
  }

  hasChildren() {
    return this.children.length > 0;
  }
}
