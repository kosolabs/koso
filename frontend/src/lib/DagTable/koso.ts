import * as Y from "yjs";

function makeTask(id: string, name: string, children: string[]) {
  return new Y.Map<string | Y.Array<string>>([
    ["id", id],
    ["name", name],
    ["children", Y.Array.from(children)],
  ]);
}

export class Koso {
  yDoc: Y.Doc;
  yGraph: Y.Map<Y.Map<string | Y.Array<string>>>;

  constructor(yDoc: Y.Doc) {
    this.yDoc = yDoc;
    this.yGraph = yDoc.getMap("graph");
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  observe(f: (arg0: Array<Y.YEvent<any>>, arg1: Y.Transaction) => void) {
    this.yGraph.observeDeep(f);
  }

  onLocalUpdate(
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    f: (arg0: Uint8Array, arg1: any, arg2: Y.Doc, arg3: Y.Transaction) => void,
  ) {
    this.yDoc.on(
      "update",
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (update: Uint8Array, arg1: any, arg2: Y.Doc, txn: Y.Transaction) => {
        if (txn.local) {
          f(update, arg1, arg2, txn);
        }
      },
    );
  }

  update(data: Uint8Array) {
    Y.applyUpdate(this.yDoc, data);
  }

  toJSON() {
    return this.yGraph.toJSON();
  }

  newId(): string {
    let max = 0;
    for (const currId of this.yGraph.keys()) {
      const curr = parseInt(currId);
      if (curr > max) {
        max = curr;
      }
    }
    return `${max + 1}`;
  }

  addNode(nodeId: string, parentId: string, offset: number) {
    this.yDoc.transact(() => {
      const yParent = this.yGraph.get(parentId);
      if (!yParent) throw new Error(`Task ${parentId} is not in the graph`);
      const yChildren = yParent.get("children") as Y.Array<string>;
      yChildren.insert(offset, [nodeId]);
    });
  }

  removeNode(nodeId: string, parentId: string) {
    this.yDoc.transact(() => {
      const yParent = this.yGraph.get(parentId);
      if (!yParent) throw new Error(`Task ${parentId} is not in the graph`);
      const yChildren = yParent.get("children") as Y.Array<string>;
      yChildren.delete(yChildren.toArray().indexOf(nodeId));
    });
  }

  deleteNode(nodeId: string) {
    this.yDoc.transact(() => {
      this.yGraph.delete(nodeId);
    });
  }

  moveNode(
    nodeId: string,
    srcParentId: string,
    srcOffset: number,
    destParentId: string,
    destOffset: number,
  ) {
    this.yDoc.transact(() => {
      const ySrcParent = this.yGraph.get(srcParentId);
      if (!ySrcParent)
        throw new Error(`Task ${srcParentId} is not in the graph`);
      const ySrcChildren = ySrcParent.get("children") as Y.Array<string>;
      ySrcChildren.delete(srcOffset);

      const yDestParent = this.yGraph.get(destParentId);
      if (!yDestParent)
        throw new Error(`Task ${destParentId} is not in the graph`);
      const yDestChildren = yDestParent.get("children") as Y.Array<string>;
      if (srcParentId === destParentId && srcOffset < destOffset) {
        destOffset -= 1;
      }
      yDestChildren.insert(destOffset, [nodeId]);
    });
  }

  insertNode(parentId: string, offset: number) {
    this.yDoc.transact(() => {
      const nodeId = this.newId();
      this.yGraph.set(nodeId, makeTask(nodeId, "Untitled", []));
      const yParent = this.yGraph.get(parentId)!;
      const yChildren = yParent.get("children") as Y.Array<string>;
      yChildren.insert(offset, [nodeId]);
    });
  }

  editTaskName(taskId: string, newName: string) {
    this.yDoc.transact(() => {
      const yNode = this.yGraph.get(taskId);
      if (!yNode) throw new Error(`Task ${taskId} is not in the graph`);
      if (yNode.get("name") !== newName) {
        yNode.set("name", newName);
      }
    });
  }
}
