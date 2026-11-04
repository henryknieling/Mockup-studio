use axum::{
    Form, Json, Router,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
    routing::{get, post},
};
use printpdf::{
    BuiltinFont, Color, Mm, PdfDocument, Point, Polygon, Rgb,
    path::{PaintMode, WindingOrder},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use std::{io::BufWriter, net::SocketAddr};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db: PgPool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScreenDocument {
    #[serde(default = "one")]
    version: u32,
    #[serde(default)]
    elements: Vec<Element>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Element {
    id: String,
    kind: String,
    #[serde(default)]
    x: f32,
    #[serde(default)]
    y: f32,
    #[serde(default = "w160")]
    width: f32,
    #[serde(default = "h48")]
    height: f32,
    #[serde(default)]
    text: String,
    #[serde(default = "white")]
    fill: String,
    #[serde(default = "ink")]
    color: String,
    #[serde(default = "fs16")]
    font_size: f32,
    #[serde(default)]
    placeholder: String,
    #[serde(default = "r8")]
    radius: f32,
    #[serde(default = "left_align")]
    text_align: String,
    #[serde(default)]
    items: Vec<String>,
}
fn one() -> u32 {
    1
}
fn w160() -> f32 {
    160.
}
fn h48() -> f32 {
    48.
}
fn fs16() -> f32 {
    16.
}
fn r8() -> f32 {
    8.
}
fn left_align() -> String {
    "left".into()
}
fn white() -> String {
    "#ffffff".into()
}
fn ink() -> String {
    "#172033".into()
}
#[derive(Deserialize)]
struct ProjectForm {
    name: String,
}
#[derive(Deserialize)]
struct ScreenForm {
    name: String,
    width: Option<i32>,
    height: Option<i32>,
    frame: Option<String>,
}
#[derive(Deserialize)]
struct RenameForm {
    name: String,
}
#[derive(Deserialize)]
struct ViewportForm {
    width: i32,
    height: i32,
    frame: String,
}
#[derive(Deserialize)]
struct OrderForm {
    screen_ids: Vec<Uuid>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://mockup:mockup@localhost:5432/mockup_studio".into());
    let db = PgPoolOptions::new()
        .max_connections(8)
        .connect(&url)
        .await?;
    sqlx::migrate!().run(&db).await?;
    if std::env::args().nth(1).as_deref() == Some("migrate") {
        println!("Database migrations are up to date.");
        return Ok(());
    }
    let app = build_router(AppState { db });
    let addr: SocketAddr = "127.0.0.1:3000".parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Mockup Studio is running at http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(home).post(create_project))
        .route("/projects/{id}", get(project_page).post(rename_project))
        .route("/projects/{id}/delete", post(delete_project))
        .route("/projects/{id}/screens", post(create_screen))
        .route("/screens/{id}/rename", post(rename_screen))
        .route("/screens/{id}/delete", post(delete_screen))
        .route("/screens/{id}/viewport", post(update_viewport))
        .route("/screens/{id}/order", post(reorder_screens))
        .route("/api/screens/{id}", get(get_document).put(save_document))
        .route("/export/screens/{id}", get(export_screen))
        .route("/export/projects/{id}", get(export_project))
        .route("/assets/{*path}", get(serve_asset))
        .with_state(state)
}

async fn home(State(s): State<AppState>) -> Result<Html<String>, AppError> {
    let rows = sqlx::query("SELECT id,name FROM projects ORDER BY updated_at DESC")
        .fetch_all(&s.db)
        .await?;
    let mut cards = String::new();
    for r in rows {
        let id: Uuid = r.try_get("id")?;
        let name: String = r.try_get("name")?;
        cards.push_str(&format!("<a class=\"project-card\" href=\"/projects/{id}\"><span class=\"project-icon\">▧</span><strong>{}</strong><small>Open project →</small></a>",esc(&name)));
    }
    let body = format!(
        "<main class=\"home\"><header class=\"topbar\"><a class=\"brand\" href=\"/\"><span class=\"brand-mark\">M</span> Mockup Studio</a><span class=\"local-pill\">● Local workspace</span></header><section class=\"welcome\"><p class=\"eyebrow\">YOUR CREATIVE WORKSPACE</p><h1>Make it look <em>real.</em></h1><p>Bring your app and website ideas to life, one screen at a time.</p></section><section class=\"projects-section\"><div class=\"section-title\"><div><h2>Your projects</h2><p>Pick up where you left off or start something new.</p></div><form class=\"new-project\" method=\"post\" action=\"/\"><input name=\"name\" placeholder=\"New project name\" required maxlength=\"80\"><button class=\"primary\">＋ New project</button></form></div><div class=\"project-grid\">{cards}<form method=\"post\" action=\"/\" class=\"new-card\"><input name=\"name\" placeholder=\"Name your project\" required maxlength=\"80\"><button>＋ Create project</button></form></div></section><footer>Made for ideas in progress. Your work stays on this device.</footer></main>"
    );
    Ok(Html(layout("Projects", &body)))
}
async fn create_project(
    State(s): State<AppState>,
    Form(f): Form<ProjectForm>,
) -> Result<Redirect, AppError> {
    let row = sqlx::query("INSERT INTO projects(name) VALUES($1) RETURNING id")
        .bind(clean_name(&f.name, "Untitled project"))
        .fetch_one(&s.db)
        .await?;
    let id: Uuid = row.try_get("id")?;
    sqlx::query("INSERT INTO screens(project_id,name,position) VALUES($1,'Welcome',0)")
        .bind(id)
        .execute(&s.db)
        .await?;
    Ok(Redirect::to(&format!("/projects/{id}")))
}

async fn project_page(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let project = sqlx::query("SELECT name FROM projects WHERE id=$1")
        .bind(id)
        .fetch_optional(&s.db)
        .await?
        .ok_or(AppError::NotFound)?;
    let pname: String = project.try_get("name")?;
    let rows=sqlx::query("SELECT id,name,position,width,height,frame FROM screens WHERE project_id=$1 ORDER BY position,id").bind(id).fetch_all(&s.db).await?;
    if rows.is_empty() {
        sqlx::query("INSERT INTO screens(project_id,name,position) VALUES($1,'Screen 1',0)")
            .bind(id)
            .execute(&s.db)
            .await?;
        return Ok(Redirect::to(&format!("/projects/{id}")).into_response());
    }
    let mut tabs = String::new();
    let mut screens = Vec::new();
    for r in rows {
        let sid: Uuid = r.try_get("id")?;
        let name: String = r.try_get("name")?;
        let w: i32 = r.try_get("width")?;
        let h: i32 = r.try_get("height")?;
        let frame: String = r.try_get("frame")?;
        tabs.push_str(&format!("<button class=\"screen-tab\" data-screen=\"{sid}\" data-name=\"{}\" data-width=\"{w}\" data-height=\"{h}\" data-frame=\"{}\">{}</button>",esc(&name),esc(&frame),esc(&name)));
        screens.push((sid, name, w, h, frame));
    }
    let (active, an, w, h, frame) = screens[0].clone();
    let body = format!(
        r#"<div class="app-shell" id="editor" data-project="{id}" data-screen="{active}"><header class="topbar editor-topbar"><a class="brand" href="/"><span class="brand-mark">M</span></a><span class="crumb">/</span><a class="project-crumb" href="/projects/{id}">{pname}</a><details class="project-menu"><summary title="Project settings">•••</summary><form method="post" action="/projects/{id}" class="project-rename"><input name="name" value="{pname}" maxlength="80" aria-label="Project name"><button>Rename</button></form><form method="post" action="/projects/{id}/delete" onsubmit="return confirm('Delete this project and all its screens?')"><button class="danger">Delete project</button></form></details><span class="crumb">/</span><span id="active-name" class="active-crumb">{an}</span><div class="top-actions"><button id="undo" title="Undo">↶</button><button id="redo" title="Redo">↷</button><span class="save-status" id="save-status">All changes saved</span><a class="export-button" href="/export/projects/{id}">Export PDF</a></div></header>
    <div class="screenbar"><div class="screen-tabs" id="screen-tabs">{tabs}</div><form class="add-screen" method="post" action="/projects/{id}/screens"><input type="hidden" name="name" value="Screen {n}"><input type="hidden" name="width" value="390"><input type="hidden" name="height" value="844"><input type="hidden" name="frame" value="phone"><button>＋ Add screen</button></form></div>
    <div class="editor-layout"><aside class="library panel"><div class="panel-heading"><span>Insert</span><span class="muted">COMPONENTS</span></div><div class="library-group"><label>Mobile</label><div class="component-grid"><button data-add="text"><span>Ｔ</span>Text</button><button data-add="button"><span>▰</span>Button</button><button data-add="image"><span>▧</span>Image</button><button data-add="input"><span>▱</span>Input</button><button data-add="card"><span>▢</span>Card</button><button data-add="nav"><span>☰</span>Tab bar</button><button data-add="nav-title"><span>↕</span>Nav title</button></div></div><div class="library-group"><label>Web</label><div class="component-grid"><button data-add="web-card"><span>▤</span>Web card</button><button data-add="web-nav"><span>↔</span>Web nav</button><button data-add="heading"><span>H</span>Heading</button></div></div><div class="library-hint">Select a component to add it to your screen.</div></aside>
    <main class="canvas-area"><div class="canvas-toolbar"><div class="viewport-info"><span class="device-dot"></span><span id="viewport-label">{w} × {h}</span><span class="muted">px</span></div><div class="canvas-tools"><select id="preset"><option value="">Viewport preset</option><optgroup label="Mobile"><option value="390,844,phone">iPhone 14 · 390 × 844</option><option value="393,852,phone">iPhone 15 · 393 × 852</option><option value="360,800,phone">Android · 360 × 800</option></optgroup><optgroup label="Tablet & Web"><option value="768,1024,tablet">Tablet · 768 × 1024</option><option value="1440,900,browser">Desktop · 1440 × 900</option><option value="1280,800,browser">Laptop · 1280 × 800</option></optgroup><option value="custom">Custom size…</option></select><button id="screen-export" title="Export this screen">↓</button></div></div><div class="stage-wrap"><div id="device" class="device-frame {frame}"><div id="canvas" class="canvas" style="width:{w}px;height:{h}px"></div></div></div><div class="canvas-footer"><span>Drag to move · Pull corner to resize</span><span id="zoom-label">100%</span></div></main>
    <aside class="right-rail"><section class="panel inspector"><div class="panel-heading"><span>Inspector</span><span class="muted">PROPERTIES</span></div><div id="inspector-content" class="inspector-content"><div class="empty-inspector">Select an element on the canvas to edit its properties.</div></div></section><section class="panel layers"><div class="panel-heading"><span>Layers</span><span id="layer-count" class="muted">0</span></div><div id="layers-list" class="layers-list"></div></section><section class="panel screen-settings"><div class="panel-heading"><span>Screen</span><span class="muted">SETTINGS</span></div><form id="viewport-form" class="settings-form"><label>Width <input name="width" type="number" min="100" max="4000" value="{w}"></label><label>Height <input name="height" type="number" min="100" max="4000" value="{h}"></label><label class="full">Device frame<select name="frame"><option value="phone">Phone</option><option value="tablet">Tablet</option><option value="browser">Browser</option><option value="none">No frame</option></select></label><button class="quiet full" type="submit">Apply viewport</button></form><form id="rename-screen" class="rename-screen"><input name="name" value="{an}" aria-label="Screen name"><button title="Rename screen">↵</button><button type="button" id="delete-screen" title="Delete screen">⌫</button></form></section></aside></div></div><script src="/assets/editor.js" defer></script>"#,
        pname = esc(&pname),
        an = esc(&an),
        n = screens.len() + 1
    );
    Ok(Html(layout(&pname, &body)).into_response())
}

async fn rename_project(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Form(f): Form<RenameForm>,
) -> Result<Redirect, AppError> {
    sqlx::query("UPDATE projects SET name=$2,updated_at=now() WHERE id=$1")
        .bind(id)
        .bind(clean_name(&f.name, "Untitled project"))
        .execute(&s.db)
        .await?;
    Ok(Redirect::to(&format!("/projects/{id}")))
}
async fn delete_project(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Redirect, AppError> {
    sqlx::query("DELETE FROM projects WHERE id=$1")
        .bind(id)
        .execute(&s.db)
        .await?;
    Ok(Redirect::to("/"))
}
async fn create_screen(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Form(f): Form<ScreenForm>,
) -> Result<Redirect, AppError> {
    let pos: i32 =
        sqlx::query_scalar("SELECT COALESCE(MAX(position),-1)+1 FROM screens WHERE project_id=$1")
            .bind(id)
            .fetch_one(&s.db)
            .await?;
    sqlx::query("INSERT INTO screens(project_id,name,position,width,height,frame) VALUES($1,$2,$3,$4,$5,$6)").bind(id).bind(clean_name(&f.name,"New screen")).bind(pos).bind(f.width.unwrap_or(390).clamp(100,4000)).bind(f.height.unwrap_or(844).clamp(100,4000)).bind(valid_frame(f.frame.as_deref().unwrap_or("phone"))).execute(&s.db).await?;
    Ok(Redirect::to(&format!("/projects/{id}")))
}
async fn rename_screen(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Form(f): Form<RenameForm>,
) -> Result<StatusCode, AppError> {
    sqlx::query("UPDATE screens SET name=$2,updated_at=now() WHERE id=$1")
        .bind(id)
        .bind(clean_name(&f.name, "Untitled screen"))
        .execute(&s.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn delete_screen(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    sqlx::query("DELETE FROM screens WHERE id=$1")
        .bind(id)
        .execute(&s.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn update_viewport(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Form(f): Form<ViewportForm>,
) -> Result<StatusCode, AppError> {
    sqlx::query("UPDATE screens SET width=$2,height=$3,frame=$4,updated_at=now() WHERE id=$1")
        .bind(id)
        .bind(f.width.clamp(100, 4000))
        .bind(f.height.clamp(100, 4000))
        .bind(valid_frame(&f.frame))
        .execute(&s.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn reorder_screens(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(f): Json<OrderForm>,
) -> Result<StatusCode, AppError> {
    let mut tx = s.db.begin().await?;
    for (pos, sid) in f.screen_ids.iter().enumerate() {
        sqlx::query("UPDATE screens SET position=$3 WHERE id=$1 AND project_id=$2")
            .bind(sid)
            .bind(id)
            .bind(pos as i32)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn get_document(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let r = sqlx::query(
        "SELECT name,width,height,frame,document_version,document FROM screens WHERE id=$1",
    )
    .bind(id)
    .fetch_optional(&s.db)
    .await?
    .ok_or(AppError::NotFound)?;
    Ok(Json(
        serde_json::json!({"id":id,"name":r.try_get::<String,_>("name")?,"width":r.try_get::<i32,_>("width")?,"height":r.try_get::<i32,_>("height")?,"frame":r.try_get::<String,_>("frame")?,"document_version":r.try_get::<i32,_>("document_version")?,"document":r.try_get::<Value,_>("document")?}),
    ))
}
async fn save_document(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
    Json(doc): Json<ScreenDocument>,
) -> Result<StatusCode, AppError> {
    if doc.elements.len() > 500 {
        return Err(AppError::BadRequest(
            "A screen can contain at most 500 elements".into(),
        ));
    }
    let value = serde_json::to_value(doc)?;
    let result = sqlx::query(
        "UPDATE screens SET document=$2,document_version=1,updated_at=now() WHERE id=$1",
    )
    .bind(id)
    .bind(value)
    .execute(&s.db)
    .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn export_screen(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let r = sqlx::query("SELECT name,width,height,frame,document FROM screens WHERE id=$1")
        .bind(id)
        .fetch_optional(&s.db)
        .await?
        .ok_or(AppError::NotFound)?;
    pdf_response(vec![(
        r.try_get("name")?,
        r.try_get("width")?,
        r.try_get("height")?,
        r.try_get("frame")?,
        r.try_get("document")?,
    )])
}
async fn export_project(
    State(s): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let p = sqlx::query("SELECT name FROM projects WHERE id=$1")
        .bind(id)
        .fetch_optional(&s.db)
        .await?
        .ok_or(AppError::NotFound)?;
    let name: String = p.try_get("name")?;
    let rows=sqlx::query("SELECT name,width,height,frame,document FROM screens WHERE project_id=$1 ORDER BY position,id").bind(id).fetch_all(&s.db).await?;
    let mut pages = Vec::new();
    for r in rows {
        pages.push((
            r.try_get("name")?,
            r.try_get("width")?,
            r.try_get("height")?,
            r.try_get("frame")?,
            r.try_get("document")?,
        ));
    }
    if pages.is_empty() {
        return Err(AppError::NotFound);
    }
    let mut res = pdf_response(pages)?;
    res.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename=\"{}.pdf\"", safe_filename(&name))
            .parse()
            .unwrap(),
    );
    Ok(res)
}
fn pdf_response(pages: Vec<(String, i32, i32, String, Value)>) -> Result<Response, AppError> {
    let (first, w, h, _, _) = &pages[0];
    let (doc, first_page, first_layer) =
        PdfDocument::new(first, Mm(px_mm(*w as f32)), Mm(px_mm(*h as f32)), "Mockup");
    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    for (i, (name, w, h, frame, json)) in pages.iter().enumerate() {
        let (page, layer_index) = if i == 0 {
            (first_page, first_layer)
        } else {
            doc.add_page(
                Mm(px_mm(*w as f32)),
                Mm(px_mm(*h as f32)),
                format!("Screen {}", i + 1),
            )
        };
        let layer = doc.get_page(page).get_layer(layer_index);
        if frame != "none" {
            let inset = if frame == "browser" { 0. } else { 10. };
            rect(
                &layer,
                inset,
                inset,
                *w as f32 - 2. * inset,
                *h as f32 - 2. * inset,
                "#ffffff",
                "#d7dce5",
            );
        }
        let elems: Vec<Element> = serde_json::from_value(
            json.get("elements")
                .cloned()
                .unwrap_or(Value::Array(vec![])),
        )
        .unwrap_or_default();
        for e in elems {
            let py = *h as f32 - e.y - e.height;
            match e.kind.as_str() {
                "text" | "heading" | "nav-title" => {
                    let text = if e.text.is_empty() { "Text" } else { &e.text };
                    let estimated_width = text.chars().count() as f32 * e.font_size * 0.52;
                    let offset = match e.text_align.as_str() {
                        "center" => (e.width - estimated_width) / 2.,
                        "right" => e.width - estimated_width,
                        _ => 0.,
                    };
                    layer.use_text(
                        text,
                        e.font_size.max(6.) * 0.75,
                        Mm(px_mm(e.x + offset.max(0.))),
                        Mm(px_mm(*h as f32 - e.y - e.font_size)),
                        &font,
                    )
                }
                "image" => rect(&layer, e.x, py, e.width, e.height, "#edf1f7", "#cbd3df"),
                "nav" | "web-nav" => {
                    rect(&layer, e.x, py, e.width, e.height, &e.fill, &e.color);
                    let items = nav_labels(&e);
                    let text_widths: Vec<f32> = items
                        .iter()
                        .map(|label| label.chars().count() as f32 * e.font_size * 0.5 + 16.)
                        .collect();
                    let total_width: f32 = text_widths.iter().sum();
                    let gap = if e.text_align == "space-between" && items.len() > 1 {
                        ((e.width - 16. - total_width) / (items.len() - 1) as f32).max(0.)
                    } else {
                        0.
                    };
                    let mut x = match e.text_align.as_str() {
                        "center" => {
                            e.x + (e.width
                                - total_width
                                - gap * items.len().saturating_sub(1) as f32)
                                / 2.
                        }
                        "right" => {
                            e.x + e.width
                                - total_width
                                - gap * items.len().saturating_sub(1) as f32
                                - 8.
                        }
                        _ => e.x + 8.,
                    };
                    for (label, text_width) in items.iter().zip(text_widths) {
                        layer.use_text(
                            label,
                            e.font_size.max(6.) * 0.75,
                            Mm(px_mm(x + 8.)),
                            Mm(px_mm(*h as f32 - e.y - e.height / 2.)),
                            &font,
                        );
                        x += text_width + gap;
                    }
                }
                _ => {
                    rect(&layer, e.x, py, e.width, e.height, &e.fill, &e.color);
                    if !e.text.is_empty() {
                        layer.use_text(
                            &e.text,
                            e.font_size.max(6.) * 0.75,
                            Mm(px_mm(e.x + 12.)),
                            Mm(px_mm(*h as f32 - e.y - e.height / 2.)),
                            &font,
                        )
                    }
                }
            }
        }
        let _ = name;
    }
    let mut out = BufWriter::new(Vec::new());
    doc.save(&mut out)?;
    let bytes = out
        .into_inner()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"mockup.pdf\"",
            ),
        ],
        bytes,
    )
        .into_response())
}
fn nav_labels(element: &Element) -> Vec<String> {
    if !element.items.is_empty() {
        return element.items.clone();
    }
    element
        .text
        .split(['\u{3000}', '|'])
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(String::from)
        .collect()
}
fn rect(
    layer: &printpdf::PdfLayerReference,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    fill: &str,
    stroke: &str,
) {
    let color = |s: &str| {
        let v = s.trim_start_matches('#');
        if v.len() == 6 {
            Color::Rgb(Rgb::new(
                u8::from_str_radix(&v[0..2], 16).unwrap_or(255) as f32 / 255.,
                u8::from_str_radix(&v[2..4], 16).unwrap_or(255) as f32 / 255.,
                u8::from_str_radix(&v[4..6], 16).unwrap_or(255) as f32 / 255.,
                None,
            ))
        } else {
            Color::Rgb(Rgb::new(1., 1., 1., None))
        }
    };
    layer.set_fill_color(color(fill));
    layer.set_outline_color(color(stroke));
    let p = vec![
        (Point::new(Mm(px_mm(x)), Mm(px_mm(y))), false),
        (Point::new(Mm(px_mm(x + w)), Mm(px_mm(y))), false),
        (Point::new(Mm(px_mm(x + w)), Mm(px_mm(y + h))), false),
        (Point::new(Mm(px_mm(x)), Mm(px_mm(y + h))), false),
    ];
    layer.add_polygon(Polygon {
        rings: vec![p],
        mode: PaintMode::FillStroke,
        winding_order: WindingOrder::NonZero,
    });
}
fn px_mm(v: f32) -> f32 {
    v * 25.4 / 96.
}
async fn serve_asset(Path(path): Path<String>) -> Result<Response, AppError> {
    let (text, ty) = match path.as_str() {
        "editor.js" => (
            include_str!("../static/editor.js"),
            "application/javascript; charset=utf-8",
        ),
        "style.css" => (
            include_str!("../static/style.css"),
            "text/css; charset=utf-8",
        ),
        _ => return Err(AppError::NotFound),
    };
    Ok(([(header::CONTENT_TYPE, ty)], text).into_response())
}

fn layout(title: &str, body: &str) -> String {
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{} · Mockup Studio</title><link rel=\"stylesheet\" href=\"/assets/style.css\"></head><body>{body}</body></html>",
        esc(title)
    )
}
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn clean_name(s: &str, fallback: &str) -> String {
    let t = s.trim();
    if t.is_empty() {
        fallback.into()
    } else {
        t.chars().take(80).collect()
    }
}
fn valid_frame(s: &str) -> &str {
    match s {
        "phone" | "tablet" | "browser" | "none" => s,
        _ => "phone",
    }
}
fn safe_filename(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>()
        .to_lowercase()
}
#[derive(Debug)]
enum AppError {
    Db(sqlx::Error),
    NotFound,
    BadRequest(String),
    Internal(String),
    Pdf(printpdf::Error),
    Json(serde_json::Error),
}
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        Self::Db(e)
    }
}
impl From<printpdf::Error> for AppError {
    fn from(e: printpdf::Error) -> Self {
        Self::Pdf(e)
    }
}
impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "Not found".into()),
            Self::BadRequest(m) => (StatusCode::BAD_REQUEST, m),
            Self::Db(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m),
            Self::Pdf(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            Self::Json(e) => (StatusCode::BAD_REQUEST, e.to_string()),
        };
        (status, msg).into_response()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn router_registers_export_routes() {
        let db = PgPoolOptions::new()
            .connect_lazy("postgres://mockup:mockup@localhost/mockup_studio")
            .unwrap();
        let _router = build_router(AppState { db });
    }

    #[test]
    fn document_defaults() {
        let d: ScreenDocument = serde_json::from_str(r#"{"elements":[]}"#).unwrap();
        assert_eq!(d.version, 1);
        assert!(d.elements.is_empty());
    }

    #[test]
    fn navigation_labels_support_legacy_and_editable_items() {
        let legacy: Element = serde_json::from_value(serde_json::json!({
            "id":"tabs","kind":"nav","text":"Home　 Search　 Profile"
        }))
        .unwrap();
        assert_eq!(nav_labels(&legacy), ["Home", "Search", "Profile"]);

        let editable: Element = serde_json::from_value(serde_json::json!({
            "id":"tabs","kind":"nav","items":["Feed", "Saved"]
        }))
        .unwrap();
        assert_eq!(nav_labels(&editable), ["Feed", "Saved"]);
    }
    #[test]
    fn input_validation() {
        assert_eq!(clean_name(" hello ", "fallback"), "hello");
        assert_eq!(valid_frame("invalid"), "phone");
        assert_eq!(safe_filename("My Project!"), "myproject");
    }
    #[test]
    fn pdf_keeps_viewport_ratio() {
        assert!((px_mm(390.) / px_mm(844.) - 390. / 844.).abs() < 0.00001);
    }

    #[tokio::test]
    async fn pdf_export_smoke_test() {
        let document = serde_json::json!({"version":1,"elements":[{
            "id":"title","kind":"heading","x":20,"y":30,"width":240,"height":50,
            "text":"Print-ready mockup","fill":"#ffffff","color":"#172033","font_size":24
        }]});
        let response = pdf_response(vec![
            ("Home".into(), 390, 844, "phone".into(), document.clone()),
            ("Details".into(), 1440, 900, "browser".into(), document),
        ])
        .unwrap();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(bytes.starts_with(b"%PDF-"));
        assert!(bytes.len() > 500);
    }
}
