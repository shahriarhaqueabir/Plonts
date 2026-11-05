use rusqlite::{params, Connection};
use std::collections::HashMap;

use crate::error::{AppError, AppResult};
use garden_core::layout::{circles_overlap, default_spacing_cm, shade_conflict};
use garden_core::models::{BedShape, Layout, LayoutDiff, LayoutPlant, LayoutWarning, WarningKind};

#[cfg(test)]
use crate::db;

#[cfg(test)]
fn in_memory() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    db::migrate(&mut conn).unwrap();
    conn
}

#[cfg(test)]
fn seeded() -> Connection {
    let mut conn = in_memory();
    db::seed::seed(&mut conn).unwrap();
    conn
}

fn new_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn layout_from_row(row: &rusqlite::Row) -> rusqlite::Result<Layout> {
    Ok(Layout {
        id: row.get(0)?,
        garden_id: row.get(1)?,
        name: row.get(2)?,
        version: row.get(3)?,
        canvas_json: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}

pub fn list_layouts(conn: &Connection, garden_id: &str) -> AppResult<Vec<Layout>> {
    let mut stmt = conn.prepare(
        "SELECT id, garden_id, name, version, canvas_json, created_at, updated_at
         FROM layouts WHERE garden_id = ?1 ORDER BY created_at",
    )?;
    let rows = stmt.query_map(params![garden_id], layout_from_row)?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn get_layout(conn: &Connection, id: &str) -> AppResult<Layout> {
    conn.query_row(
        "SELECT id, garden_id, name, version, canvas_json, created_at, updated_at
         FROM layouts WHERE id = ?1",
        params![id],
        layout_from_row,
    )
    .map_err(|_| AppError::NotFound(id.to_string()))
}

pub fn create_layout(conn: &Connection, garden_id: &str, name: &str) -> AppResult<Layout> {
    if name.trim().is_empty() {
        return Err(AppError::InvalidInput("layout name is required".into()));
    }
    let id = new_id();
    conn.execute(
        "INSERT INTO layouts (id, garden_id, name, version, canvas_json) VALUES (?1, ?2, ?3, 1, '[]')",
        params![id, garden_id, name],
    )?;
    get_layout(conn, &id)
}

pub fn save_layout(conn: &Connection, id: &str, canvas_json: &str) -> AppResult<Layout> {
    let n = conn.execute(
        "UPDATE layouts SET canvas_json = ?1, version = version + 1, updated_at = datetime('now') WHERE id = ?2",
        params![canvas_json, id],
    )?;
    if n == 0 {
        return Err(AppError::NotFound(id.to_string()));
    }
    get_layout(conn, id)
}

pub fn delete_layout(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM layouts WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn duplicate_layout(conn: &Connection, id: &str) -> AppResult<Layout> {
    let src = get_layout(conn, id)?;
    let new_layout_id = new_id();
    conn.execute(
        "INSERT INTO layouts (id, garden_id, name, version, canvas_json) VALUES (?1, ?2, ?3, 1, ?4)",
        params![new_layout_id, src.garden_id, format!("{} (copy)", src.name), src.canvas_json],
    )?;
    let mut sel = conn.prepare(
        "SELECT catalog_id, bed_id, x, y, quantity FROM layout_plants WHERE layout_id = ?1",
    )?;
    let rows = sel.query_map(params![id], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, Option<String>>(1)?,
            r.get::<_, f64>(2)?,
            r.get::<_, f64>(3)?,
            r.get::<_, i64>(4)?,
        ))
    })?;
    for row in rows {
        let (catalog_id, bed_id, x, y, quantity) = row?;
        let plant_id = new_id();
        conn.execute(
            "INSERT INTO layout_plants (id, layout_id, catalog_id, bed_id, x, y, quantity)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![plant_id, new_layout_id, catalog_id, bed_id, x, y, quantity],
        )?;
    }
    get_layout(conn, &new_layout_id)
}

pub fn list_layout_plants(conn: &Connection, layout_id: &str) -> AppResult<Vec<LayoutPlant>> {
    let mut stmt = conn.prepare(
        "SELECT lp.id, lp.layout_id, lp.catalog_id, c.name, lp.bed_id, lp.x, lp.y, lp.quantity, lp.created_at
         FROM layout_plants lp JOIN catalog c ON c.id = lp.catalog_id
         WHERE lp.layout_id = ?1 ORDER BY lp.created_at",
    )?;
    let rows = stmt.query_map(params![layout_id], |r| {
        Ok(LayoutPlant {
            id: r.get(0)?,
            layout_id: r.get(1)?,
            catalog_id: r.get(2)?,
            plant_name: r.get(3)?,
            bed_id: r.get(4)?,
            x: r.get(5)?,
            y: r.get(6)?,
            quantity: r.get(7)?,
            created_at: r.get(8)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

pub fn get_layout_plant(conn: &Connection, id: &str) -> AppResult<LayoutPlant> {
    conn.query_row(
        "SELECT lp.id, lp.layout_id, lp.catalog_id, c.name, lp.bed_id, lp.x, lp.y, lp.quantity, lp.created_at
         FROM layout_plants lp JOIN catalog c ON c.id = lp.catalog_id WHERE lp.id = ?1",
        params![id],
        |r| {
            Ok(LayoutPlant {
                id: r.get(0)?,
                layout_id: r.get(1)?,
                catalog_id: r.get(2)?,
                plant_name: r.get(3)?,
                bed_id: r.get(4)?,
                x: r.get(5)?,
                y: r.get(6)?,
                quantity: r.get(7)?,
                created_at: r.get(8)?,
            })
        },
    )
    .map_err(|_| AppError::NotFound(id.to_string()))
}

#[allow(clippy::too_many_arguments)]
pub fn add_layout_plant(
    conn: &Connection,
    layout_id: &str,
    catalog_id: &str,
    bed_id: Option<&str>,
    x: f64,
    y: f64,
    quantity: i64,
) -> AppResult<LayoutPlant> {
    let id = new_id();
    conn.execute(
        "INSERT INTO layout_plants (id, layout_id, catalog_id, bed_id, x, y, quantity)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, layout_id, catalog_id, bed_id, x, y, quantity],
    )?;
    get_layout_plant(conn, &id)
}

pub fn update_layout_plant(
    conn: &Connection,
    id: &str,
    bed_id: Option<&str>,
    x: f64,
    y: f64,
    quantity: i64,
) -> AppResult<LayoutPlant> {
    let n = conn.execute(
        "UPDATE layout_plants SET bed_id = ?1, x = ?2, y = ?3, quantity = ?4 WHERE id = ?5",
        params![bed_id, x, y, quantity, id],
    )?;
    if n == 0 {
        return Err(AppError::NotFound(id.to_string()));
    }
    get_layout_plant(conn, id)
}

pub fn delete_layout_plant(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM layout_plants WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn compare_layouts(conn: &Connection, a_id: &str, b_id: &str) -> AppResult<LayoutDiff> {
    let a = list_layout_plants(conn, a_id)?;
    let b = list_layout_plants(conn, b_id)?;

    fn group(plants: &[LayoutPlant]) -> HashMap<String, Vec<LayoutPlant>> {
        let mut m: HashMap<String, Vec<LayoutPlant>> = HashMap::new();
        for p in plants {
            m.entry(p.catalog_id.clone()).or_default().push(p.clone());
        }
        for v in m.values_mut() {
            v.sort_by(|x, y| (x.x, x.y).partial_cmp(&(y.x, y.y)).unwrap());
        }
        m
    }

    let a_g = group(&a);
    let b_g = group(&b);

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut moved = Vec::new();

    for (cat, a_list) in &a_g {
        match b_g.get(cat) {
            None => removed.extend(a_list.iter().cloned()),
            Some(b_list) => {
                let n = a_list.len().min(b_list.len());
                for i in 0..n {
                    let pa = &a_list[i];
                    let pb = &b_list[i];
                    if (pa.x - pb.x).abs() > f64::EPSILON || (pa.y - pb.y).abs() > f64::EPSILON {
                        moved.push(pb.clone());
                    }
                }
                if a_list.len() > b_list.len() {
                    removed.extend(a_list[n..].iter().cloned());
                } else if b_list.len() > a_list.len() {
                    added.extend(b_list[n..].iter().cloned());
                }
            }
        }
    }
    for (cat, b_list) in &b_g {
        if !a_g.contains_key(cat) {
            added.extend(b_list.iter().cloned());
        }
    }

    Ok(LayoutDiff { added, removed, moved })
}

struct PlantMeta {
    categories: Vec<String>,
    sunlight: Option<String>,
    antagonists: Vec<String>,
}

fn plant_meta(conn: &Connection, catalog_id: &str) -> AppResult<PlantMeta> {
    let (categories_json, sunlight): (String, Option<String>) = conn
        .query_row(
            "SELECT categories, sunlight FROM catalog WHERE id = ?1",
            params![catalog_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .map_err(|_| AppError::NotFound(catalog_id.to_string()))?;
    let categories: Vec<String> = serde_json::from_str(&categories_json).unwrap_or_default();
    let mut stmt = conn.prepare(
        "SELECT related_id FROM plant_relations WHERE plant_id = ?1 AND relation_type = 'antagonist'",
    )?;
    let antagonists = stmt
        .query_map(params![catalog_id], |r| r.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(PlantMeta { categories, sunlight, antagonists })
}

fn spacing_of(meta: &PlantMeta) -> f64 {
    default_spacing_cm(meta.categories.first().map(|s| s.as_str()).unwrap_or(""))
}

pub fn analyze_layout(conn: &Connection, layout_id: &str) -> AppResult<Vec<LayoutWarning>> {
    let layout = get_layout(conn, layout_id)?;
    let beds: Vec<BedShape> = serde_json::from_str(&layout.canvas_json).unwrap_or_default();
    let plants = list_layout_plants(conn, layout_id)?;

    let mut metas: HashMap<String, PlantMeta> = HashMap::new();
    for p in &plants {
        if !metas.contains_key(&p.catalog_id) {
            if let Ok(m) = plant_meta(conn, &p.catalog_id) {
                metas.insert(p.catalog_id.clone(), m);
            }
        }
    }

    let mut by_bed: HashMap<Option<String>, Vec<&LayoutPlant>> = HashMap::new();
    for p in &plants {
        by_bed.entry(p.bed_id.clone()).or_default().push(p);
    }

    let mut warnings = Vec::new();

    for (bed_id, group) in &by_bed {
        for i in 0..group.len() {
            for j in (i + 1)..group.len() {
                let a = group[i];
                let b = group[j];

                if let (Some(ma), Some(mb)) = (metas.get(&a.catalog_id), metas.get(&b.catalog_id)) {
                    let ra = spacing_of(ma) / 2.0;
                    let rb = spacing_of(mb) / 2.0;
                    if circles_overlap(a.x, a.y, ra, b.x, b.y, rb) {
                        warnings.push(LayoutWarning {
                            kind: WarningKind::Overcrowding,
                            plant_a: a.catalog_id.clone(),
                            plant_b: Some(b.catalog_id.clone()),
                            message: format!("{} and {} are too close — spacing circles overlap.", a.plant_name, b.plant_name),
                        });
                    }
                }

                let a_against_b = metas.get(&a.catalog_id).map(|m| m.antagonists.contains(&b.catalog_id)).unwrap_or(false);
                let b_against_a = metas.get(&b.catalog_id).map(|m| m.antagonists.contains(&a.catalog_id)).unwrap_or(false);
                if a_against_b || b_against_a {
                    warnings.push(LayoutWarning {
                        kind: WarningKind::Companion,
                        plant_a: a.catalog_id.clone(),
                        plant_b: Some(b.catalog_id.clone()),
                        message: format!("{} and {} are poor companions.", a.plant_name, b.plant_name),
                    });
                }
            }
        }

        if let Some(bed_id) = bed_id {
            if let Some(bed) = beds.iter().find(|b| &b.id == bed_id) {
                for p in group.iter() {
                    let sunlight = metas.get(&p.catalog_id).and_then(|m| m.sunlight.as_deref());
                    if shade_conflict(sunlight, bed.sun_exposure.as_deref()) {
                        warnings.push(LayoutWarning {
                            kind: WarningKind::Shade,
                            plant_a: p.catalog_id.clone(),
                            plant_b: None,
                            message: format!("{} needs more sun than this bed provides.", p.plant_name),
                        });
                    }
                }
            }
        }
    }

    Ok(warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup(garden_name: &str) -> (Connection, String) {
        let conn = seeded();
        let g = db::gardens::create_garden(&conn, garden_name, "raised_bed").unwrap();
        (conn, g.id)
    }

    fn bed_json(id: &str, sun: Option<&str>) -> String {
        let beds = vec![BedShape {
            id: id.to_string(),
            name: "Bed 1".to_string(),
            x: 0.0,
            y: 0.0,
            width_cm: 100.0,
            height_cm: 100.0,
            sun_exposure: sun.map(|s| s.to_string()),
        }];
        serde_json::to_string(&beds).unwrap()
    }

    #[test]
    fn create_and_list_layouts() {
        let (conn, gid) = setup("G");
        let l = create_layout(&conn, &gid, "Spring").unwrap();
        assert_eq!(l.version, 1);
        assert_eq!(list_layouts(&conn, &gid).unwrap().len(), 1);
    }

    #[test]
    fn save_bumps_version() {
        let (conn, gid) = setup("G");
        let l = create_layout(&conn, &gid, "Spring").unwrap();
        let saved = save_layout(&conn, &l.id, &bed_json("b1", None)).unwrap();
        assert_eq!(saved.version, 2);
    }

    #[test]
    fn duplicate_copies_plants_and_canvas() {
        let (conn, gid) = setup("G");
        let l = create_layout(&conn, &gid, "Spring").unwrap();
        let saved = save_layout(&conn, &l.id, &bed_json("b1", None)).unwrap();
        add_layout_plant(&conn, &l.id, "plant_tomato", Some("b1"), 10.0, 10.0, 1).unwrap();
        let dup = duplicate_layout(&conn, &l.id).unwrap();
        assert_eq!(dup.name, "Spring (copy)");
        assert_eq!(list_layout_plants(&conn, &dup.id).unwrap().len(), 1);
        assert_eq!(dup.canvas_json, saved.canvas_json);
    }

    #[test]
    fn plants_roundtrip() {
        let (conn, gid) = setup("G");
        let l = create_layout(&conn, &gid, "Spring").unwrap();
        let p = add_layout_plant(&conn, &l.id, "plant_tomato", None, 10.0, 20.0, 2).unwrap();
        assert!(!p.plant_name.is_empty());
        assert_eq!(p.x, 10.0);
        let moved = update_layout_plant(&conn, &p.id, Some("b1"), 50.0, 60.0, 3).unwrap();
        assert_eq!(moved.x, 50.0);
        assert_eq!(moved.quantity, 3);
        assert_eq!(list_layout_plants(&conn, &l.id).unwrap().len(), 1);
        delete_layout_plant(&conn, &p.id).unwrap();
        assert!(list_layout_plants(&conn, &l.id).unwrap().is_empty());
    }

    #[test]
    fn compare_reports_added_removed_moved() {
        let (conn, gid) = setup("G");
        let a = create_layout(&conn, &gid, "A").unwrap();
        let b = create_layout(&conn, &gid, "B").unwrap();
        add_layout_plant(&conn, &a.id, "plant_tomato", None, 0.0, 0.0, 1).unwrap();
        add_layout_plant(&conn, &b.id, "plant_tomato", None, 10.0, 10.0, 1).unwrap();
        add_layout_plant(&conn, &b.id, "plant_arugula", None, 0.0, 0.0, 1).unwrap();
        let diff = compare_layouts(&conn, &a.id, &b.id).unwrap();
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.added[0].catalog_id, "plant_arugula");
        assert_eq!(diff.removed.len(), 0);
        assert_eq!(diff.moved.len(), 1);
        assert_eq!(diff.moved[0].catalog_id, "plant_tomato");
    }

    #[test]
    fn analyze_reports_overcrowding() {
        let (conn, gid) = setup("G");
        let l = create_layout(&conn, &gid, "Spring").unwrap();
        save_layout(&conn, &l.id, &bed_json("b1", None)).unwrap();
        add_layout_plant(&conn, &l.id, "plant_tomato", Some("b1"), 0.0, 0.0, 1).unwrap();
        add_layout_plant(&conn, &l.id, "plant_tomato", Some("b1"), 10.0, 10.0, 1).unwrap();
        let ws = analyze_layout(&conn, &l.id).unwrap();
        assert!(ws.iter().any(|w| w.kind == WarningKind::Overcrowding));
    }

    #[test]
    fn analyze_reports_companion_conflict() {
        let (conn, gid) = setup("G");
        let l = create_layout(&conn, &gid, "Spring").unwrap();
        save_layout(&conn, &l.id, &bed_json("b1", None)).unwrap();
        // seeded: plant_apricot lists plant_tomato as an antagonist
        add_layout_plant(&conn, &l.id, "plant_apricot", Some("b1"), 0.0, 0.0, 1).unwrap();
        add_layout_plant(&conn, &l.id, "plant_tomato", Some("b1"), 10.0, 10.0, 1).unwrap();
        let ws = analyze_layout(&conn, &l.id).unwrap();
        assert!(ws.iter().any(|w| w.kind == WarningKind::Companion));
    }

    #[test]
    fn analyze_reports_shade_conflict() {
        let (conn, gid) = setup("G");
        let l = create_layout(&conn, &gid, "Spring").unwrap();
        save_layout(&conn, &l.id, &bed_json("b1", Some("shade"))).unwrap();
        // seeded: plant_apricot is full_sun, placed in a shady bed
        add_layout_plant(&conn, &l.id, "plant_apricot", Some("b1"), 0.0, 0.0, 1).unwrap();
        let ws = analyze_layout(&conn, &l.id).unwrap();
        assert!(ws.iter().any(|w| w.kind == WarningKind::Shade));
    }
}