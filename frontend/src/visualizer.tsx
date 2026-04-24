import { useEffect, useState, useCallback, useMemo } from "react";
import ReactFlow, { ReactFlowProvider, useNodesState, useEdgesState, Background, Controls, MiniMap, type Node, type Edge} from "reactflow";
import "reactflow/dist/style.css";
import type { JsonValue } from "./types";

const MAX_ANIMATED_STEPS = 500;
const ANIMATION_BATCH_SIZE = 20;
const ANIMATION_FRAME_MS = 33;

interface Tree {
    id: string;
    tag: string;
    parent: string | null;
    children: string[];
    depth: number;
    attrs?: {key: string; value: string}[];
}

interface TreeData{
    root_id: string;
    nodes: Tree[];
}

interface Visualizer {
    treeData: JsonValue;
    traversalPath: string[];
    matchNodeIds: string[];
    maxDepth: number | null;
}

function calculatePos(nodes: Tree[]): Record<string, { x: number; y: number}> {
    const depthGroups: Record<number, string[]> = {};

    for (const node of nodes) {
        if (!depthGroups[node.depth]) {
            depthGroups[node.depth] = [];
        }

        depthGroups[node.depth].push(node.id);
    }

    const pos: Record<string, { x: number; y: number}> = {};
    const depthIndexMap: Record<number, Map<string, number>> = {};

    for (const [depthKey, ids] of Object.entries(depthGroups)) {
        const depth = Number.parseInt(depthKey, 10);
        const indexMap = new Map<string, number>();
        ids.forEach((id, idx) => indexMap.set(id, idx));
        depthIndexMap[depth] = indexMap;
    }

    // coords calcs
    for (const node of nodes) {
        const siblings = depthGroups[node.depth];
        const idx = depthIndexMap[node.depth]?.get(node.id) ?? 0;
        const total = siblings.length;
        // Keep dense levels readable by preserving a safer minimum horizontal gap.
        const xSpacing = Math.max(92, 210 - Math.log2(Math.max(total, 1)) * 14);

        pos[node.id] = {
            x: (idx - (total/2)) * xSpacing,
            y: node.depth * 126,
        };
    }

    return pos;
}

function filterByDepth(nodes: Tree[], maxDepth: number): Tree[] {
    return nodes.filter((n) => n.depth <= maxDepth)
}

function calculateMaxDepth(nodes: Tree[]): number {
    for (let depth = 1; depth <= 10; depth++) {
        const count = nodes.filter((n) => n.depth <= depth).length;

        if (count > 60) {
            return Math.max(depth - 1, 1);
        }
    }
    return 10;
}

function buildNodeTooltip(node: Tree): string {
    const attrs = node.attrs ?? [];
    const lines = [`id=${node.id}`, `tag=<${node.tag}>`];

    if (attrs.length === 0) {
        lines.push("atribut: (kosong)");
    } else {
        lines.push("atribut:");
        for (const attr of attrs) {
            lines.push(`${attr.key}=${attr.value}`);
        }
    }

    return lines.join("\n");
}

function resolveMaxDepthLimit(
    treeData: TreeData,
    traversalPath: string[],
    requestedMaxDepth: number | null,
): number {
    if (requestedMaxDepth == null) {
        return Number.MAX_SAFE_INTEGER;
    }

    const nodeById = new Map(treeData.nodes.map((node) => [node.id, node]));
    let deepestTraversalNode = 0;

    for (const nodeId of traversalPath) {
        const node = nodeById.get(nodeId);
        if (!node) {
            continue;
        }
        deepestTraversalNode = Math.max(deepestTraversalNode, node.depth);
    }

    return Math.max(requestedMaxDepth, deepestTraversalNode);
}

