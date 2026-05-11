/// Dev seeder — creates a populated vault for local development.
///
/// Password: "dev123"
/// Call once from the frontend console or a dev button.
/// Idempotent: no-ops if the vault already exists.
use chrono::Utc;
use tauri::State;
use uuid::Uuid;

use crate::{
    models::{note::Note, notebook::Notebook},
    storage::{fs_repo::FsRepo, repo::StorageRepo},
};

const DEV_PASSWORD: &str = "dev123";

// ── Tiptap JSON helpers ───────────────────────────────────────────────────────

fn doc(blocks: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({ "type": "doc", "content": blocks })
}
fn heading(level: u8, text: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "heading",
        "attrs": { "level": level },
        "content": [{ "type": "text", "text": text }]
    })
}
fn paragraph(text: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "paragraph",
        "content": [{ "type": "text", "text": text }]
    })
}
fn bold(text: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "text",
        "marks": [{ "type": "bold" }],
        "text": text
    })
}
fn paragraph_mixed(parts: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({ "type": "paragraph", "content": parts })
}
fn bullet_list(items: Vec<&str>) -> serde_json::Value {
    let list_items: Vec<serde_json::Value> = items
        .into_iter()
        .map(|t| {
            serde_json::json!({
                "type": "listItem",
                "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": t }] }]
            })
        })
        .collect();
    serde_json::json!({ "type": "bulletList", "content": list_items })
}
fn task_list(items: Vec<(&str, bool)>) -> serde_json::Value {
    let list_items: Vec<serde_json::Value> = items
        .into_iter()
        .map(|(t, checked)| {
            serde_json::json!({
                "type": "taskItem",
                "attrs": { "checked": checked },
                "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": t }] }]
            })
        })
        .collect();
    serde_json::json!({ "type": "taskList", "content": list_items })
}
fn blockquote(text: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "blockquote",
        "content": [{ "type": "paragraph", "content": [{ "type": "text", "text": text }] }]
    })
}
fn code_block(lang: &str, code: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "codeBlock",
        "attrs": { "language": lang },
        "content": [{ "type": "text", "text": code }]
    })
}

// ── Seed data ─────────────────────────────────────────────────────────────────

struct NbSpec {
    title: &'static str,
    sort_order: i32,
    parent: Option<usize>, // index into the notebooks vec
}

fn notebook_specs() -> Vec<NbSpec> {
    vec![
        NbSpec {
            title: "Personal",
            sort_order: 0,
            parent: None,
        }, // 0
        NbSpec {
            title: "Journal",
            sort_order: 0,
            parent: Some(0),
        }, // 1
        NbSpec {
            title: "Ideas",
            sort_order: 1,
            parent: Some(0),
        }, // 2
        NbSpec {
            title: "Work",
            sort_order: 1,
            parent: None,
        }, // 3
        NbSpec {
            title: "Projects",
            sort_order: 0,
            parent: Some(3),
        }, // 4
        NbSpec {
            title: "Meetings",
            sort_order: 1,
            parent: Some(3),
        }, // 5
        NbSpec {
            title: "Reference",
            sort_order: 2,
            parent: None,
        }, // 6
    ]
}

