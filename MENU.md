# Context Menu en vsyncnotes — Estudio completo

## El problema

En macOS, Tauri 2 usa WKWebView (Safari). El clic derecho muestra un menu contextual nativo con items del SO y del navegador mezclados. Queremos:

1. **Eliminar** la mayoria de items por defecto (Look Up, Translate, Search Web, Share, Services, Font/Style, etc.)
2. **Conservar** Spelling & Grammar y Speech (son features del SO valiosas)
3. **Agregar** items propios de la app (acciones sobre notas, notebooks, editor)

---

## Items del menu nativo de WKWebView en macOS

| Identificador interno | Item visible | Conservar? |
|---|---|---|
| `_WKMenuItemIdentifierCopy` | Copy | No (reemplazar con Tiptap) |
| `_WKMenuItemIdentifierCut` | Cut | No (reemplazar con Tiptap) |
| `_WKMenuItemIdentifierPaste` | Paste | No (reemplazar con Tiptap) |
| `_WKMenuItemIdentifierLookUp` | Look Up | No |
| `_WKMenuItemIdentifierSearchWeb` | Search with Google | No |
| `_WKMenuItemIdentifierTranslate` | Translate | No |
| `_WKMenuItemIdentifierShareMenu` | Share | No |
| `_WKMenuItemIdentifierWritingTools` | Writing Tools (Sequoia+) | No |
| `_WKMenuItemIdentifierSpellingMenu` | **Spelling & Grammar** | **SI** |
| `_WKMenuItemIdentifierSpeechMenu` | **Speech** | **SI** |
| Services | Services submenu | No |
| Font/Style items | Font, Bold, etc. | No |
| Inspect Element | Dev tools | No (solo dev) |

---

## Enfoques analizados

### A. Pure JS: `preventDefault()` + menu HTML/CSS

```js
document.addEventListener('contextmenu', e => {
  e.preventDefault()
  showCustomMenu(e.clientX, e.clientY)
})
```

- `preventDefault()` **SI funciona** en WKWebView/Tauri y suprime el menu nativo
- Un `<div>` posicionado como menu

| Pro | Contra |
|---|---|
| Simple, control total del diseno | Pierde TODOS los items nativos |
| Sin dependencias Rust | No se siente nativo (no es NSMenu) |
| Facil de estilizar | Sin Spelling & Grammar ni Speech |
| | Sin integracion de accesibilidad macOS |

**Veredicto: No viable** si queremos conservar Spelling & Grammar.

---

### B. Tauri 2 `Menu.popup()` (API nativa built-in)

```typescript
import { Menu, MenuItem, PredefinedMenuItem } from '@tauri-apps/api/menu'

document.addEventListener('contextmenu', async (e) => {
  e.preventDefault()
  const menu = await Menu.new({
    items: [
      await PredefinedMenuItem.new({ item: 'Copy' }),
      await PredefinedMenuItem.new({ item: 'Cut' }),
      await PredefinedMenuItem.new({ item: 'Paste' }),
      await PredefinedMenuItem.new({ item: 'Separator' }),
      await MenuItem.new({ text: 'Eliminar nota', action: () => deleteNote() }),
    ]
  })
  await menu.popup() // NSMenu nativo real (via muda)
})
```

**`PredefinedMenuItem` disponibles:** `Separator`, `Copy`, `Cut`, `Paste`, `SelectAll`, `Undo`, `Redo`, `Minimize`, `Maximize`, `Fullscreen`, `Hide`, `HideOthers`, `ShowAll`, `CloseWindow`, `Quit`, `Services`, `About`

**NO disponibles:** Spelling & Grammar, Speech, Look Up, Share, Translate

| Pro | Contra |
|---|---|
| Menu nativo real (NSMenu) | Pierde Spelling & Grammar y Speech |
| Tema/animaciones del sistema | `preventDefault()` mata todo el menu nativo |
| Copy/Paste/Undo como PredefinedMenuItem | |
| Facil de implementar | |

**Veredicto: Buena opcion** si aceptamos perder Spelling & Grammar.

---

### C. `tauri-plugin-context-menu` (plugin externo)

- Solo soporta **Tauri v1** (modo mantenimiento)
- El autor recomienda usar la API nativa de Tauri v2
- **No soporta Spelling/Grammar/Speech**

**Veredicto: Irrelevante** para Tauri v2.

---

### D. WKWebView `willOpenMenu` swizzling (Objective-C desde Rust)

Esta es la **unica via tecnica** para conservar items nativos selectivamente.

**Como funciona:** WKWebView hereda de NSView que tiene:
```objc
- (void)willOpenMenu:(NSMenu *)menu withEvent:(NSEvent *)event;
```

Se llama ANTES de que el menu aparezca. Permite filtrar/modificar/agregar items al NSMenu.

**Implementacion conceptual en Rust:**

```rust
// En lib.rs setup, tras crear la ventana:
webview.with_webview(|wv| {
    #[cfg(target_os = "macos")]
    unsafe {
        // Obtener el WKWebView real
        let wk_view: *mut objc2::runtime::AnyObject = wv.inner().cast();
        // Swizzle willOpenMenu:withEvent: en su clase
        swizzle_context_menu(wk_view);
    }
})?;
```

