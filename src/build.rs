use std::collections::{BinaryHeap, BTreeSet};
use std::cmp::Ordering;

use cgmath::{Point2, Vector2};
use fnv::{FnvHashMap, FnvHashSet};

use diagram::Diagram;

#[derive(Debug)]
pub struct Rect {
    position: Point2<f32>,
    size: Vector2<f32>,
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
        self.get_y().partial_cmp(&other.get_y())
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Event) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Less)
    }
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

    fn get_y(&self, k: f32, x: f32) -> f32 {
        1.0 / (2.0 * (self.origin.y - k)) * (x - self.origin.x).powi(2) + (self.origin.y + k) / 2.0
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

    pub fn find_arc(&self, x: f32) -> Option<ArcId> {
        None
    }
}

#[derive(Debug)]
struct DiagramBuilder {
    diagram: Diagram,
    event_queue: BinaryHeap<Event>,
    beachline: BeachLine,
    next_circle_event_id: u32,

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
            next_circle_event_id: 0,
            future_circle_events: FnvHashSet::default(),
        }
    }

    fn handle_site_event(&mut self, site: Point2<f32>) {
        // Find existing arc directly above this site
        let current_arc = self.beachline.find_arc(site.x);

        // Insert arc for this site
        let new_arc = self.beachline.add_arc(site, current_arc);

        // Cancel circle event for existing arc if it has one
        if let Some(current_arc) = current_arc {
            self.future_circle_events.remove(&current_arc);
        }
    }

    fn handle_circle_event(&mut self, y: f32, centroid: Point2<f32>, arc: ArcId) {

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
