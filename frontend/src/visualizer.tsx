import { useEffect, useState, useCallback } from "react";
import ReactFlow, { ReactFlowProvider, useNodesState, useEdgesState, Background, Controls, MiniMap, type Node, type Edge} from "reactflow";
import "reactflow/dist/style.css";
import type { JsonValue } from "./types";

interface Tree {
    id: string;
    tag: string;
    parent: string | null;
    children: string[];
    depth: number;
    attributes: {key: string; value: string}[];
}

interface TreeData{
    root_id: string;
    nodes: Tree[];
}

interface Visualizer {
    treeData: JsonValue;
    traversalPath: string[];
    matchNodeIds: string[];
}

// func: menentukan coords (x, y) tiap node dari Tree.
function calculatePos(nodes: Tree[]): Record<string, { x: number; y: number}> {
    const depthGroups: Record<number, string[]> = {};

    // loop tiap nodes untuk mengelompokan node based on depth 
    for (const node of nodes) {
        if (!depthGroups[node.depth]) {
            depthGroups[node.depth] = [];
        }

        depthGroups[node.depth].push(node.id);
    }

    const pos: Record<string, { x: number; y: number}> = {};

    // coords calcs
    for (const node of nodes) {
        const siblings = depthGroups[node.depth];
        const idx = siblings.indexOf(node.id);
        const total = siblings.length;

        pos[node.id] = {
            x: (idx - (total/2)) * 170, // divide 2 untuk center-align
            y: node.depth * 110,
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

function drawGraph ( treeData: TreeData, visitedSet: Set<string>, matchedSet: Set<string>): { rfNodes: Node[], rfEdges: Edge[] } {
    const rfNodes: Node[]= [];
    const rfEdges: Edge[]= [];

    const maxDepth = calculateMaxDepth(treeData.nodes);
    const nodesRender = filterByDepth(treeData.nodes, maxDepth)
    // const nodesRender = treeData.nodes.slice(0, 144);
    const renderedNode = new Set(nodesRender.map((n) => n.id));
    
    const pos = calculatePos(nodesRender);
    for (const node of nodesRender) {
        let background = "#f8fafc";
        let border = "#cbd5e1";
        let text = "#1e293b";

        if (matchedSet.has(node.id) && visitedSet.has(node.id)) { // green if sesuai input set
            background = "#22c55e";
            border = "#15803d";
            text = "#ffffff";
        } else if (visitedSet.has(node.id)) { // yellow if explored tapi ga sesuai input
            background = "#fbbf24";
            border = "#b45309";
            text = "#1e293b";
        }

        rfNodes.push({
            id: node.id,
            position: pos[node.id] ?? { x:0, y:0},
            data: { label: `<${node.tag}>` },
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

        if (node.parent != null && renderedNode.has(node.parent)) { // draw garis edge antara node ke parentnya
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


function Inner ({ treeData, traversalPath, matchNodeIds}: Visualizer) {
    const [nodes, setNodes, onNodesChange] = useNodesState([]);
    const [edges, setEdges, onEdgesChange] =  useEdgesState([]);
    const [visitedSet, setVisitedSet] = useState<Set<string>>(new Set());
    const [isAnimating, setIsAnimating] = useState(false);

    const parsedTree = treeData as unknown as TreeData;
    const matchedSet = new Set(matchNodeIds);

    useEffect(() => {
        if (!parsedTree?.nodes?.length) {
            return;
        }

        const {rfNodes, rfEdges} = drawGraph(parsedTree, visitedSet, matchedSet);
        setNodes(rfNodes);
        setEdges(rfEdges);
    }, [visitedSet, treeData]);

    const animation = useCallback(() => {
        setVisitedSet(new Set());
        setIsAnimating(true);

        const maxDepth = parsedTree?.nodes ? calculateMaxDepth(parsedTree.nodes) : 5;
        const renderedIds = new Set(
            (parsedTree?.nodes ?? []).filter((n) => n.depth <= maxDepth).map((n) => n.id)
        );
        const pathAnimation = traversalPath.filter((id) => renderedIds.has(id));
        // const pathAnimation = traversalPath.slice(0, 144);
        pathAnimation.forEach((nodeId, index) => {
            setTimeout(() => {
                setVisitedSet((prev) => new Set([...prev, nodeId]));

                if (index === (pathAnimation.length - 1)) {
                    setIsAnimating(false);
                }
            }, index * 125); // 125ms tiap node
        });
    }, [traversalPath]);

    useEffect(() => {
        if (traversalPath.length > 0 && parsedTree?.nodes?.length) {
            animation();
        }
    }, [traversalPath, treeData]);
    
    let nodeTotal = 0;
    if (parsedTree && parsedTree.nodes) {
        nodeTotal = parsedTree.nodes.length
    }

    let isTruncated = false;
    if (nodeTotal > 120) {
        isTruncated = true;
    }

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

            {isTruncated}
        </div>

        <div style={{ flex: 1, border: "1px solid #e2e8f0", borderRadius: "8px", overflow: "hidden" }}>
            <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={onNodesChange}
            onEdgesChange={onEdgesChange}
            fitView
            fitViewOptions={{ padding: 0.2 }}
            minZoom={0.05}
            maxZoom={2}
            proOptions={{ hideAttribution: true }}>

            <Background color="#e2e8f0" gap={20} />
            <Controls />
            <MiniMap
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

