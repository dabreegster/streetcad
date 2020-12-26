use std::error::Error;

use geojson::{GeoJson, Value};

use geom::{Bounds, Circle, Distance, Polygon, Pt2D, Ring};
use widgetry::{Color, GeomBatch, SharedAppState};

pub struct Object {
    pub color: Color,
    pub polygon: Polygon,
    pub name: String,
}

pub struct Model {
    pub objects: Vec<Object>,
}

// Just indexes into model.objects
type ID = usize;

#[derive(Clone, PartialEq)]
pub enum Hovering {
    Polygon(ID),
}

impl SharedAppState for Model {}

impl Model {
    pub fn load_geojson(path: String) -> Result<Model, Box<dyn Error>> {
        let raw = abstutil::slurp_file(&path)?;
        let geojson = String::from_utf8(raw)?.parse::<GeoJson>()?;
        let mut objects = Vec::new();
        let features = match geojson {
            GeoJson::Feature(feature) => vec![feature],
            GeoJson::FeatureCollection(feature_collection) => feature_collection.features,
            _ => return Err(format!("Unexpected geojson: {:?}", geojson).into()),
        };
        // TODO Can't we just use https://docs.rs/geojson/0.21.0/geojson/#conversion-to-geo-objects and
        // the geo->geom conversions?
        for mut feature in features {
            let points = match feature.geometry.take().map(|g| g.value) {
                Some(Value::MultiPolygon(multi_polygon)) => multi_polygon[0][0].clone(),
                Some(Value::Polygon(polygon)) => polygon[0].clone(),
                _ => {
                    return Err(format!("Unexpected feature: {:?}", feature).into());
                }
            };
            let polygon = Ring::new(
                points
                    .into_iter()
                    .map(|pt| Pt2D::new(pt[0], pt[1]))
                    .collect(),
            )?
            .to_polygon();
            let name = feature
                .property("id")
                .and_then(|prop| prop.as_str())
                .unwrap_or("unnamed")
                .to_string();
            let color = match feature.property("type").and_then(|prop| prop.as_str()) {
                Some("intersection") => Color::grey(0.5),
                Some("road") => Color::grey(0.2),
                _ => Color::grey(0.8),
            };
            objects.push(Object {
                polygon,
                color,
                name,
            });
        }
        Ok(Model { objects })
    }

    pub fn get_bounds(&self) -> Bounds {
        let mut b = Bounds::new();
        for obj in &self.objects {
            for pt in obj.polygon.points() {
                b.update(*pt);
            }
        }
        b
    }

    pub fn render(&self) -> GeomBatch {
        let mut batch = GeomBatch::new();
        for obj in &self.objects {
            batch.push(obj.color, obj.polygon.clone());
        }
        batch
    }

    pub fn compute_hovering(&self, cursor: Pt2D) -> Option<Hovering> {
        for (idx, obj) in self.objects.iter().enumerate() {
            if obj.polygon.contains_pt(cursor) {
                return Some(Hovering::Polygon(idx));
            }
        }
        None
    }
}

impl Object {
    fn get_pts(&self) -> Vec<Circle> {
        let mut circles: Vec<Circle> = self
            .polygon
            .points()
            .iter()
            .map(|pt| Circle::new(*pt, Distance::meters(1.0)))
            .collect();
        // Don't return a duplicate for the first/last point
        circles.pop();
        circles
    }
}

impl Hovering {
    pub fn render(&self, model: &Model) -> GeomBatch {
        let mut batch = GeomBatch::new();
        match self {
            Hovering::Polygon(idx) => {
                let obj = &model.objects[*idx];
                if let Ok(p) = obj.polygon.to_outline(Distance::meters(1.0)) {
                    batch.push(Color::YELLOW, p);
                }
                for circle in obj.get_pts() {
                    batch.push(Color::RED, circle.to_polygon());
                }
            }
        }
        batch
    }
}
