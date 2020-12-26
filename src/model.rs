use std::error::Error;

use geojson::{GeoJson, Value};

use geom::{Bounds, Polygon, Pt2D, Ring};
use widgetry::{Color, GeomBatch, SharedAppState};

pub struct Object {
    pub color: Color,
    pub polygon: Polygon,
    pub name: String,
}

pub struct Model {
    pub objects: Vec<Object>,
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
                Some("intersection") => Color::RED,
                Some("road") => Color::GREEN,
                _ => Color::BLUE,
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
}