fn note_specs(nb_ids: &[Uuid]) -> Vec<(usize, Note)> {
    // Returns (notebook_index, note) pairs
    let now = Utc::now();

    let mut notes: Vec<(usize, Note)> = Vec::new();

    // ── Personal / Journal ────────────────────────────────────────────────────
    let mut n = Note::new(nb_ids[1], "Primer día con vsyncnotes".to_string());
    n.body = doc(vec![
        heading(1, "Primer día con vsyncnotes"),
        paragraph("Hoy empecé a usar vsyncnotes. La idea es tener todas mis notas cifradas y sincronizadas entre dispositivos sin depender de servicios en la nube propietarios."),
        blockquote("Privacy is not something that I'm merely entitled to, it's an absolute prerequisite. — Marlon Brando"),
        paragraph("Lo que más me gusta hasta ahora: el cifrado es transparente. Escribes, guardas, y todo queda protegido."),
    ]);
    n.is_pinned = true;
    notes.push((1, n));

    let mut n = Note::new(nb_ids[1], "Reflexiones de la semana".to_string());
    n.body = doc(vec![
        heading(2, "Semana del 3 de marzo"),
        paragraph("Productividad alta. Conseguí terminar el backend de la app sin distracciones."),
        bullet_list(vec![
            "Completar la fase 2 del proyecto",
            "Revisar la documentación de Tauri",
            "Leer sobre Argon2id",
        ]),
    ]);
    notes.push((1, n));

    let mut n = Note::new(nb_ids[1], "Cena con amigos en el centro".to_string());
    n.body = doc(vec![
        heading(2, "Cena en La Bodeguilla"),
        paragraph("Quedamos ocho para cenar. La comida estaba excelente como siempre — probamos el menú de temporada con setas."),
        paragraph("Buena conversación sobre planes de verano. Quizá organicemos un viaje juntos en julio."),
    ]);
    notes.push((1, n));

    let mut n = Note::new(
        nb_ids[1],
        "Primeras impresiones del libro Sapiens".to_string(),
    );
    n.body = doc(vec![
        heading(2, "Sapiens — Yuval Noah Harari"),
        paragraph("Llevo tres capítulos y engancha desde la primera página. La tesis sobre la revolución cognitiva y la capacidad de los humanos para creer en ficciones compartidas es reveladora."),
        paragraph("Quiero terminar el capítulo sobre la revolución agrícola antes del fin de semana."),
    ]);
    notes.push((1, n));

    let mut n = Note::new(nb_ids[1], "Lista de podcasts recomendados".to_string());
    n.body = doc(vec![
        heading(2, "Podcasts para escuchar"),
        bullet_list(vec![
            "Darknet Diaries — historias de ciberseguridad",
            "The Rustacean Station — novedades de Rust",
            "Radio Ambulante — crónicas latinoamericanas",
            "Lex Fridman Podcast — entrevistas largas sobre IA",
        ]),
        paragraph("Idea: crear una nota por podcast con los episodios favoritos."),
    ]);
    notes.push((1, n));

    let mut n = Note::new(nb_ids[1], "Reflexiones tras la caminata de hoy".to_string());
    n.body = doc(vec![
        heading(2, "Caminata por el parque del Oeste"),
        paragraph("Salí a caminar una hora sin móvil. Es increíble cómo se despeja la mente cuando te desconectas un rato."),
        paragraph("Se me ocurrió una solución para el bug del editor que me llevaba dos días dando vueltas. A veces el mejor debugging es no pensar en ello."),
    ]);
    notes.push((1, n));

    let mut n = Note::new(nb_ids[1], "Receta: pasta al pesto casero".to_string());
    n.body = doc(vec![
        heading(2, "Pesto genovés casero"),
        paragraph("Triturar albahaca fresca (50g), piñones (30g), ajo (1 diente), parmesano rallado (50g) y aceite de oliva virgen extra. Sal y pimienta al gusto."),
        paragraph("Clave: no calentar el pesto. Mezclar con la pasta caliente directamente para que no pierda el color verde intenso."),
    ]);
    notes.push((1, n));

    let mut n = Note::new(nb_ids[1], "Objetivos del mes de marzo".to_string());
    n.body = doc(vec![
        heading(2, "Marzo 2026"),
        task_list(vec![
            ("Terminar el MVP de vsyncnotes", false),
            ("Correr 3 veces por semana", false),
            ("Leer 2 libros", false),
            ("Organizar el escritorio y cables", true),
            ("Cita dentista día 18", true),
        ]),
    ]);
    notes.push((1, n));

    let mut n = Note::new(
        nb_ids[1],
        "Películas por ver este fin de semana".to_string(),
    );
    n.body = doc(vec![
        heading(2, "Watchlist"),
        bullet_list(vec![
            "Ex Machina (segundo visionado)",
            "The Imitation Game",
            "Arrival — Denis Villeneuve",
        ]),
        paragraph("Aprovechar que llueve todo el sábado para hacer maratón."),
    ]);
    notes.push((1, n));

    let mut n = Note::new(nb_ids[1], "Ideas para el balcón esta primavera".to_string());
    n.body = doc(vec![
        heading(2, "Plantas para el balcón"),
        paragraph("Comprar macetas de terracota medianas. Plantar tomates cherry, albahaca y pimientos. La lavanda puede ir en la esquina con más sol."),
        paragraph("Mirar un sistema de riego automático pequeño en IKEA o Leroy Merlin."),
    ]);
    notes.push((1, n));

    let mut n = Note::new(nb_ids[1], "Entrenamiento de la semana".to_string());
    n.body = doc(vec![
        heading(2, "Semana 10 — Cutting"),
        paragraph("Lunes: pecho + tríceps. Martes: espalda + bíceps. Miércoles: descanso. Jueves: piernas. Viernes: hombros + core."),
        paragraph("Peso estable en 74kg. El déficit calórico de 300kcal parece el punto dulce — pierdo grasa sin perder fuerza."),
    ]);
    notes.push((1, n));

    let mut n = Note::new(nb_ids[1], "Frase del día — Marco Aurelio".to_string());
    n.body = doc(vec![
        blockquote("You have power over your mind — not outside events. Realize this, and you will find strength."),
        paragraph("Recordatorio personal: no puedo controlar lo que pasa, solo cómo reacciono. Aplicable al bug de producción de ayer."),
    ]);
    notes.push((1, n));

    // ── Personal / Ideas ──────────────────────────────────────────────────────
    let mut n = Note::new(nb_ids[2], "App de meditación offline".to_string());
    n.body = doc(vec![
        heading(1, "App de meditación offline"),
        paragraph_mixed(vec![
            serde_json::json!({ "type": "text", "text": "Idea: una app de " }),
            bold("meditación guiada"),
            serde_json::json!({ "type": "text", "text": " que funcione 100% offline, con sesiones en audio cifrado." }),
        ]),
        heading(2, "Features clave"),
        bullet_list(vec![
            "Temporizador con sonidos de cuencos",
            "Sesiones guiadas sin conexión",
            "Estadísticas de práctica local",
            "Sin tracking, sin anuncios",
        ]),
    ]);
    notes.push((2, n));

    let mut n = Note::new(nb_ids[2], "Mejoras para vsyncnotes".to_string());
    n.body = doc(vec![
        heading(1, "Backlog de ideas"),
        task_list(vec![
            ("Soporte para tablas en el editor", true),
            ("Búsqueda full-text con tantivy", false),
            ("Sync con WebDAV", false),
            ("Export a Markdown", false),
            ("Plugin de syntax highlighting", false),
            ("Tema oscuro/claro", false),
        ]),
    ]);
    n.is_pinned = true;
    notes.push((2, n));

    // ── Work / Projects ───────────────────────────────────────────────────────
    let mut n = Note::new(nb_ids[4], "vsyncnotes — roadmap Q1".to_string());
    n.body = doc(vec![
        heading(1, "Roadmap Q1 2026"),
        paragraph("Objetivo: tener un MVP funcional con cifrado E2EE y sync básico."),
        heading(2, "Fases"),
        task_list(vec![
            ("Fase 0 — Bootstrap Tauri 2", true),
            ("Fase 1 — Modelo de datos y almacenamiento", true),
            ("Fase 2 — Cifrado E2EE", true),
            ("Fase 3 — UI: layout y árbol de notebooks", false),
            ("Fase 4 — Editor Tiptap", false),
            ("Fase 5 — Búsqueda por título", false),
            ("Fase 6 — Sync filesystem", false),
        ]),
    ]);
    n.is_pinned = true;
    notes.push((4, n));

    let mut n = Note::new(nb_ids[4], "Decisiones de arquitectura".to_string());
    n.body = doc(vec![
        heading(1, "Decisiones técnicas"),
        heading(2, "¿Por qué JSON y no SQLite?"),
        paragraph("Usar JSON en filesystem permite cifrar cada entidad de forma independiente. Con SQLite el archivo de base de datos sería una unidad opaca difícil de cifrar de forma granular."),
        heading(2, "¿Por qué Argon2id?"),
        paragraph("Es el ganador de la Password Hashing Competition (2015). Resistente a GPU cracking y side-channel attacks. Los parámetros por defecto son conservadores y seguros para 2026."),
        code_block("toml", "aes-gcm = \"0.10\"\nargon2 = \"0.5\"\nrand = \"0.8\"\nzeroize = \"1\""),
    ]);
    notes.push((4, n));

    // ── Work / Meetings ───────────────────────────────────────────────────────
    let mut n = Note::new(nb_ids[5], "Kick-off vsyncnotes — 2026-03-03".to_string());
    n.body = doc(vec![
        heading(1, "Kick-off — 3 mar 2026"),
        heading(2, "Asistentes"),
        bullet_list(vec!["Guillermo (dev)", "Claude (pair programmer)"]),
        heading(2, "Acuerdos"),
        task_list(vec![
            ("Stack: Tauri 2 + Vue 3 + Rust", true),
            ("Cifrado: AES-256-GCM + Argon2id", true),
            ("Almacenamiento inicial: JSON en filesystem", true),
            ("No usar SQLite (complejidad innecesaria)", true),
        ]),
        heading(2, "Próximos pasos"),
        paragraph("Implementar fases 0-2 antes de arrancar con la UI. El backend debe estar sólido antes de exponerlo al frontend."),
    ]);
    notes.push((5, n));

    // ── Reference ─────────────────────────────────────────────────────────────
    let mut n = Note::new(nb_ids[6], "Cheatsheet Tauri 2 — Commands".to_string());
    n.body = doc(vec![
        heading(1, "Tauri 2 — Commands cheatsheet"),
        paragraph("Cómo definir y registrar commands en Tauri 2."),
        code_block("rust", "#[tauri::command]\npub async fn my_command(\n    state: State<'_, MyState>,\n    arg: String,\n) -> Result<String, String> {\n    Ok(format!(\"Hello, {}!\", arg))\n}"),
        paragraph("Registro en lib.rs:"),
        code_block("rust", ".invoke_handler(tauri::generate_handler![\n    my_command,\n])"),
        paragraph("Llamada desde el frontend:"),
        code_block("typescript", "import { invoke } from '@tauri-apps/api/core'\n\nconst result = await invoke<string>('my_command', { arg: 'world' })"),
    ]);
    notes.push((6, n));

    let mut n = Note::new(nb_ids[6], "AES-GCM — notas de seguridad".to_string());
    n.body = doc(vec![
        heading(1, "AES-256-GCM — notas de implementación"),
        heading(2, "Nonce"),
        paragraph("NUNCA reutilizar un nonce con la misma clave. En vsyncnotes cada cifrado genera un nonce aleatorio de 12 bytes (96 bits) via OsRng."),
        blockquote("Nonce reuse in GCM is catastrophic — reveals the authentication key and allows forgery."),
        heading(2, "Envelope encryption"),
        paragraph("Cada nota tiene su propio DEK (Data Encryption Key) aleatorio de 256 bits. El DEK se cifra con el master_key derivado de la contraseña. Cambiar la contraseña solo requiere re-cifrar los DEKs, no los datos."),
        heading(2, "Zeroize"),
        paragraph("El master_key se almacena en Zeroizing<[u8;32]> que sobreescribe la memoria al hacer drop. Al hacer vault_lock() se descarta la clave de memoria de forma segura."),
    ]);
    notes.push((6, n));

    // Adjust timestamps so notes appear in a natural order
    let total = notes.len();
    for (i, (_, note)) in notes.iter_mut().enumerate() {
        note.created_at = now - chrono::Duration::days(7) + chrono::Duration::hours(i as i64);
        note.updated_at = now - chrono::Duration::hours((total - i) as i64);
    }

    notes
}