function drawGraph (
    treeData: TreeData,
    visitedSet: Set<string>,
    matchedSet: Set<string>,
    traversalPath: string[],
    requestedMaxDepth: number | null,
): { rfNodes: Node[], rfEdges: Edge[] } {
    const rfNodes: Node[]= [];
    const rfEdges: Edge[]= [];

    const maxDepth = resolveMaxDepthLimit(treeData, traversalPath, requestedMaxDepth);
    const nodesRender = filterByDepth(treeData.nodes, maxDepth)
    const renderedNode = new Set(nodesRender.map((n) => n.id));
    
    const pos = calculatePos(nodesRender);
    for (const node of nodesRender) {
        let background = "#f8fafc";
        let border = "#cbd5e1";
        let text = "#1e293b";

        if (matchedSet.has(node.id) && visitedSet.has(node.id)) {
            background = "#22c55e";
            border = "#15803d";
            text = "#ffffff";
        } else if (visitedSet.has(node.id)) {
            background = "#fbbf24";
            border = "#b45309";
            text = "#1e293b";
        }

        const tooltip = buildNodeTooltip(node);

        rfNodes.push({
            id: node.id,
            position: pos[node.id] ?? { x:0, y:0},
            data: {
                label: (
                    <span title={tooltip}>
                        {`<${node.tag}>`}
                    </span>
                ),
                tooltip,
            },
            style: {
                background: background,
                border: `2px solid ${border}`,
                color: text,
                borderRadius: "8px",
                padding: "5px 10px",
                fontSize: "11.33px",
                fontWeight: 600,
                transition: "background 0.25s, border-color 0.25s",
                minWidth: "60px",
                textAlign: "center",
            },
        });

        if (node.parent != null && renderedNode.has(node.parent)) {
            rfEdges.push({
                id: `e-${node.parent}-${node.id}`,
                source: node.parent,
                target: node.id,
                style: { stroke: "#94a3b8", strokeWidth: 1.5 },
                animated: false,
            })
        }
    }
    return { rfNodes, rfEdges };
}


