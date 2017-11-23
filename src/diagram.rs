use cgmath::Point2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HalfEdgeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FaceId(u32);

#[derive(Debug, Clone)]
pub struct Vertex {
    pub coordinates: Point2<f32>,
    pub incident_edge: HalfEdgeId,
}

#[derive(Debug, Clone)]
pub struct HalfEdge {
    pub origin: VertexId,
    pub twin: HalfEdgeId,
    pub incident_face: FaceId,
    pub next: HalfEdgeId,
    pub prev: HalfEdgeId,
}

#[derive(Debug, Clone)]
pub struct Face {
    pub first_halfedge: HalfEdgeId,
}

#[derive(Debug, Default, Clone)]
pub struct Diagram {
    pub vertices: Vec<Vertex>,
    pub halfedges: Vec<HalfEdge>,
    pub faces: Vec<Face>,
}

impl Diagram {
    pub fn get_vertex(&self, vertex_id: VertexId) -> Option<&Vertex> {
        self.vertices.get(vertex_id.0 as usize)
    }

    pub fn get_half_edge(&self, halfedge_id: HalfEdgeId) -> Option<&HalfEdge> {
        self.halfedges.get(halfedge_id.0 as usize)
    }

    pub fn get_face(&self, face_id: FaceId) -> Option<&Face> {
        self.faces.get(face_id.0 as usize)
    }
}
