use std::collections::BinaryHeap;
use std::cmp::Ordering;
use std::f32::INFINITY;

use cgmath::{Point2, Vector2, MetricSpace};
use fnv::{FnvHashMap, FnvHashSet};

use diagram::Diagram;

#[derive(Debug)]
pub struct Rect {
    pub position: Point2<f32>,
    pub size: Vector2<f32>,
}

#[derive(Debug)]
enum Event {
    Site(Point2<f32>),
    Circle(f32, Point2<f32>, ArcId),
}

impl Event {
    fn get_y(&self) -> f32 {
        match *self {
            Event::Site(p) => p.y,
            Event::Circle(y, ..) => y,
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
        let z0 = 2.0 * (left_focus.y - directrix);
        let z1 = 2.0 * (right_focus.y - directrix);

        let a = 1.0 / z0 - 1.0 / z1;
        let b = -2.0 * (left_focus.x / z0 - right_focus.x / z0);
        let c = (left_focus.x * left_focus.x + left_focus.y * left_focus.y - directrix * directrix) / z0
                - (right_focus.x * right_focus.x + right_focus.y * right_focus.y - directrix * directrix) / z1;

        (-b - (b * b - 4.0 * a * c).sqrt()) / (2.0 * a)
    };

    // Plug back into one of the parabola equations.
    let y = (p.y * p.y + (p.x - x) * (p.x - x) - directrix * directrix) / (2.0 * p.y - 2.0 * directrix);

    Point2::new(x, y)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ArcId(u32);

#[derive(Debug)]
struct ArcData {
    pub origin: Point2<f32>,
    pub left: Option<ArcId>,
    pub right: Option<ArcId>,
}

impl ArcData {
    fn from_point(p: Point2<f32>) -> ArcData {
        ArcData {
            origin: p,
            left: None,
            right: None,
        }
    }

    fn get_y(&self, directrix: f32, x: f32) -> f32 {
        1.0 / (2.0 * (self.origin.y - directrix)) * (x - self.origin.x).powi(2) + (self.origin.y + directrix) / 2.0
    }
}

#[derive(Debug, Default)]
struct BeachLine {
    next_arc_id: u32,
    arcs: FnvHashMap<ArcId, ArcData>,
    arc_ordering: Vec<ArcId>,  // TODO: RBTree
}

impl BeachLine {
    fn new_arc_id(&mut self) -> ArcId {
        let arc_id = ArcId(self.next_arc_id);
        self.next_arc_id += 1;
        arc_id
    }

    pub fn add_arc(&mut self, site: Point2<f32>, current_arc_id: Option<ArcId>) -> ArcId {
        // Create new arc
        let arc_id = self.new_arc_id();
        let mut arc = ArcData::from_point(site);

        // If there's an existing arc, split it in two
        if let Some(current_arc_id) = current_arc_id {
            // Copy existing arc and place it to the right of the new one
            let right_arc_id = self.new_arc_id();
            let right_arc = {
                let current_arc = self.arcs.get(&current_arc_id).unwrap();

                let mut right_arc = ArcData::from_point(current_arc.origin);
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

        intersection(left_arc.origin, right_arc.origin, directrix).x
    }

    pub fn get_right_breakpoint(&self, arc_id: ArcId, directrix: f32) -> f32 {
        let left_arc = self.arcs.get(&arc_id).unwrap();
        let right_arc = match left_arc.right {
            Some(right_arc_id) => self.arcs.get(&right_arc_id).unwrap(),
            None => return -INFINITY,
        };

        intersection(left_arc.origin, right_arc.origin, directrix).x
    }

    pub fn get_circumcircle(&self, middle_arc_id: ArcId) -> Option<(Point2<f32>, f32)> {
        let mut middle_arc = self.arcs.get(&middle_arc_id).unwrap();
        let (left_arc_id, right_arc_id) = match (middle_arc.left, middle_arc.right) {
            (Some(left_arc_id), Some(right_arc_id)) => (left_arc_id, right_arc_id),
            _ => return None
        };

        let left_arc = self.arcs.get(&left_arc_id).unwrap();
        let right_arc = self.arcs.get(&right_arc_id).unwrap();

        circumcircle_of_points(left_arc.origin, middle_arc.origin, right_arc.origin)
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
}

#[derive(Debug)]
pub struct DiagramBuilder {
    diagram: Diagram,
    event_queue: BinaryHeap<Event>,
    beachline: BeachLine,

    /// Keeps track of valid future circle events
    future_circle_events: FnvHashSet<ArcId>,
}

impl DiagramBuilder {
    pub fn new(bounding_rect: Rect, sites: Vec<Point2<f32>>) -> DiagramBuilder {
        let mut event_queue = BinaryHeap::new();

        for site in sites {
            let position = Point2::new(
                (site.x - bounding_rect.position.x) / bounding_rect.size.x,
                (site.y - bounding_rect.position.y) / bounding_rect.size.y
            );

            if position.x > 0.0 && position.y > 0.0 && position.x < 1.0 && position.y < 1.0 {
                event_queue.push(Event::Site(position));
            }
        }

        DiagramBuilder {
            diagram: Diagram::default(),
            beachline: BeachLine::default(),
            event_queue: event_queue,
            future_circle_events: FnvHashSet::default(),
        }
    }

    fn handle_site_event(&mut self, site: Point2<f32>) {
        // Find existing arc directly above this site
        let current_arc = self.beachline.find_arc(site.x, site.y);

        // Insert arc for this site
        let new_arc = self.beachline.add_arc(site, current_arc);

        if let Some(current_arc) = current_arc {
            // Cancel existing circle event if one exists
            self.future_circle_events.remove(&current_arc);
        }

        let (left_arc, right_arc) = self.beachline.get_left_right_arcs(new_arc);

        // Check for circle event on the left
        if let Some(left_arc) = left_arc {
            if let Some((centroid, radius)) = self.beachline.get_circumcircle(left_arc) {
                if centroid.y + radius > site.y {
                    // Add to event_queue
                    self.event_queue.push(Event::Circle(centroid.y + radius, centroid, left_arc));

                    // Add to future_circle_events set
                    // This allows us to remove the event at any time before processing,
                    // which is difficult to do with just the event queue.
                    self.future_circle_events.insert(left_arc);
                }
            }
        }

        // Check for circle event on the right
        if let Some(right_arc) = right_arc {
            if let Some((centroid, radius)) = self.beachline.get_circumcircle(right_arc) {
                if centroid.y + radius > site.y {
                    // Add to event_queue
                    self.event_queue.push(Event::Circle(centroid.y + radius, centroid, right_arc));

                    // Add to future_circle_events set
                    // This allows us to remove the event at any time before processing,
                    // which is difficult to do with just the event queue.
                    self.future_circle_events.insert(right_arc);
                }
            }
        }
    }

    fn handle_circle_event(&mut self, y: f32, centroid: Point2<f32>, arc: ArcId) {
        // Remove the arc
        self.beachline.remove_arc(arc);
    }

    pub fn step(&mut self) -> bool {
        let event = self.event_queue.pop();

        match event {
            Some(Event::Site(p)) => {
                self.handle_site_event(p);
            }
            Some(Event::Circle(y, centroid, id)) => {
                // Only run handle_circle_event if the ID is still in future_circle_events
                if self.future_circle_events.remove(&id) {
                    self.handle_circle_event(y, centroid, id);
                }
            }
            None => return true,
        }

        false
    }

    pub fn finish(mut self) -> Diagram {
        while !self.step() {}
        self.diagram
    }
}
