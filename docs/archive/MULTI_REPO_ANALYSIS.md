# Multi-Repository Analysis

## Graph-Universe Organ Systems

The following systems implement the graph-universe thesis:

### knhk (Priority: 95%)
**Domain**: knowledge_graph

### mu-kernel (Priority: 90%)
**Domain**: timing_kernel

### chicago-tdd-tools (Priority: 85%)
**Domain**: verification

### ggen (Priority: 80%)
**Domain**: code_generation

### nomrg (Priority: 75%)
**Domain**: graph_overlay

## System Dependencies

| From | To | Type |
|---|----|------|
| knhk | ggen | integration |
| mu-kernel | chicago-tdd-tools | integration |
| knhk | nomrg | integration |
| clnrm | mu-kernel | integration |
| clnrm | chicago-tdd-tools | integration |
| ggen | clnrm | integration |
