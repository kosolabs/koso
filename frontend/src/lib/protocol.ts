import * as encoding from "lib0/encoding";
import * as Y from "yjs";

const MSG_SYNC = 0;
// const MSG_AWARENESS = 1;
// const MSG_AUTH = 2;
// const MSG_QUERY_AWARENESS = 3;

const MSG_SYNC_STEP_1 = 0;
// const MSG_SYNC_STEP_2 = 1;
// const MSG_SYNC_UPDATE = 2;

export function syncRequest(doc: Y.Doc): Uint8Array {
  const encoder = encoding.createEncoder();
  encoding.writeVarUint(encoder, MSG_SYNC);
  encoding.writeVarUint(encoder, MSG_SYNC_STEP_1);
  const sv = Y.encodeStateVector(doc);
  encoding.writeVarUint8Array(encoder, sv);
  return encoding.toUint8Array(encoder);
}
