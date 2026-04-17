// tipe primitif yang diperbolehkan dalam payload JSON
export type JsonPrimitive = string | number | boolean | null;

// JSON value rekursif yang digunakan untuk API payload dengan bentuk objek/array yang dinamis
export type JsonValue =
  | JsonPrimitive
  | { [key: string]: JsonValue }
  | JsonValue[];

// node yang cocok dengan selector query
export interface MatchedNode {
  // Node identifier dari hasil traversal
  id: string;
  // HTML tag name dari node yang cocok dengan selector query
  tag: string;
  // nilai atribut kelas dari matched node
  class: string;
}

// Response dari traversal endpoint (BFS/DFS)
export interface TraversalResponse {
  // Total waktu eksekusi traversal
  execution_time_us: number;
  // List node yang sesuai dengan criteria selector query
  matched_nodes: MatchedNode[];
  // Path kunjungan terurut dari ID node selama traversal
  traversal_path: string[];
  // Payload tree DOM yang diparsing dan dikembalikan oleh backend sebagai JSON
  tree_data: JsonValue;
}