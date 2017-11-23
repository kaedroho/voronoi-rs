use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::f32::INFINITY;

use cgmath::{Point2, Vector2, MetricSpace};
use fnv::{FnvHashMap, FnvHashSet};

use diagram::{Diagram, Vertex, HalfEdgeId};

#[derive(Debug)]
pub struct Rect {
    pub position: Point2<f32>,
    pub size: Vector2<f32>,
}

impl Rect {
    #[inline]
    pub fn left(&self) -> f32 {
        self.position.x
    }

    #[inline]
    pub fn top(&self) -> f32 {
        self.position.y
    }

    #[inline]
    pub fn right(&self) -> f32 {
        self.position.x + self.size.x
    }

    #[inline]
    pub fn bottom(&self) -> f32 {
        self.position.y + self.size.y
    }

    #[inline]
    pub fn contains_point(&self, point: Point2<f32>) -> bool {
        point.x > self.left() && point.y > self.top() && point.x < self.right() && point.y < self.bottom()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SiteId(u32);

#[derive(Debug, Clone, Copy)]
pub struct Site {
    pub position: Point2<f32>,
}

impl Site {
    pub fn new(position: Point2<f32>) -> Site {
        Site {
            position: position,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct CircleId(u32);

#[derive(Debug, Clone, Copy)]
struct Circle {
    pub position: Point2<f32>,
    pub radius: f32,
    pub arc: ArcId,
}

#[derive(Debug)]
enum Event {
    Site(f32, SiteId),
    Circle(f32, CircleId),
}

impl Event {
    fn get_y(&self) -> f32 {
        match *self {
            Event::Site(y, _) => y,
            Event::Circle(y, _) => y,
        }
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Event) -> bool {
        self.get_y().eq(&other.get_y())
    }
}

impl Eq for Event {}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Event) -> Option<Ordering> {
        let y = -self.get_y();
        let other_y = -other.get_y();
        y.partial_cmp(&other_y)
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Event) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Less)
    }
}

fn circumcircle_of_points(a: Point2<f32>, b: Point2<f32>, c: Point2<f32>) -> Option<(Point2<f32>, f32)> {
    // http://en.wikipedia.org/wiki/Circumscribed_circle#Cartesian_coordinates
    let d = 2.0 * (a.x * (b.y - c.y)
                + b.x * (c.y - a.y)
                + c.x * (a.y - b.y));

    if d == 0.0 {
        return None;
    }

    let axy2 = a.x * a.x + a.y * a.y;
    let bxy2 = b.x * b.x + b.y * b.y;
    let cxy2 = c.x * c.x + c.y * c.y;

    let x = axy2 * (b.y - c.y) + bxy2 * (c.y - a.y) + cxy2 * (a.y - b.y);
    let y = axy2 * (c.x - b.x) + bxy2 * (a.x - c.x) + cxy2 * (b.x - a.x);

    let centroid = Point2::new(x / d, y / d);
    let radius = a.distance(centroid);

    Some((centroid, radius))
}

fn intersection(left_focus: Point2<f32>, right_focus: Point2<f32>, directrix: f32) -> Point2<f32> {
    // Based on intersection function from https://www.cs.hmc.edu/~mbrubeck/voronoi.html
    let mut p = &left_focus;

    let x = if left_focus.y == right_focus.y {
        // Focii are at the same height so breakpoint is in the middle
        (left_focus.x + right_focus.x) / 2.0
    } else if right_focus.y == directrix {
        // Right focus is on the directrix
        right_focus.x
    } else if left_focus.y == directrix {
        // Left focus is on the directrix
        p = &right_focus;
        left_focus.x
    } else {
        // Use the quadratic formula
        let z_left = 2.0 * (left_focus.y - directrix);
        let z_right = 2.0 * (right_focus.y - directrix);

        let a = 1.0 / z_left - 1.0 / z_right;
        let b = -2.0 * (left_focus.x / z_left - right_focus.x / z_right);
        let c = (left_focus.x * left_focus.x + left_focus.y * left_focus.y - directrix * directrix) / z_left
                - (right_focus.x * right_focus.x + right_focus.y * right_focus.y - directrix * directrix) / z_right;

        (-b - (b * b - 4.0 * a * c).abs().sqrt()) / (2.0 * a)
    };

    // Plug back into one of the parabola equations
    let y = (p.y * p.y + (p.x - x) * (p.x - x) - directrix * directrix) / (2.0 * p.y - 2.0 * directrix);

    Point2::new(x, y)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct EdgeId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ArcId(u32);

#[derive(Debug)]
enum BeachLineNode {
    Inner(Box<BeachLineNode>, Box<BeachLineNode>, EdgeId),
    Leaf(ArcId, SiteId, Option<CircleId>),
}

/*
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ArcId(u32);

#[derive(Debug)]
struct Arc {
    pub site: Site,
    pub left: Option<ArcId>,
    pub right: Option<ArcId>,
}

impl Arc {
    fn from_site(site: Site) -> Arc {
        Arc {
            site: site,
            left: None,
            right: None,
        }
    }
}

#[derive(Debug, Default)]
struct BeachLine {
    next_arc_id: u32,
    arcs: FnvHashMap<ArcId, Arc>,
    arc_ordering: Vec<ArcId>,  // TODO: RBTree
}

impl BeachLine {
    fn new_arc_id(&mut self) -> ArcId {
        let arc_id = ArcId(self.next_arc_id);
        self.next_arc_id += 1;
        arc_id
    }

    pub fn add_arc(&mut self, site: Site, current_arc_id: Option<ArcId>) -> ArcId {
        // Create new arc
        let arc_id = self.new_arc_id();
        let mut arc = Arc::from_site(site);

        // If there's an existing arc, split it in two
        if let Some(current_arc_id) = current_arc_id {
            // Copy existing arc and place it to the right of the new one
            let right_arc_id = self.new_arc_id();
            let right_arc = {
                let current_arc = self.arcs.get(&current_arc_id).unwrap();

                let mut right_arc = Arc::from_site(current_arc.site);
                arc.right = Some(right_arc_id);
                right_arc.left = Some(arc_id);
                right_arc.right = current_arc.right;

                right_arc
            };
            self.arcs.insert(right_arc_id, right_arc);

            // Relink right neighbour's left arc to the new right arc
            if let Some(right_neighbour_arc_id) = self.arcs.get(&current_arc_id).unwrap().right {
                let mut right_neighbour_arc = self.arcs.get_mut(&right_neighbour_arc_id).unwrap();
                right_neighbour_arc.left = Some(right_arc_id);
            }

            // current_arc is now to the left of the new arc
            if let Some(mut current_arc) = self.arcs.get_mut(&current_arc_id) {
                current_arc.right = Some(arc_id);
                arc.left = Some(current_arc_id);
            }

            // Insert new arcs into arc_ordering
            if let Some(position) = self.arc_ordering.iter().position(|id| *id == current_arc_id) {
                self.arc_ordering.insert(position + 1, right_arc_id);
                self.arc_ordering.insert(position + 1, arc_id);
            }
        } else {
            self.arc_ordering.push(arc_id);
        }

        // Insert new arc
        self.arcs.insert(arc_id, arc);

        arc_id
    }

    pub fn get_left_right_arcs(&self, arc_id: ArcId) -> (Option<ArcId>, Option<ArcId>) {
        let arc = self.arcs.get(&arc_id).unwrap();
        (arc.left, arc.right)
    }

    pub fn get_left_breakpoint(&self, arc_id: ArcId, directrix: f32) -> f32 {
        let right_arc = self.arcs.get(&arc_id).unwrap();
        let left_arc = match right_arc.left {
            Some(left_arc_id) => self.arcs.get(&left_arc_id).unwrap(),
            None => return -INFINITY,
        };

        intersection(left_arc.site.position, right_arc.site.position, directrix).x
    }

    pub fn get_right_breakpoint(&self, arc_id: ArcId, directrix: f32) -> f32 {
        let left_arc = self.arcs.get(&arc_id).unwrap();
        let right_arc = match left_arc.right {
            Some(right_arc_id) => self.arcs.get(&right_arc_id).unwrap(),
            None => return INFINITY,
        };

        intersection(left_arc.site.position, right_arc.site.position, directrix).x
    }

    pub fn get_circumcircle(&self, middle_arc_id: ArcId) -> Option<(Point2<f32>, f32)> {
        let middle_arc = self.arcs.get(&middle_arc_id).unwrap();
        let (left_arc_id, right_arc_id) = match (middle_arc.left, middle_arc.right) {
            (Some(left_arc_id), Some(right_arc_id)) => (left_arc_id, right_arc_id),
            _ => return None
        };

        let left_arc = self.arcs.get(&left_arc_id).unwrap();
        let right_arc = self.arcs.get(&right_arc_id).unwrap();

        circumcircle_of_points(left_arc.site.position, middle_arc.site.position, right_arc.site.position)
    }

    pub fn find_arc(&self, x: f32, directrix: f32) -> Option<ArcId> {
        let mut current_arc = None;

        for arc_id in &self.arc_ordering {
            if self.get_left_breakpoint(*arc_id, directrix) > x {
                break;
            }

            current_arc = Some(*arc_id);
        }

        current_arc
    }

    pub fn remove_arc(&mut self, arc_id: ArcId) {
        // Link left and right arcs together
        let (left_arc_id, right_arc_id) = {
            let arc = self.arcs.get(&arc_id).unwrap();
            (arc.left, arc.right)
        };
        if let Some(left_arc_id) = left_arc_id {
            let mut left_arc = self.arcs.get_mut(&left_arc_id).unwrap();
            left_arc.right = right_arc_id;
        }
        if let Some(right_arc_id) = right_arc_id {
            let mut right_arc = self.arcs.get_mut(&right_arc_id).unwrap();
            right_arc.left = left_arc_id;
        }

        // Remove arc data
        self.arcs.remove(&arc_id);

        // Remove arc from ordering
        self.arc_ordering.retain(|a| *a != arc_id)
    }

    pub fn debug(&self, directrix: f32) {
        for arc_id in &self.arc_ordering {
            let xl = self.get_left_breakpoint(*arc_id, directrix);
            let xr = self.get_right_breakpoint(*arc_id, directrix);
            let arc = self.arcs.get(arc_id).unwrap();

            println!("arc {}: xl={}, xr={}, site={{id: {}, x:{}, y:{}}}", arc_id.0, xl, xr, arc.site.id, arc.site.position.x, arc.site.position.y);
        }
    }
}

*/

#[derive(Debug)]
pub struct DiagramBuilder {
    sites: Vec<Site>,
    circles: FnvHashMap<CircleId, Circle>,
    event_queue: BinaryHeap<Event>,
    beachline: Option<BeachLineNode>,
    step: u32,
    total_events: u32,
    cancelled_events: u32,
    debug: bool,
}

impl DiagramBuilder {
    pub fn new(bounding_rect: Rect, sites: Vec<Site>) -> DiagramBuilder {
        let mut event_queue = BinaryHeap::new();

        for (site_id, site) in sites.iter().enumerate() {
            if bounding_rect.contains_point(site.position) {
                event_queue.push(Event::Site(site.position.y, SiteId(site_id as u32)));
            }
        }

        DiagramBuilder {
            sites: sites,
            circles: FnvHashMap::default(),
            beachline: None,
            step: 0,
            total_events: 0,
            cancelled_events: 0,
            debug: false,
            event_queue: event_queue,
        }
    }

    pub fn set_debug(&mut self, enable: bool) {
        self.debug = enable;
    }

    fn handle_site_event(&mut self, site: Site) {
/*
        // Find existing arc directly above this site
        let current_arc = self.beachline.find_arc(site.position.x, site.position.y);

        // Insert arc for this site
        let new_arc = self.beachline.add_arc(site, current_arc);

        if let Some(current_arc) = current_arc {
            // Cancel existing circle event if one exists
            if self.future_circle_events.remove(&current_arc) {
                self.cancelled_events += 1;
            }
        }

        let (left_arc, right_arc) = self.beachline.get_left_right_arcs(new_arc);

        // Check for circle event on the left
        if let Some(left_arc) = left_arc {
            // Cancel existing circle event if one exists
            if self.future_circle_events.remove(&left_arc) {
                self.cancelled_events += 1;
            }

            if let Some((centroid, radius)) = self.beachline.get_circumcircle(left_arc) {
                if centroid.y + radius > site.position.y {
                    // Add to event_queue
                    self.event_queue.push(Event::Circle(centroid.y + radius, centroid, left_arc));

                    // Add to future_circle_events set
                    // This allows us to remove the event at any time before processing,
                    // which is difficult to do with just the event queue.
                    self.future_circle_events.insert(left_arc);
                    self.total_events += 1;
                }
            }
        }

        // Check for circle event on the right
        if let Some(right_arc) = right_arc {
            // Cancel existing circle event if one exists
            if self.future_circle_events.remove(&right_arc) {
                self.cancelled_events += 1;
            }

            if let Some((centroid, radius)) = self.beachline.get_circumcircle(right_arc) {
                if centroid.y + radius > site.position.y {
                    // Add to event_queue
                    self.event_queue.push(Event::Circle(centroid.y + radius, centroid, right_arc));

                    // Add to future_circle_events set
                    // This allows us to remove the event at any time before processing,
                    // which is difficult to do with just the event queue.
                    self.future_circle_events.insert(right_arc);
                    self.total_events += 1;
                }
            }
        }
*/
    }

    fn handle_circle_event(&mut self, circle: Circle) {
/*
        // Add vertex into diagram
        self.diagram.vertices.push(
            Vertex {
                coordinates: Point2::new(
                    centroid.x / self.scale.x + self.offset.x,
                    centroid.y / self.scale.y + self.offset.y,
                ),
                incident_edge: HalfEdgeId(0),
            }
        );

        // Remove the arc
        self.beachline.remove_arc(arc);
*/
    }

    fn debug_beachline(&self, directrix: f32) {
        //self.beachline.debug(directrix);
    }

    pub fn step(&mut self) -> bool {
        self.step += 1;

        if self.debug {
            println!("step {}", self.step);
        }

        let event = self.event_queue.pop();

        match event {
            Some(Event::Site(y, site_id)) => {
                let site = self.sites[site_id.0 as usize];
                self.handle_site_event(site);

                if self.debug {
                    println!("directrix={}", site.position.y);
                    println!("site event: id={} x={}, y={}", site_id.0, site.position.x, site.position.y);
                    self.debug_beachline(site.position.y);
                }
            }
            Some(Event::Circle(y, circle_id)) => {
                // Only run handle_circle_event if the ID is still in future_circle_events
                if let Some(circle) = self.circles.remove(&circle_id) {
                    self.handle_circle_event(circle);

                    if self.debug {
                        println!("directrix={}", y);
                        println!("circle event: id={}, cx={}, cy={}", circle_id.0, circle.position.x, circle.position.y);
                        self.debug_beachline(y);
                    }
                } else {
                    if self.debug {
                        println!("directrix={}", y);
                        println!("cancelled circle event (skipping)");
                    }
                }
            }
            None => return true,
        }

        if self.debug {
            println!("total_events={}, cancelled_events={}", self.total_events, self.cancelled_events);

            println!("end step");
            println!("");
        }

        false
    }

    pub fn finish(mut self) -> Diagram {
        while !self.step() {}
        Diagram::default()
    }
}