El swizzle reemplazaria `willOpenMenu` por una implementacion custom que:
1. Itera `menu.itemArray`
2. Oculta/elimina items no deseados (Look Up, Translate, Search Web, Share, etc.)
3. Conserva Spelling & Grammar y Speech
4. Agrega items custom de la app

**Dependencias Rust necesarias:**
```toml
objc2 = "0.6"
objc2-app-kit = { version = "0.3", features = ["NSMenu", "NSMenuItem", "NSView", "NSEvent", "NSResponder"] }
objc2-foundation = "0.3"
```

| Pro | Contra |
|---|---|
| Conserva Spelling & Grammar nativo | Codigo `unsafe` no trivial |
| Conserva Speech nativo | Method swizzling es fragil |
| Menu sigue siendo NSMenu del sistema | Puede romperse con updates de Wry |
| Permite agregar items custom | Sin ejemplos en el ecosistema Tauri |
| Respeta tema, accesibilidad, etc. | Requiere conocimiento de objc2 |

**Veredicto: Unica solucion completa**, pero con complejidad significativa.

---

### E. Hibrido: Tauri `Menu.popup()` + Speech via Web API + Spelling nativo

Combina el enfoque B con alternativas JS para las features perdidas:

- **Spell check visual** (subrayado rojo): **funciona independientemente del menu** — es parte del renderer de WebKit en elementos `contenteditable` con `spellcheck="true"`. No se pierde al usar `preventDefault()`.
- **Speech**: reemplazable con `window.speechSynthesis.speak()` (Web Speech API, soportada en WebKit)
- **Panel de ortografia**: Se puede abrir con el atajo `Cmd+Shift+;` aunque no haya item de menu. Tambien se puede intentar invocar `orderFrontSpellingPanel:` via Tauri command con `objc2` (mucho mas simple que swizzle completo).

```typescript
// Speech via Web API
const speakSelection = () => {
  const text = window.getSelection()?.toString()
  if (text) {
    const utterance = new SpeechSynthesisUtterance(text)
    speechSynthesis.speak(utterance)
  }
}

// En el menu de Tauri:
await MenuItem.new({ text: 'Leer en voz alta', action: speakSelection })
```

| Pro | Contra |
|---|---|
| Menu nativo real | Spelling submenu no es el nativo (no tiene checkboxes de estado) |
| Speech funcional via Web API | Pierde "Check Spelling While Typing" toggle del menu |
| Spell check visual se conserva | No es 100% identico al menu del sistema |
| Mucho mas simple que swizzle | |
| Extensible con items custom | |

**Veredicto: Mejor compromiso** entre funcionalidad y complejidad.

---

## Tabla resumen

| Enfoque | Spelling & Grammar | Speech | Menu nativo | Items custom | Complejidad |
|---|---|---|---|---|---|
| A: HTML/CSS | NO | NO | NO | SI | Baja |
| B: Tauri Menu.popup() | NO | NO | SI (NSMenu) | SI | Media |
| C: Plugin v1 | NO | NO | SI | SI | N/A (obsoleto) |
| D: willOpenMenu swizzle | **SI (completo)** | **SI (completo)** | **SI (original)** | SI | **Alta** |
| E: Hibrido (B + Web APIs) | Parcial (visual si, submenu no) | SI (Web Speech API) | SI (NSMenu) | SI | Media |

---

## Recomendacion

### Para empezar: Enfoque E (Hibrido)

1. `preventDefault()` en `contextmenu` para zonas especificas (editor, lista de notas, arbol)
2. `Menu.popup()` de Tauri v2 para menus nativos NSMenu con items custom
3. Web Speech API para "Leer en voz alta"
4. El spell check visual sigue funcionando sin cambios
5. Opcionalmente, un comando Tauri simple que invoque `orderFrontSpellingPanel:` para abrir el panel de ortografia

### Para el futuro: Enfoque D (willOpenMenu swizzle)

Si la experiencia del enfoque E no es suficiente, se puede evolucionar a D para tener el submenu completo de Spelling & Grammar con todos sus toggles.

---

## Fuentes

- [iCab Blog — Customize the contextual menu of WKWebView on macOS](https://icab.de/blog/2022/06/12/customize-the-contextual-menu-of-wkwebview-on-macos/)
- [Michael Tsai — Customize the Contextual Menu of a Mac WKWebView](https://mjtsai.com/blog/2022/06/24/customize-the-contextual-menu-of-a-mac-wkwebview/)
- [WebKit source — WKMenuItemIdentifiers.mm](https://github.com/WebKit/WebKit/blob/main/Source/WebKit/UIProcess/API/Cocoa/WKMenuItemIdentifiers.mm)
- [Wry issue #30 — context menu disable via JS](https://github.com/tauri-apps/wry/issues/30)
- [Tauri discussion #11808 — disable context menu from Rust](https://github.com/orgs/tauri-apps/discussions/11808)
- [Tauri v2 Menu API](https://v2.tauri.app/reference/javascript/api/namespacemenu/)
- [Tauri v2 Window Menu docs](https://v2.tauri.app/learn/window-menu/)
- [tauri-plugin-context-menu (v1, mantenimiento)](https://github.com/c2r0b/tauri-plugin-context-menu)
- [Web Speech API — MDN](https://developer.mozilla.org/en-US/docs/Web/API/Web_Speech_API)