function Inner ({ treeData, traversalPath, matchNodeIds, maxDepth}: Visualizer) {
    const [nodes, setNodes, onNodesChange] = useNodesState([]);
    const [edges, setEdges, onEdgesChange] =  useEdgesState([]);
    const [visitedSet, setVisitedSet] = useState<Set<string>>(new Set());
    const [isAnimating, setIsAnimating] = useState(false);
    const [selectedNodeInfo, setSelectedNodeInfo] = useState<string | null>(null);

    const parsedTree = treeData as unknown as TreeData;
    const matchedSet = useMemo(() => new Set(matchNodeIds), [matchNodeIds]);
    const depthLimit = useMemo(
        () => resolveMaxDepthLimit(parsedTree, traversalPath, maxDepth),
        [parsedTree, traversalPath, maxDepth],
    );

    useEffect(() => {
        if (!parsedTree?.nodes?.length) {
            return;
        }

        const {rfNodes, rfEdges} = drawGraph(parsedTree, visitedSet, matchedSet, traversalPath, maxDepth);
        setNodes(rfNodes);
        setEdges(rfEdges);
    }, [visitedSet, treeData, matchedSet, traversalPath, maxDepth, setNodes, setEdges]);

    const animation = useCallback(() => {
        setVisitedSet(new Set());

        const renderedIds = new Set(
            (parsedTree?.nodes ?? []).filter((n) => n.depth <= depthLimit).map((n) => n.id)
        );
        const pathAnimation = traversalPath.filter((id) => renderedIds.has(id));

        if (pathAnimation.length === 0) {
            setIsAnimating(false);
            return;
        }

        if (pathAnimation.length > MAX_ANIMATED_STEPS) {
            setVisitedSet(new Set(pathAnimation));
            setIsAnimating(false);
            return;
        }

        setIsAnimating(true);

        let index = 0;
        const timer = window.setInterval(() => {
            const chunk = pathAnimation.slice(index, index + ANIMATION_BATCH_SIZE);
            if (chunk.length === 0) {
                window.clearInterval(timer);
                setIsAnimating(false);
                return;
            }

            setVisitedSet((prev) => {
                const next = new Set(prev);
                chunk.forEach((id) => next.add(id));
                return next;
            });

            index += ANIMATION_BATCH_SIZE;
            if (index >= pathAnimation.length) {
                window.clearInterval(timer);
                setIsAnimating(false);
            }
        }, ANIMATION_FRAME_MS);
    }, [traversalPath, parsedTree, depthLimit]);

    useEffect(() => {
        if (traversalPath.length > 0 && parsedTree?.nodes?.length) {
            animation();
        }
    }, [traversalPath, treeData, animation]);

    const recommendedDepth = parsedTree?.nodes?.length ? calculateMaxDepth(parsedTree.nodes) : 0;

    return (
        <div style={{ display: "flex", flexDirection: "column", height: "100%", width: "100%" }}>
        <div
            style={{
            display: "flex",
            alignItems: "center",
            gap: "14px",
            padding: "8px 0",
            flexWrap: "wrap",
            }}
        >
            <button
            onClick={animation}
            disabled={isAnimating}
            style={{
                padding: "6px 16px",
                borderRadius: "6px",
                background: isAnimating ? "#94a3b8" : "#3b82f6",
                color: "#fff",
                border: "none",
                cursor: isAnimating ? "not-allowed" : "pointer",
                fontSize: "13px",
                fontWeight: 600,
            }}
            >
            {isAnimating ? "Menggambar animasi" : "Replay"}
            </button>

            <div style={{display: "flex", gap: "12px", fontSize: "11px", alignItems: "center"}}>
            {[
                {color: "#f8fafc", border:"#cbd5e1", label: "Not Explored" },
                {color: "#fbbf24", border:"#b45309", label: "Explored" },
                {color: "#22c55e", border:"#15803d", label: "Match" },
            ].map(({ color, border, label }) => (
                <span key={label} style={{ display: "flex", alignItems: "center", gap: "5px" }}>
                <span
                    style={{
                    width:14,
                    height:14,
                    background: color,
                    border: `2px solid ${border}`,
                    display: "inline-block",
                    borderRadius:3,
                    }}
                />
                {label}
                </span>
            ))}
            </div>

            <span style={{ fontSize: "12px", color: "#64748b" }}>
                {maxDepth == null
                    ? `Semua depth ditampilkan (rekomendasi ringkas: ${recommendedDepth})`
                    : `Depth aktif: ${depthLimit}`}
            </span>

            <span style={{ marginLeft: "auto", fontSize: "12px", color: "#64748b" }}>
                Klik node untuk lihat detail elemen
            </span>
        </div>

        <div style={{ flex: 1, border: "1px solid #e2e8f0", borderRadius: "8px", overflow: "hidden", position: "relative" }}>
            {selectedNodeInfo && (
                <div
                    style={{
                        position: "absolute",
                        top: 10,
                        right: 10,
                        zIndex: 15,
                        maxWidth: "340px",
                        fontSize: "12px",
                        lineHeight: 1.4,
                        color: "#0f172a",
                        background: "rgba(241, 245, 249, 0.96)",
                        border: "1px solid #cbd5e1",
                        borderRadius: "8px",
                        padding: "8px 10px",
                        whiteSpace: "pre-wrap",
                        boxShadow: "0 8px 18px -14px rgba(15, 23, 42, 0.45)",
                    }}
                >
                    {selectedNodeInfo}
                </div>
            )}
            <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            onNodeClick={(_, node) => {
                const detail = (node.data as { tooltip?: string } | undefined)?.tooltip;
                setSelectedNodeInfo(detail ?? null);
            }}
            onPaneClick={() => setSelectedNodeInfo(null)}
            onlyRenderVisibleElements
            nodesDraggable={false}
            nodesConnectable={false}
            elementsSelectable={false}
            fitView
            fitViewOptions={{ padding: 0.2 }}
            minZoom={0.05}
            maxZoom={2}
            proOptions={{ hideAttribution: true }}>

            <Background color="#e2e8f0" gap={20} />
            <Controls />
            <MiniMap
                pannable
                zoomable
                style={{ width: 150, height: 100 }}
                nodeColor={(node) => {
                const bg = (node.style?.background as string) ?? "#f8fafc";
                return bg;
                }}
            />
            </ReactFlow>
        </div>
        </div>
    );
    }

    export default function TreeVisualizer(props: Visualizer) {
    if (!props.treeData || !props.traversalPath.length) {
        return (
        <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%", color: "#94a3b8", fontSize: "14px",}}>
            Tidak ada data dari Tree
        </div>
        );
    }

    return (
        <ReactFlowProvider>
        <Inner {...props} />
        </ReactFlowProvider>
    );
}

