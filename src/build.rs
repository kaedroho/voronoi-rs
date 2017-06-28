use std::collections::{BinaryHeap, BTreeSet};
use std::cmp::Ordering;

use cgmath::{Point2, Vector2};

use diagram::Diagram;

#[derive(Debug)]
pub struct Rect {
    position: Point2<f32>,
    size: Vector2<f32>,
}

#[derive(Debug)]
enum Event {
    Site(Point2<f32>),
    Circle(f32),
}

impl Event {
    fn get_y(&self) -> f32 {
        match *self {
            Event::Site(p) => p.y,
            Event::Circle(y) => y,
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

#[derive(Debug)]
struct Arc {
    origin: Point2<f32>,
}

impl Arc {
    fn get_x(&self) -> f32 {
        self.origin.x
    }
}

impl PartialEq for Arc {
    fn eq(&self, other: &Arc) -> bool {
        self.get_x().eq(&other.get_x())
    }
}

impl Eq for Arc {}

impl PartialOrd for Arc {
    fn partial_cmp(&self, other: &Arc) -> Option<Ordering> {
        self.get_x().partial_cmp(&other.get_x())
    }
}

impl Ord for Arc {
    fn cmp(&self, other: &Arc) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Less)
    }
}

#[derive(Debug)]
struct DiagramBuilder {
    diagram: Diagram,
    arcs: BTreeSet<Arc>,
    events: BinaryHeap<Event>,
}

impl DiagramBuilder {
    pub fn new(bounding_rect: Rect, sites: Vec<Point2<f32>>) -> DiagramBuilder {
        let mut events = BinaryHeap::new();

        for site in sites {
            let position = Point2::new(
                (site.x - bounding_rect.position.x) / bounding_rect.size.x,
                (site.y - bounding_rect.position.y) / bounding_rect.size.y
            );

            if position.x > 0.0 && position.y > 0.0 && position.x < 1.0 && position.y < 1.0 {
                events.push(Event::Site(position));
            }
        }

        DiagramBuilder {
            diagram: Diagram::default(),
            arcs: BTreeSet::new(),
            events: events,
        }
    }

    fn handle_site_event(&mut self, p: Point2<f32>) {

    }

    fn handle_circle_event(&mut self, y: f32) {

    }

    pub fn step(&mut self) -> bool {
        let event = self.events.pop();

        match event {
            Some(Event::Site(p)) => {
                self.handle_site_event(p);
            }
            Some(Event::Circle(y)) => {
                self.handle_circle_event(y);
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