// ── Command ───────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn dev_seed(repo: State<'_, FsRepo>) -> Result<serde_json::Value, String> {
    // No-op if vault already exists
    if repo.vault_exists().await {
        return Ok(serde_json::json!({
            "skipped": true,
            "reason": "Vault already exists — delete $APP_DATA/vault to re-seed"
        }));
    }

    // Create vault
    repo.vault_create(DEV_PASSWORD)
        .await
        .map_err(|e| e.to_string())?;

    // Create notebooks
    let specs = notebook_specs();
    let mut nb_ids: Vec<Uuid> = Vec::with_capacity(specs.len());

    for spec in &specs {
        let parent_id = spec.parent.map(|i| nb_ids[i]);
        let mut nb = Notebook::new(spec.title.to_string(), parent_id);
        nb.sort_order = spec.sort_order;
        repo.save_notebook(&nb).await.map_err(|e| e.to_string())?;
        nb_ids.push(nb.id);
    }

    // Create notes (with the real notebook UUIDs)
    let note_specs = note_specs(&nb_ids);
    let note_count = note_specs.len();

    for (_, note) in note_specs {
        repo.save_note(&note).await.map_err(|e| e.to_string())?;
    }

    Ok(serde_json::json!({
        "skipped": false,
        "password": DEV_PASSWORD,
        "notebooks": nb_ids.len(),
        "notes": note_count,
    }))
}
